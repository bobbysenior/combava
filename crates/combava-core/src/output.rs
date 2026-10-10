use std::ops::Range;

use crate::source_map::SourceMap;

/// Résultat d'une transpilation réussie.
#[derive(Debug, Clone)]
pub struct Output {
    /// Le code Typst complet, à écrire tel quel dans `paths::MAIN`.
    pub typst: String,
    /// Uniquement des avertissements, triés par `span.start`.
    pub diagnostics: Vec<Diagnostic>,
    pub source_map: SourceMap,
}

/// Un problème rattaché à une position du markdown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: Code,
    pub message: String,
    /// Plage d'octets dans le markdown reçu.
    pub span: Range<usize>,
    /// Ligne de `span.start`, à partir de 1.
    pub line: usize,
    /// Colonne de `span.start` en caractères, à partir de 1.
    pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Warning,
    Error,
}

/// Codes de diagnostic du core (spécification, section 2.4).
///
/// `non_exhaustive` : le CLI ne doit pas faire de `match` exhaustif.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Code {
    RemoteImage,
    InvalidImagePath,
    BrokenLink,
    DuplicateLabel,
    CitationWithoutBibliography,
    RawHtml,
    InvalidLabel,
    InvalidCodeAttributes,
    UndefinedFootnote,
    UnusedFootnote,
}

impl Code {
    /// Identifiant stable en kebab-case, par exemple `"raw-html"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Code::RemoteImage => "remote-image",
            Code::InvalidImagePath => "invalid-image-path",
            Code::BrokenLink => "broken-link",
            Code::DuplicateLabel => "duplicate-label",
            Code::CitationWithoutBibliography => "citation-without-bibliography",
            Code::RawHtml => "raw-html",
            Code::InvalidLabel => "invalid-label",
            Code::InvalidCodeAttributes => "invalid-code-attributes",
            Code::UndefinedFootnote => "undefined-footnote",
            Code::UnusedFootnote => "unused-footnote",
        }
    }

    /// Gravité fixe du code.
    pub fn severity(self) -> Severity {
        match self {
            Code::RemoteImage
            | Code::InvalidImagePath
            | Code::BrokenLink
            | Code::DuplicateLabel
            | Code::CitationWithoutBibliography => Severity::Error,
            Code::RawHtml
            | Code::InvalidLabel
            | Code::InvalidCodeAttributes
            | Code::UndefinedFootnote
            | Code::UnusedFootnote => Severity::Warning,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [Code; 10] = [
        Code::RemoteImage,
        Code::InvalidImagePath,
        Code::BrokenLink,
        Code::DuplicateLabel,
        Code::CitationWithoutBibliography,
        Code::RawHtml,
        Code::InvalidLabel,
        Code::InvalidCodeAttributes,
        Code::UndefinedFootnote,
        Code::UnusedFootnote,
    ];

    #[test]
    fn identifiants_en_kebab_case_et_uniques() {
        let mut seen = std::collections::HashSet::new();
        for code in ALL {
            let id = code.as_str();
            assert!(
                id.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
                "{id}"
            );
            assert!(seen.insert(id), "doublon : {id}");
        }
    }

    #[test]
    fn gravites_conformes_a_la_specification() {
        let errors: Vec<_> = ALL
            .into_iter()
            .filter(|c| c.severity() == Severity::Error)
            .map(Code::as_str)
            .collect();
        assert_eq!(
            errors,
            [
                "remote-image",
                "invalid-image-path",
                "broken-link",
                "duplicate-label",
                "citation-without-bibliography"
            ]
        );
    }
}
