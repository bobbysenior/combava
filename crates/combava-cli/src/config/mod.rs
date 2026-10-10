//! Configuration d'un document : couches fusionnées puis résolues
//! (spécification, sections 6 et 7).

pub mod layers;
pub mod paths;

use std::ops::Range;
use std::path::{Path, PathBuf};

use combava_core::Config;

use crate::diagnostics::{Diagnostic, Location, display_path};
use crate::error::{Code, io_message};
use crate::template::{self, Template};
use layers::{Input, Layer, Setting, Source, TemplateRef};
use paths::Env;

/// Les valeurs données en arguments de `combava build`.
#[derive(Debug, Clone, Default)]
pub struct Overrides {
    pub template: Option<String>,
    pub output: Option<PathBuf>,
}

/// Tout ce qu'il faut pour transpiler et compiler un document.
#[derive(Debug, Clone)]
pub struct Settings {
    pub config: Config,
    pub template: Template,
    /// Le fichier `.bib`, qui existe.
    pub bibliography: Option<PathBuf>,
    /// Le fichier de sortie, dont le dossier parent existe.
    pub output: PathBuf,
}

/// Lit les couches de configuration du document `markdown` (chemin absolu,
/// contenu `text`, frontmatter à la plage `frontmatter`), les fusionne et
/// résout le template, la bibliographie et le fichier de sortie.
pub fn load(
    markdown: &Path,
    text: &str,
    frontmatter: Option<Range<usize>>,
    overrides: &Overrides,
    env: &Env,
) -> Result<Settings, Vec<Diagnostic>> {
    let md_dir = markdown.parent().unwrap_or(Path::new("/"));
    let project_root = paths::project_root(md_dir);
    let mut errors = Vec::new();
    let mut read = |result: Result<Layer, Vec<Diagnostic>>| {
        result.unwrap_or_else(|diagnostics| {
            errors.extend(diagnostics);
            Layer::default()
        })
    };

    let arguments = Layer {
        template: overrides.template.as_ref().map(|value| Setting {
            value: TemplateRef::parse(value, &env.cwd, env),
            location: Location::None,
        }),
        output: overrides.output.as_ref().map(|path| Setting {
            value: env.argument(path),
            location: Location::None,
        }),
        ..Layer::default()
    };
    let frontmatter = read(match frontmatter {
        Some(range) => layers::parse(
            &Input {
                source: Source::Frontmatter,
                path: markdown,
                text,
                range,
                base: md_dir,
            },
            env,
        ),
        None => Ok(Layer::default()),
    });
    let project = read(match &project_root {
        Some(root) => config_file(Source::Project, root, &root.join(".combava"), env),
        None => Ok(Layer::default()),
    });
    let global = read(match &env.global_config {
        Some(dir) => config_file(Source::Global, dir, dir, env),
        None => Ok(Layer::default()),
    });
    if !errors.is_empty() {
        return Err(errors);
    }

    let merged = arguments.or(frontmatter).or(project).or(global);
    resolve(merged, markdown, project_root.as_deref(), env)
}

/// Lit `<dir>/config.toml` ; un fichier absent ne définit aucune clé. Les
/// chemins relatifs le sont à `base`.
fn config_file(
    source: Source,
    base: &Path,
    dir: &Path,
    env: &Env,
) -> Result<Layer, Vec<Diagnostic>> {
    let path = dir.join("config.toml");
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Layer::default());
        }
        Err(error) => {
            let message = format!(
                "impossible de lire « {} » : {}",
                display_path(&path, &env.cwd),
                io_message(&error)
            );
            return Err(vec![Diagnostic::error(Code::Io, message)]);
        }
    };
    let Ok(text) = String::from_utf8(bytes) else {
        let error = Diagnostic::error(
            Code::InvalidToml,
            "TOML invalide : le fichier n'est pas en UTF-8",
        )
        .at(Location::File(path));
        return Err(vec![error]);
    };
    let input = Input {
        source,
        path: &path,
        text: &text,
        range: 0..text.len(),
        base,
    };
    layers::parse(&input, env)
}

/// Transforme la couche fusionnée en réglages complets.
fn resolve(
    layer: Layer,
    markdown: &Path,
    project_root: Option<&Path>,
    env: &Env,
) -> Result<Settings, Vec<Diagnostic>> {
    let mut errors = Vec::new();
    let show = |path: &Path| display_path(path, &env.cwd);

    let template_ref = layer.template.unwrap_or(Setting {
        value: TemplateRef::Name("default".into()),
        location: Location::None,
    });
    let template = template::resolve(
        &template_ref.value,
        project_root,
        env.global_config.as_deref(),
    )
    .map_err(|message| {
        errors.push(Diagnostic::error(Code::TemplateNotFound, message).at(template_ref.location));
    })
    .ok();

    let bibliography = layer.bibliography.and_then(|Setting { value, location }| {
        let is_bib = value
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("bib"));
        let message = if !value.is_file() {
            format!("fichier de bibliographie « {} » introuvable", show(&value))
        } else if !is_bib {
            format!(
                "le fichier de bibliographie « {} » n'a pas l'extension .bib",
                show(&value)
            )
        } else {
            return Some(value);
        };
        errors.push(Diagnostic::error(Code::BibliographyNotFound, message).at(location));
        None
    });

    let Setting {
        value: output,
        location,
    } = layer.output.unwrap_or_else(|| Setting {
        value: markdown.with_extension("pdf"),
        location: Location::None,
    });
    let parent = output.parent().filter(|p| !p.as_os_str().is_empty());
    if let Some(parent) = parent.filter(|p| !p.is_dir()) {
        let message = format!(
            "le dossier « {} » du fichier de sortie n'existe pas",
            show(parent)
        );
        errors.push(Diagnostic::error(Code::Io, message).at(location));
    }

    if !errors.is_empty() {
        return Err(errors);
    }
    let config = Config {
        title: layer.title,
        subtitle: layer.subtitle,
        authors: layer.authors.unwrap_or_default(),
        teachers: layer.teachers.unwrap_or_default(),
        date: layer.date,
        school: layer.school,
        university: layer.university,
        academic_year: layer.academic_year,
        cohort: layer.cohort,
        specialization: layer.specialization,
        subject: layer.subject,
        toc: layer.toc.unwrap_or(true),
        list_of_figures: layer.list_of_figures.unwrap_or(false),
        list_of_listings: layer.list_of_listings.unwrap_or(false),
        // Laissé à `None` s'il est absent : le core le remplace par le titre.
        header_text: layer.header_text,
        bibliography: bibliography.is_some(),
    };
    Ok(Settings {
        config,
        template: template.expect("une erreur de template a été rapportée"),
        bibliography,
        output,
    })
}
