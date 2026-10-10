//! Collecte des diagnostics et conversion des octets en lignes et colonnes.

use std::ops::Range;

use crate::output::{Code, Diagnostic};

/// Diagnostics accumulés pendant la transpilation.
pub(crate) struct Diagnostics<'a> {
    source: &'a str,
    /// Octet de début de chaque ligne.
    line_starts: Vec<usize>,
    list: Vec<Diagnostic>,
}

impl<'a> Diagnostics<'a> {
    pub(crate) fn new(source: &'a str) -> Self {
        // Comme en CommonMark, une ligne se termine par `\n`, `\r\n` ou `\r` seul.
        let bytes = source.as_bytes();
        let line_starts = std::iter::once(0)
            .chain(bytes.iter().enumerate().filter_map(|(i, &b)| {
                let ends_line = b == b'\n' || (b == b'\r' && bytes.get(i + 1) != Some(&b'\n'));
                ends_line.then_some(i + 1)
            }))
            .collect();
        Diagnostics {
            source,
            line_starts,
            list: Vec::new(),
        }
    }

    /// Ligne de l'octet `offset`, à partir de 1.
    pub(crate) fn line(&self, offset: usize) -> usize {
        self.line_starts.partition_point(|&start| start <= offset)
    }

    /// Colonne de l'octet `offset` en caractères, à partir de 1.
    pub(crate) fn column(&self, offset: usize) -> usize {
        let mut offset = offset.min(self.source.len());
        while !self.source.is_char_boundary(offset) {
            offset -= 1;
        }
        let start = self.line_starts[self.line(offset) - 1];
        self.source[start..offset].chars().count() + 1
    }

    pub(crate) fn push(&mut self, code: Code, span: Range<usize>, message: impl Into<String>) {
        self.list.push(Diagnostic {
            severity: code.severity(),
            code,
            message: message.into(),
            line: self.line(span.start),
            column: self.column(span.start),
            span,
        });
    }

    /// Les diagnostics triés par position, dans l'ordre d'émission à position égale.
    pub(crate) fn into_sorted(mut self) -> Vec<Diagnostic> {
        self.list.sort_by_key(|d| d.span.start);
        self.list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lignes_et_colonnes_a_partir_de_1() {
        let diags = Diagnostics::new("ab\ncdé\n\nf");
        assert_eq!((diags.line(0), diags.column(0)), (1, 1));
        assert_eq!((diags.line(1), diags.column(1)), (1, 2));
        assert_eq!((diags.line(3), diags.column(3)), (2, 1));
        // « é » occupe deux octets : l'octet 7 est le saut de ligne qui le suit.
        assert_eq!((diags.line(7), diags.column(7)), (2, 4));
        assert_eq!((diags.line(8), diags.column(8)), (3, 1));
        assert_eq!((diags.line(9), diags.column(9)), (4, 1));
    }

    #[test]
    fn fins_de_ligne_crlf_et_cr() {
        let diags = Diagnostics::new("a\r\nb\rc");
        assert_eq!((diags.line(1), diags.column(1)), (1, 2));
        assert_eq!((diags.line(3), diags.column(3)), (2, 1));
        assert_eq!((diags.line(5), diags.column(5)), (3, 1));
    }

    #[test]
    fn colonne_au_milieu_d_un_caractere() {
        let diags = Diagnostics::new("é");
        assert_eq!(diags.column(1), 1);
        assert_eq!(diags.column(100), 2);
    }

    #[test]
    fn tri_par_position() {
        let mut diags = Diagnostics::new("abc\ndef");
        diags.push(Code::RawHtml, 4..5, "b");
        diags.push(Code::BrokenLink, 0..1, "a");
        let sorted = diags.into_sorted();
        assert_eq!(sorted[0].message, "a");
        assert_eq!(sorted[1].line, 2);
        assert_eq!(sorted[1].severity, Code::RawHtml.severity());
    }
}
