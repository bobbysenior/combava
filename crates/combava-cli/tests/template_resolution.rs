//! Résolution des templates (spécification, section 7).

use std::path::{Path, PathBuf};

use combava_cli::config::layers::TemplateRef;
use combava_cli::template::{self, Template};
use tempfile::TempDir;

struct Dirs {
    dir: TempDir,
}

impl Dirs {
    fn new() -> Self {
        Self {
            dir: tempfile::tempdir().unwrap(),
        }
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.dir.path().join(relative)
    }

    fn template(&self, relative: &str) -> PathBuf {
        let dir = self.path(relative);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("template.typ"), relative).unwrap();
        dir
    }

    fn project(&self) -> Option<PathBuf> {
        Some(self.path("projet"))
    }

    fn global(&self) -> Option<PathBuf> {
        Some(self.path("globale"))
    }

    fn resolve(&self, name: &str) -> Result<Template, String> {
        template::resolve(
            &TemplateRef::Name(name.into()),
            self.project().as_deref(),
            self.global().as_deref(),
        )
    }
}

fn disk(template: Template) -> PathBuf {
    match template {
        Template::Disk(dir) => dir,
        other => panic!("template embarqué : {other:?}"),
    }
}

#[test]
fn template_embarque_par_defaut() {
    let dirs = Dirs::new();
    let template = dirs.resolve("default").unwrap();
    assert!(matches!(template, Template::Embedded { ref name, .. } if name == "default"));
    assert_eq!(template.name(), "default");
    assert!(template.read("template.typ").is_some());
    assert!(template.read("absent.typ").is_none());
    assert_eq!(
        template.display_path("template.typ"),
        Path::new("<default>/template.typ")
    );
    // Sans racine de projet ni configuration globale.
    assert!(template::resolve(&TemplateRef::Name("default".into()), None, None).is_ok());
}

#[test]
fn ordre_de_recherche() {
    let dirs = Dirs::new();
    let global = dirs.template("globale/templates/maison");
    assert_eq!(dirs.resolve("maison").unwrap().name(), "maison");
    assert_eq!(disk(dirs.resolve("maison").unwrap()), global);
    let project = dirs.template("projet/.combava/templates/maison");
    assert_eq!(disk(dirs.resolve("maison").unwrap()), project);
}

#[test]
fn un_dossier_default_remplace_le_template_embarque() {
    let dirs = Dirs::new();
    let global = dirs.template("globale/templates/default");
    assert_eq!(disk(dirs.resolve("default").unwrap()), global);
    let project = dirs.template("projet/.combava/templates/default");
    let template = dirs.resolve("default").unwrap();
    assert_eq!(
        template.read("template.typ").unwrap().unwrap(),
        b"projet/.combava/templates/default"
    );
    assert_eq!(disk(template), project);
}

#[test]
fn dossier_sans_template_typ() {
    let dirs = Dirs::new();
    dirs.template("globale/templates/maison");
    std::fs::create_dir_all(dirs.path("projet/.combava/templates/maison")).unwrap();
    // Le dossier du projet l'emporte, même incomplet : pas de repli silencieux.
    let error = dirs.resolve("maison").unwrap_err();
    assert!(
        error.starts_with("template « maison » introuvable : le dossier « "),
        "{error}"
    );
    assert!(
        error.ends_with("ne contient pas de template.typ"),
        "{error}"
    );
}

#[test]
fn introuvable() {
    let dirs = Dirs::new();
    let error = dirs.resolve("absent").unwrap_err();
    let project = dirs.path("projet/.combava/templates/absent");
    let global = dirs.path("globale/templates/absent");
    assert_eq!(
        error,
        format!(
            "template « absent » introuvable ; emplacements essayés : « {} », « {} », les templates embarqués",
            project.display(),
            global.display()
        )
    );
    assert!(
        template::resolve(&TemplateRef::Name("absent".into()), None, None)
            .unwrap_err()
            .ends_with("emplacements essayés : les templates embarqués")
    );
    assert!(dirs.resolve("").is_err());
}

#[test]
fn chemins() {
    let dirs = Dirs::new();
    let dir = dirs.template("ailleurs/t");
    let resolve = |path: &Path| template::resolve(&TemplateRef::Path(path.into()), None, None);
    assert_eq!(disk(resolve(&dir).unwrap()), dir);
    let error = resolve(&dirs.path("absent")).unwrap_err();
    assert!(
        error.starts_with("template introuvable : le dossier « "),
        "{error}"
    );
    assert!(error.ends_with("» n'existe pas"), "{error}");
}

#[test]
fn polices_du_template() {
    use combava_cli::template::Fonts;
    let dirs = Dirs::new();
    let dir = dirs.template("t");
    assert!(matches!(
        Template::Disk(dir.clone()).fonts(),
        Fonts::Dir(None)
    ));
    std::fs::create_dir(dir.join("fonts")).unwrap();
    assert!(
        matches!(Template::Disk(dir.clone()).fonts(), Fonts::Dir(Some(ref d)) if *d == dir.join("fonts"))
    );
}
