//! Compilation du code Typst généré en PDF (spécification, section 8).

pub mod pdf;
pub mod world;

use std::path::{Path, PathBuf};

use combava_core::SourceMap;
use typst::diag::{Severity as TypstSeverity, SourceDiagnostic, Warned};
use typst::{World, WorldExt};
use typst_layout::PagedDocument;

use crate::diagnostics::{Diagnostic, Location, Severity, position};
use crate::error::Code;
use crate::template::Template;
use world::{CombavaWorld, Files};

/// Un document à compiler.
pub struct Job<'a> {
    /// `Output::typst`, tel quel.
    pub typst: String,
    pub source_map: &'a SourceMap,
    /// Le fichier markdown, chemin absolu.
    pub markdown: &'a Path,
    pub template: Template,
    pub bibliography: Option<PathBuf>,
}

/// Résultat de la compilation : le PDF si elle a réussi, et ses diagnostics.
pub struct Compiled {
    pub pdf: Option<Vec<u8>>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn compile(job: Job) -> Compiled {
    let world = CombavaWorld::new(Files {
        main: job.typst,
        root: job
            .markdown
            .parent()
            .unwrap_or(Path::new("/"))
            .to_path_buf(),
        template: job.template,
        bibliography: job.bibliography,
    });
    let convert =
        |diagnostic: &SourceDiagnostic| convert(&world, diagnostic, job.markdown, job.source_map);

    let Warned { output, warnings } = typst::compile::<PagedDocument>(&world);
    let mut diagnostics: Vec<_> = warnings.iter().map(convert).collect();
    match output.and_then(|document| pdf::export(&document)) {
        Ok(pdf) => Compiled {
            pdf: Some(pdf),
            diagnostics,
        },
        Err(errors) => {
            // Un package manquant fait échouer tout ce qui l'utilise : seule la
            // cause est rapportée.
            match world.download_failure() {
                Some((spec, reason)) => diagnostics.push(
                    Diagnostic::error(
                        Code::PackageDownload,
                        format!("impossible de télécharger le package « {spec} » : {reason}"),
                    )
                    .with_hint(
                        "une connexion est nécessaire au premier usage des maths ; \
                         le package est ensuite gardé en cache",
                    ),
                ),
                None => diagnostics.extend(errors.iter().map(convert)),
            }
            Compiled {
                pdf: None,
                diagnostics,
            }
        }
    }
}

/// Un diagnostic Typst, ramené dans le markdown quand il vient du code
/// généré.
fn convert(
    world: &CombavaWorld,
    diagnostic: &SourceDiagnostic,
    markdown: &Path,
    source_map: &SourceMap,
) -> Diagnostic {
    let mut message = diagnostic.message.to_string();
    let range = world.range(diagnostic.span);
    let location = match (diagnostic.span.id(), range) {
        (None, _) => Location::None,
        (Some(id), None) if id == world.main_id() => Location::File(markdown.to_path_buf()),
        (Some(id), None) => Location::File(world.display_path(id)),
        (Some(id), Some(range)) if id == world.main_id() => {
            let typst_line = world
                .source(id)
                .map(|source| position(source.text(), range.start).0)
                .unwrap_or(0);
            match source_map.markdown_line(typst_line) {
                Some(line) => Location::Position {
                    path: markdown.to_path_buf(),
                    line,
                    column: 1,
                },
                None => {
                    message.push_str(&format!(
                        " (ligne {typst_line} du code généré, voir --transpile-only)"
                    ));
                    Location::File(markdown.to_path_buf())
                }
            }
        }
        (Some(id), Some(range)) => {
            let path = world.display_path(id);
            match file_text(world, id) {
                Some(text) => Location::at(&path, &text, range.start),
                None => Location::File(path),
            }
        }
    };
    Diagnostic {
        severity: match diagnostic.severity {
            TypstSeverity::Error => Severity::Error,
            TypstSeverity::Warning => Severity::Warning,
        },
        code: Code::Typst.as_str(),
        message,
        location,
        hints: diagnostic
            .hints
            .iter()
            .map(|hint| hint.v.to_string())
            .collect(),
    }
}

/// Le texte d'un fichier, tel que Typst l'a lu (sans BOM pour une source).
fn file_text(world: &CombavaWorld, id: typst::syntax::FileId) -> Option<String> {
    if let Ok(source) = world.source(id) {
        return Some(source.text().to_string());
    }
    let bytes = world.file(id).ok()?;
    String::from_utf8(bytes.to_vec()).ok()
}
