//! Diagnostics du CLI et leur affichage (spécification, section 5.4).

use std::io::Write;
use std::path::{Path, PathBuf};

pub use combava_core::Severity;

use crate::error::{Code, Failed};

/// Où pointe un diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Location {
    /// Ni fichier ni position : `combava: erreur[…] : …`.
    None,
    /// Un fichier, sans ligne : `rapport.md: erreur[…] : …`.
    File(PathBuf),
    /// Une position, lignes et colonnes numérotées à partir de 1.
    Position {
        path: PathBuf,
        line: usize,
        column: usize,
    },
}

impl Location {
    /// Position de l'octet `offset` de `text`, qui est le contenu de `path`.
    pub fn at(path: &Path, text: &str, offset: usize) -> Self {
        let (line, column) = position(text, offset);
        Self::Position {
            path: path.to_path_buf(),
            line,
            column,
        }
    }
}

/// Un diagnostic prêt à afficher, venu du CLI, du core ou de Typst.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    /// Code en kebab-case : section 5.6 pour le CLI, 2.4 pour le core.
    pub code: &'static str,
    pub message: String,
    pub location: Location,
    /// Indications affichées sous le diagnostic, une par ligne.
    pub hints: Vec<String>,
}

impl Diagnostic {
    /// Une erreur du CLI, sans position.
    pub fn error(code: Code, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            code: code.as_str(),
            message: message.into(),
            location: Location::None,
            hints: Vec::new(),
        }
    }

    pub fn at(mut self, location: Location) -> Self {
        self.location = location;
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hints.push(hint.into());
        self
    }

    /// Un diagnostic du core, dont la position est déjà celle du fichier
    /// `markdown` (section 2.2).
    pub fn from_core(diagnostic: &combava_core::Diagnostic, markdown: &Path) -> Self {
        Self {
            severity: diagnostic.severity,
            code: diagnostic.code.as_str(),
            message: diagnostic.message.clone(),
            location: Location::Position {
                path: markdown.to_path_buf(),
                line: diagnostic.line,
                column: diagnostic.column,
            },
            hints: Vec::new(),
        }
    }

    pub fn is_error(&self) -> bool {
        self.severity == Severity::Error
    }

    /// Le texte affiché, sans saut de ligne final. Les chemins sont relatifs
    /// à `cwd` quand c'est possible.
    pub fn render(&self, cwd: &Path) -> String {
        let severity = match self.severity {
            Severity::Error => "erreur",
            Severity::Warning => "avertissement",
        };
        let location = match &self.location {
            Location::None => "combava".to_string(),
            Location::File(path) => display_path(path, cwd),
            Location::Position { path, line, column } => {
                format!("{}:{line}:{column}", display_path(path, cwd))
            }
        };
        let mut out = format!("{location}: {severity}[{}] : {}", self.code, self.message);
        for hint in &self.hints {
            out.push_str("\n  aide : ");
            out.push_str(hint);
        }
        out
    }
}

/// `path` relatif à `cwd` s'il s'y trouve, tel quel sinon.
pub fn display_path(path: &Path, cwd: &Path) -> String {
    path.strip_prefix(cwd).unwrap_or(path).display().to_string()
}

/// Ligne et colonne de l'octet `offset` de `text`, numérotées à partir de 1.
///
/// Une ligne se termine par `\n`, `\r\n` ou `\r` seul ; une colonne compte
/// des caractères. Un `offset` au milieu d'un caractère ou au-delà du texte
/// est ramené au caractère précédent.
pub fn position(text: &str, offset: usize) -> (usize, usize) {
    let mut offset = offset.min(text.len());
    while !text.is_char_boundary(offset) {
        offset -= 1;
    }
    let before = &text.as_bytes()[..offset];
    let mut line = 1;
    let mut line_start = 0;
    for (i, &byte) in before.iter().enumerate() {
        let ends_line =
            byte == b'\n' || (byte == b'\r' && text.as_bytes().get(i + 1) != Some(&b'\n'));
        if ends_line {
            line += 1;
            line_start = i + 1;
        }
    }
    let column = text[line_start..offset].chars().count() + 1;
    (line, column)
}

/// Retire les diagnostics identiques à un diagnostic précédent, sans changer
/// l'ordre des autres.
pub fn dedup(diagnostics: &mut Vec<Diagnostic>) {
    let mut kept: Vec<Diagnostic> = Vec::with_capacity(diagnostics.len());
    for diagnostic in diagnostics.drain(..) {
        if !kept.contains(&diagnostic) {
            kept.push(diagnostic);
        }
    }
    *diagnostics = kept;
}

/// Affiche les diagnostics sur la sortie d'erreur, au fur et à mesure.
pub struct Reporter {
    cwd: PathBuf,
    errors: usize,
}

impl Reporter {
    pub fn new(cwd: PathBuf) -> Self {
        Self { cwd, errors: 0 }
    }

    pub fn report(&mut self, diagnostic: &Diagnostic) {
        if diagnostic.is_error() {
            self.errors += 1;
        }
        // Une sortie d'erreur fermée ne doit pas faire paniquer le binaire.
        let _ = writeln!(std::io::stderr(), "{}", diagnostic.render(&self.cwd));
    }

    pub fn report_all<'a>(&mut self, diagnostics: impl IntoIterator<Item = &'a Diagnostic>) {
        for diagnostic in diagnostics {
            self.report(diagnostic);
        }
    }

    /// Affiche une erreur et arrête la commande.
    pub fn fail(&mut self, diagnostic: Diagnostic) -> Failed {
        self.report(&diagnostic);
        Failed
    }

    /// Affiche des diagnostics et arrête la commande s'ils contiennent une
    /// erreur.
    pub fn check(&mut self, diagnostics: &[Diagnostic]) -> Result<(), Failed> {
        self.report_all(diagnostics);
        if diagnostics.iter().any(Diagnostic::is_error) {
            Err(Failed)
        } else {
            Ok(())
        }
    }

    pub fn has_errors(&self) -> bool {
        self.errors > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions() {
        let text = "ab\ncé\r\nx\ry";
        assert_eq!(position(text, 0), (1, 1));
        assert_eq!(position(text, 2), (1, 3));
        assert_eq!(position(text, 3), (2, 1));
        // « é » fait deux octets mais une seule colonne.
        assert_eq!(position(text, 6), (2, 3));
        // Au milieu de `\r\n` : toujours sur la ligne 2.
        assert_eq!(position(text, 7), (2, 4));
        assert_eq!(position(text, 8), (3, 1));
        assert_eq!(position(text, 10), (4, 1));
        assert_eq!(position(text, 1000), (4, 2));
        // Au milieu de « é » : ramené au début du caractère.
        assert_eq!(position(text, 5), (2, 2));
    }

    #[test]
    fn doublons() {
        let a = Diagnostic::error(Code::Typst, "a");
        let b = Diagnostic::error(Code::Typst, "b");
        let a_ailleurs = a.clone().at(Location::File("x.md".into()));
        let mut diagnostics = vec![
            a.clone(),
            b.clone(),
            a.clone(),
            a_ailleurs.clone(),
            b.clone(),
        ];
        dedup(&mut diagnostics);
        assert_eq!(diagnostics, [a, b, a_ailleurs]);
    }

    #[test]
    fn rendu() {
        let cwd = Path::new("/projet");
        let d =
            Diagnostic::error(Code::UnknownKey, "clé inconnue « autors »").at(Location::Position {
                path: "/projet/rapport.md".into(),
                line: 3,
                column: 1,
            });
        assert_eq!(
            d.render(cwd),
            "rapport.md:3:1: erreur[unknown-key] : clé inconnue « autors »"
        );

        let d = Diagnostic::error(Code::Typst, "x")
            .at(Location::File("/ailleurs/a.md".into()))
            .with_hint("première")
            .with_hint("seconde");
        assert_eq!(
            d.render(cwd),
            "/ailleurs/a.md: erreur[typst] : x\n  aide : première\n  aide : seconde"
        );

        let mut d = Diagnostic::error(Code::Io, "y");
        d.severity = Severity::Warning;
        assert_eq!(d.render(cwd), "combava: avertissement[io] : y");
    }
}
