//! Correspondance entre les lignes du Typst généré et celles du markdown.

/// Correspondance ligne Typst → ligne markdown, pour traduire les erreurs de
/// compilation.
#[derive(Debug, Clone)]
pub struct SourceMap {
    /// Ligne markdown de chaque ligne Typst (indice 0 = ligne 1).
    lines: Vec<Option<usize>>,
}

impl SourceMap {
    /// Ligne markdown qui a produit la ligne `typst_line` du code généré.
    /// `None` pour le préambule, les lignes de séparation et la bibliographie.
    pub fn markdown_line(&self, typst_line: usize) -> Option<usize> {
        self.lines
            .get(typst_line.checked_sub(1)?)
            .copied()
            .flatten()
    }
}

/// Tampon d'écriture du Typst, qui note pour chaque ligne la ligne markdown
/// courante au moment où elle commence.
pub(crate) struct Writer {
    out: String,
    lines: Vec<Option<usize>>,
    tag: Option<usize>,
    at_line_start: bool,
    after_code: bool,
}

impl Writer {
    pub(crate) fn new() -> Self {
        Writer {
            out: String::new(),
            lines: Vec::new(),
            tag: None,
            at_line_start: true,
            after_code: false,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.out.is_empty()
    }

    pub(crate) fn tag(&self) -> Option<usize> {
        self.tag
    }

    /// Ligne markdown attribuée aux lignes Typst qui commencent ensuite.
    pub(crate) fn set_tag(&mut self, tag: Option<usize>) {
        self.tag = tag;
    }

    pub(crate) fn push(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }
        // En markup, `#f()` suivi de `(`, `[` ou `.` prolongerait l'expression :
        // le `;` la termine sans rien afficher.
        if std::mem::take(&mut self.after_code) && s.starts_with(['(', '[', '.']) {
            self.push_raw(";");
        }
        self.push_raw(s);
    }

    /// Signale que le texte écrit juste avant termine une expression de code.
    pub(crate) fn end_code(&mut self) {
        self.after_code = true;
    }

    fn push_raw(&mut self, s: &str) {
        for piece in s.split_inclusive('\n') {
            if self.at_line_start {
                self.lines.push(self.tag);
                self.at_line_start = false;
            }
            self.out.push_str(piece);
            self.at_line_start = piece.ends_with('\n');
        }
    }

    /// Ajoute le contenu d'un autre tampon, qui doit commencer une ligne.
    pub(crate) fn append(&mut self, other: Writer) {
        debug_assert!(self.at_line_start);
        self.out.push_str(&other.out);
        self.lines.extend(other.lines);
        self.at_line_start = other.at_line_start;
        self.after_code = other.after_code;
    }

    pub(crate) fn finish(self) -> (String, SourceMap) {
        (self.out, SourceMap { lines: self.lines })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chaque_ligne_recoit_l_etiquette_courante_a_son_debut() {
        let mut w = Writer::new();
        w.push("préambule\n");
        w.set_tag(Some(3));
        w.push("a");
        w.set_tag(Some(4));
        w.push("b\nc\n");
        w.set_tag(None);
        w.push("\n");
        let (out, map) = w.finish();
        assert_eq!(out, "préambule\nab\nc\n\n");
        assert_eq!(map.markdown_line(1), None);
        assert_eq!(map.markdown_line(2), Some(3));
        assert_eq!(map.markdown_line(3), Some(4));
        assert_eq!(map.markdown_line(4), None);
        assert_eq!(map.markdown_line(5), None);
        assert_eq!(map.markdown_line(0), None);
    }

    #[test]
    fn point_virgule_apres_une_expression_suivie_de_parenthese_crochet_ou_point() {
        for (next, expected) in [
            ("(x)", "#emph[a];(x)"),
            ("[", "#emph[a];["),
            (".", "#emph[a];."),
            (" x", "#emph[a] x"),
            ("\\[", "#emph[a]\\["),
        ] {
            let mut w = Writer::new();
            w.push("#emph[a]");
            w.end_code();
            w.push(next);
            assert_eq!(w.finish().0, expected);
        }
    }

    #[test]
    fn concatenation_de_tampons() {
        let mut a = Writer::new();
        a.push("x\n");
        let mut b = Writer::new();
        b.set_tag(Some(7));
        b.push("y\n");
        a.append(b);
        let (out, map) = a.finish();
        assert_eq!(out, "x\ny\n");
        assert_eq!(map.markdown_line(2), Some(7));
    }
}
