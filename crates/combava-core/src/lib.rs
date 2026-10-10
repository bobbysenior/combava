//! Transpilation markdown → Typst de Combava.
//!
//! Le contrat de ce crate est décrit dans `docs/specification.md`, sections 2 à 4.
//! Il ne lit ni n'écrit aucun fichier : le CLI fournit le markdown et la
//! configuration, et compile le Typst renvoyé.

mod ast;
mod config;
mod diag;
mod emit;
mod error;
mod escape;
mod label;
mod output;
mod parser;
mod path;
mod source_map;

pub use config::Config;
pub use error::TranspileError;
pub use output::{Code, Diagnostic, Output, Severity};
pub use source_map::SourceMap;

/// Chemins virtuels partagés avec le `World` du CLI.
pub mod paths {
    /// Le code généré.
    pub const MAIN: &str = "/__combava__/main.typ";
    /// Le dossier du template résolu.
    pub const TEMPLATE_DIR: &str = "/__combava__/template";
    /// Le point d'entrée du template, importé par le code généré.
    pub const TEMPLATE_ENTRY: &str = "/__combava__/template/template.typ";
    /// Le fichier de bibliographie résolu.
    pub const BIBLIOGRAPHY: &str = "/__combava__/bibliography.bib";
}

/// Package Typst utilisé pour les maths, version épinglée.
pub const MITEX_PACKAGE: &str = "@preview/mitex:0.2.7";

/// Transpile le corps markdown d'un document en code Typst complet.
///
/// `markdown` est le contenu complet du fichier, frontmatter remplacé par des
/// espaces (spécification, section 2.2). Toutes les erreurs du document sont
/// rapportées en une fois.
pub fn transpile(markdown: &str, config: &Config) -> Result<Output, TranspileError> {
    let mut diags = diag::Diagnostics::new(markdown);
    let document = parser::parse(markdown, &mut diags);
    let (typst, source_map) = emit::document(&document, config, markdown, &mut diags);
    let diagnostics = diags.into_sorted();

    if diagnostics.iter().any(|d| d.severity == Severity::Error) {
        Err(TranspileError { diagnostics })
    } else {
        Ok(Output {
            typst,
            diagnostics,
            source_map,
        })
    }
}
