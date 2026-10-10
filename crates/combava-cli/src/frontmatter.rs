//! Découpage du frontmatter `+++` (spécification, sections 2.2 et 6.1).

use std::ops::Range;
use std::path::Path;

use crate::diagnostics::{Diagnostic, Location};
use crate::error::Code;

const BOM: &str = "\u{feff}";
const DELIMITER: &str = "+++";

/// Un fichier markdown découpé.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    /// Le fichier complet, dans lequel chaque octet du frontmatter (BOM et
    /// délimiteurs compris) est remplacé par une espace, sauf les `\n` : les
    /// octets, lignes et colonnes restent ceux du fichier.
    pub markdown: String,
    /// Plage d'octets du TOML dans le fichier, entre les deux délimiteurs.
    pub frontmatter: Option<Range<usize>>,
}

/// Découpe `text`, le contenu du fichier `path`.
pub fn split(text: &str, path: &Path) -> Result<Document, Diagnostic> {
    let start = if text.starts_with(BOM) { BOM.len() } else { 0 };
    let mut lines = Lines {
        text,
        position: start,
    };
    let (blank_end, frontmatter) = match lines.next() {
        Some(first) if is_delimiter(&text[first.content.clone()]) => {
            let toml_start = first.next;
            let closing = lines.find(|line| is_delimiter(&text[line.content.clone()]));
            let Some(closing) = closing else {
                return Err(Diagnostic::error(
                    Code::UnclosedFrontmatter,
                    "frontmatter non fermé : il manque la ligne « +++ » de fin",
                )
                .at(Location::at(path, text, 0)));
            };
            (closing.next, Some(toml_start..closing.content.start))
        }
        // Un BOM seul est aussi effacé : il empêcherait de reconnaître un
        // titre sur la première ligne.
        _ => (start, None),
    };

    let mut bytes = text.as_bytes().to_vec();
    for byte in &mut bytes[..blank_end] {
        if *byte != b'\n' {
            *byte = b' ';
        }
    }
    // `blank_end` est une fin de ligne : le reste du texte est intact.
    let markdown = String::from_utf8(bytes).expect("seuls des caractères entiers sont remplacés");
    Ok(Document {
        markdown,
        frontmatter,
    })
}

/// `+++`, suivi éventuellement d'espaces et de tabulations.
fn is_delimiter(line: &str) -> bool {
    line.trim_end_matches([' ', '\t']) == DELIMITER
}

struct Line {
    /// Le contenu, sans la fin de ligne.
    content: Range<usize>,
    /// Le début de la ligne suivante.
    next: usize,
}

/// Lignes terminées par `\n` ou `\r\n`, la dernière pouvant ne pas l'être.
struct Lines<'a> {
    text: &'a str,
    position: usize,
}

impl Iterator for Lines<'_> {
    type Item = Line;

    fn next(&mut self) -> Option<Line> {
        if self.position >= self.text.len() {
            return None;
        }
        let start = self.position;
        let (content_end, next) = match self.text[start..].find('\n') {
            Some(i) => {
                let newline = start + i;
                let end = if self.text[start..newline].ends_with('\r') {
                    newline - 1
                } else {
                    newline
                };
                (end, newline + 1)
            }
            None => (self.text.len(), self.text.len()),
        };
        self.position = next;
        Some(Line {
            content: start..content_end,
            next,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split_ok(text: &str) -> Document {
        let document = split(text, Path::new("a.md")).unwrap();
        assert_eq!(document.markdown.len(), text.len());
        // Les `\n` sont tous à leur place : les numéros de ligne sont conservés.
        let newlines = |s: &str| s.match_indices('\n').map(|(i, _)| i).collect::<Vec<_>>();
        assert_eq!(newlines(&document.markdown), newlines(text));
        document
    }

    #[test]
    fn sans_frontmatter() {
        let document = split_ok("# Titre\n\n+++\n");
        assert_eq!(document.markdown, "# Titre\n\n+++\n");
        assert_eq!(document.frontmatter, None);
        // `++++` n'est pas un délimiteur.
        assert_eq!(split_ok("++++\na\n+++\n").frontmatter, None);
        assert_eq!(split_ok("").frontmatter, None);
    }

    #[test]
    fn frontmatter_efface() {
        let text = "+++\ntitle = \"Été\"\n+++\n\n# Titre\n";
        let document = split_ok(text);
        assert_eq!(
            &text[document.frontmatter.clone().unwrap()],
            "title = \"Été\"\n"
        );
        assert_eq!(
            document.markdown,
            format!(
                "   \n{}\n   \n\n# Titre\n",
                " ".repeat("title = \"Été\"".len())
            )
        );
    }

    #[test]
    fn bom_crlf_et_espaces_finaux() {
        let text = "\u{feff}+++ \t\r\na = 1\r\n+++\t\r\nTexte\r\n";
        let document = split_ok(text);
        assert_eq!(&text[document.frontmatter.clone().unwrap()], "a = 1\r\n");
        assert!(document.markdown.ends_with("\nTexte\r\n"));
        assert!(
            document.markdown[..text.len() - 7]
                .bytes()
                .all(|b| b == b' ' || b == b'\n')
        );
    }

    #[test]
    fn bom_seul() {
        let document = split_ok("\u{feff}# Titre\n");
        assert_eq!(document.markdown, "   # Titre\n");
        assert_eq!(document.frontmatter, None);
    }

    #[test]
    fn fermeture_en_fin_de_fichier() {
        let document = split_ok("+++\na = 1\n+++");
        assert_eq!(document.markdown, "   \n     \n   ");
        assert_eq!(document.frontmatter, Some(4..10));
        assert_eq!(split_ok("+++\n+++\n").frontmatter, Some(4..4));
    }

    #[test]
    fn non_ferme() {
        let error = split("+++\na = 1\n++++\n", Path::new("a.md")).unwrap_err();
        assert_eq!(error.code, "unclosed-frontmatter");
        assert_eq!(
            error.location,
            Location::Position {
                path: "a.md".into(),
                line: 1,
                column: 1
            }
        );
        assert!(split("+++", Path::new("a.md")).is_err());
    }
}
