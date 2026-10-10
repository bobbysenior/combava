//! Codes de diagnostic du CLI (spécification, section 5.6).

/// Erreur ou avertissement détecté par le CLI. Les codes du core sont repris
/// tels quels par [`combava_core::Code::as_str`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Code {
    InvalidUtf8,
    UnclosedFrontmatter,
    InvalidToml,
    UnknownKey,
    InvalidType,
    OutputOutsideFrontmatter,
    TemplateNotFound,
    BibliographyNotFound,
    FileExists,
    PackageDownload,
    Typst,
    Io,
}

impl Code {
    /// Identifiant stable en kebab-case, affiché entre crochets.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidUtf8 => "invalid-utf8",
            Self::UnclosedFrontmatter => "unclosed-frontmatter",
            Self::InvalidToml => "invalid-toml",
            Self::UnknownKey => "unknown-key",
            Self::InvalidType => "invalid-type",
            Self::OutputOutsideFrontmatter => "output-outside-frontmatter",
            Self::TemplateNotFound => "template-not-found",
            Self::BibliographyNotFound => "bibliography-not-found",
            Self::FileExists => "file-exists",
            Self::PackageDownload => "package-download",
            Self::Typst => "typst",
            Self::Io => "io",
        }
    }
}

/// La commande s'est arrêtée sur une erreur, déjà affichée : le code de
/// sortie vaut 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Failed;

/// Message en français d'une erreur d'entrée-sortie, sans point final.
pub fn io_message(error: &std::io::Error) -> String {
    use std::io::ErrorKind;
    match error.kind() {
        ErrorKind::NotFound => "fichier ou dossier introuvable".to_string(),
        ErrorKind::PermissionDenied => "permission refusée".to_string(),
        ErrorKind::IsADirectory => "c'est un dossier".to_string(),
        ErrorKind::AlreadyExists => "le fichier existe déjà".to_string(),
        _ => error.to_string(),
    }
}
