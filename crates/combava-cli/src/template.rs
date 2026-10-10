//! Résolution du template et templates embarqués (spécification, section 7).

use std::path::{Path, PathBuf};

use include_dir::{Dir, include_dir};

use crate::config::layers::TemplateRef;

/// Les templates embarqués dans le binaire : un sous-dossier par nom.
static EMBEDDED: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../../templates");

/// Le fichier d'entrée d'un template.
pub const ENTRY: &str = "template.typ";

/// Un template résolu : un dossier contenant `template.typ`.
#[derive(Debug, Clone)]
pub enum Template {
    Disk(PathBuf),
    Embedded {
        name: String,
        dir: &'static Dir<'static>,
    },
}

impl Template {
    /// Lit le fichier `relative` (sans `/` de tête) du template. `None` s'il
    /// n'existe pas.
    pub fn read(&self, relative: &str) -> Option<std::io::Result<Vec<u8>>> {
        match self {
            Self::Disk(dir) => {
                let path = dir.join(relative);
                match std::fs::read(&path) {
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                    result => Some(result),
                }
            }
            Self::Embedded { dir, .. } => {
                let path = dir.path().join(relative);
                EMBEDDED
                    .get_file(path)
                    .map(|file| Ok(file.contents().to_vec()))
            }
        }
    }

    /// Nom affiché dans les messages : celui du template embarqué, ou celui
    /// de son dossier.
    pub fn name(&self) -> String {
        match self {
            Self::Disk(dir) => dir
                .file_name()
                .unwrap_or(dir.as_os_str())
                .to_string_lossy()
                .into_owned(),
            Self::Embedded { name, .. } => name.clone(),
        }
    }

    /// Chemin affiché dans les diagnostics pour le fichier `relative` : le
    /// chemin réel, ou `<nom>/…` pour un template embarqué.
    pub fn display_path(&self, relative: &str) -> PathBuf {
        match self {
            Self::Disk(dir) => dir.join(relative),
            Self::Embedded { name, .. } => PathBuf::from(format!("<{name}>/{relative}")),
        }
    }

    /// Les polices du dossier `fonts/` du template (section 8) : chemins sur
    /// le disque, ou contenus embarqués.
    pub fn fonts(&self) -> Fonts {
        match self {
            Self::Disk(dir) => {
                let fonts = dir.join("fonts");
                Fonts::Dir(fonts.is_dir().then_some(fonts))
            }
            Self::Embedded { dir, .. } => {
                let mut files = Vec::new();
                if let Some(fonts) = EMBEDDED.get_dir(dir.path().join("fonts")) {
                    collect_fonts(fonts, &mut files);
                }
                Fonts::Embedded(files)
            }
        }
    }
}

/// Les polices fournies par un template.
pub enum Fonts {
    Dir(Option<PathBuf>),
    Embedded(Vec<&'static [u8]>),
}

const FONT_EXTENSIONS: [&str; 4] = ["ttf", "otf", "ttc", "otc"];

fn collect_fonts(dir: &'static Dir<'static>, out: &mut Vec<&'static [u8]>) {
    for file in dir.files() {
        let extension = file.path().extension().and_then(|e| e.to_str());
        if extension.is_some_and(|e| FONT_EXTENSIONS.iter().any(|f| e.eq_ignore_ascii_case(f))) {
            out.push(file.contents());
        }
    }
    for sub in dir.dirs() {
        collect_fonts(sub, out);
    }
}

/// Résout un template. En cas d'échec, renvoie le message de l'erreur
/// `template-not-found`, qui liste les emplacements essayés.
pub fn resolve(
    template: &TemplateRef,
    project_root: Option<&Path>,
    global_config: Option<&Path>,
) -> Result<Template, String> {
    let name = match template {
        TemplateRef::Path(dir) => return disk(dir, None),
        TemplateRef::Name(name) => name,
    };

    let mut tried = Vec::new();
    let candidates = [
        project_root.map(|root| root.join(".combava").join("templates")),
        global_config.map(|dir| dir.join("templates")),
    ];
    for dir in candidates.into_iter().flatten() {
        let dir = dir.join(name);
        // Le premier dossier existant l'emporte, même sans `template.typ` :
        // un template utilisateur incomplet n'est pas remplacé en silence.
        if dir.is_dir() {
            return disk(&dir, Some(name));
        }
        tried.push(format!("« {} »", dir.display()));
    }

    if !name.is_empty()
        && let Some(dir) = EMBEDDED.get_dir(name)
        && dir.get_file(dir.path().join(ENTRY)).is_some()
    {
        return Ok(Template::Embedded {
            name: name.clone(),
            dir,
        });
    }
    tried.push("les templates embarqués".to_string());
    Err(format!(
        "template « {name} » introuvable ; emplacements essayés : {}",
        tried.join(", ")
    ))
}

fn disk(dir: &Path, name: Option<&str>) -> Result<Template, String> {
    let label = name.map(|name| format!(" « {name} »")).unwrap_or_default();
    if dir.join(ENTRY).is_file() {
        Ok(Template::Disk(dir.to_path_buf()))
    } else if dir.is_dir() {
        Err(format!(
            "template{label} introuvable : le dossier « {} » ne contient pas de {ENTRY}",
            dir.display()
        ))
    } else {
        Err(format!(
            "template{label} introuvable : le dossier « {} » n'existe pas",
            dir.display()
        ))
    }
}
