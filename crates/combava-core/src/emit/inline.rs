//! Émission des éléments en ligne (spécification, sections 4.1, 4.3, 4.4,
//! 4.7, 4.8 et 4.9).

use std::ops::Range;

use super::{Emitter, FOOTNOTE_LABEL_PREFIX, FootnoteState, footnote_key};
use crate::ast::{Inline, InlineKind, LinkTarget, plain_text};
use crate::escape::{markup, percent_decode, string};
use crate::output::Code;
use crate::path::{ImagePathError, resolve_image};

impl Emitter<'_, '_, '_> {
    pub(super) fn inlines(&mut self, inlines: &[Inline]) {
        for inline in inlines {
            self.inline(inline);
        }
    }

    fn inline(&mut self, inline: &Inline) {
        match &inline.kind {
            InlineKind::Text(text) => self.text(text, &inline.span),
            InlineKind::Code(code) => self.code(&format!("#raw({})", string(code))),
            InlineKind::SoftBreak => self.w.push(" "),
            InlineKind::HardBreak => self.code("#linebreak()"),
            InlineKind::Emph(content) => self.wrapped("#emph[", content),
            InlineKind::Strong(content) => self.wrapped("#strong[", content),
            InlineKind::Strike(content) => self.wrapped("#strike[", content),
            InlineKind::Group(content) => self.inlines(content),
            InlineKind::Link { target, content } => self.link(target, content, &inline.span),
            InlineKind::Image { dest, alt, .. } => {
                if let Some(call) = self.image_call(dest, alt, &inline.span, Some("1em")) {
                    self.code(&format!("#box({call})"));
                }
            }
            InlineKind::FootnoteRef(label) => self.footnote(label, &inline.span),
            InlineKind::Math { display, source } => {
                self.uses_math = true;
                let function = if *display { "mitex" } else { "mi" };
                self.code(&format!("#{function}({})", string(source)));
            }
            InlineKind::Task(checked) => self.w.push(if *checked { "☒ " } else { "☐ " }),
        }
    }

    /// Écrit une expression de code complète.
    fn code(&mut self, expression: &str) {
        self.w.push(expression);
        self.w.end_code();
    }

    fn wrapped(&mut self, open: &str, content: &[Inline]) {
        self.w.push(open);
        self.inlines(content);
        self.w.push("]");
        self.w.end_code();
    }

    /// Texte échappé, avec reconnaissance des citations `[@cle]` et des appels
    /// de note sans définition `[^x]`.
    fn text(&mut self, text: &str, span: &Range<usize>) {
        let mut plain_start = 0;
        let mut search = 0;
        while let Some(relative) = text[search..].find('[') {
            let at = search + relative;
            if let Some((keys, len)) = citation(&text[at..]) {
                self.w.push(&markup(&text[plain_start..at]));
                let written = &text[at..at + len];
                if !self.config.bibliography {
                    let location = self.locate(span, written);
                    self.diags.push(
                        Code::CitationWithoutBibliography,
                        location,
                        format!("citation « {written} » sans bibliographie : définir la clé « bibliography »"),
                    );
                }
                for key in keys {
                    self.code(&format!("#cite(label({}))", string(key)));
                }
                search = at + len;
                plain_start = search;
            } else if let Some(len) = footnote_call(&text[at..]) {
                let written = &text[at..at + len];
                let location = self.locate(span, written);
                self.diags.push(
                    Code::UndefinedFootnote,
                    location,
                    format!(
                        "note de bas de page « {} » non définie",
                        &written[2..len - 1]
                    ),
                );
                search = at + len;
            } else {
                search = at + 1;
            }
        }
        self.w.push(&markup(&text[plain_start..]));
    }

    /// Position exacte de `needle` dans le source, ou à défaut tout le texte.
    fn locate(&self, span: &Range<usize>, needle: &str) -> Range<usize> {
        self.source
            .get(span.clone())
            .and_then(|source| source.find(needle))
            .map(|p| span.start + p..span.start + p + needle.len())
            .unwrap_or_else(|| span.clone())
    }

    fn link(&mut self, target: &LinkTarget, content: &[Inline], span: &Range<usize>) {
        let destination = match target {
            LinkTarget::Url(url) if url.is_empty() => None,
            LinkTarget::Url(url) => Some(string(url)),
            LinkTarget::Anchor(anchor) => self.label_ref(&percent_decode(anchor), anchor, span),
            LinkTarget::Wiki(target) => {
                let label = self.labels.wiki_target(target);
                self.label_ref(&label, target, span)
            }
        };
        match destination {
            Some(destination) => self.wrapped(&format!("#link({destination})["), content),
            None => self.inlines(content),
        }
    }

    fn label_ref(&mut self, label: &str, written: &str, span: &Range<usize>) -> Option<String> {
        if self.labels.contains(label) {
            Some(format!("label({})", string(label)))
        } else {
            self.diags.push(
                Code::BrokenLink,
                span.clone(),
                format!("lien vers un label inexistant « {written} »"),
            );
            None
        }
    }

    /// Appel `image(…)` ; `None` après un diagnostic si le chemin est refusé.
    pub(super) fn image_call(
        &mut self,
        dest: &str,
        alt: &[Inline],
        span: &Range<usize>,
        height: Option<&str>,
    ) -> Option<String> {
        let path = match resolve_image(dest) {
            Ok(path) => path,
            Err(ImagePathError::Remote) => {
                self.diags.push(
                    Code::RemoteImage,
                    span.clone(),
                    format!("image distante non prise en charge « {dest} »"),
                );
                return None;
            }
            Err(ImagePathError::Invalid) => {
                self.diags.push(
                    Code::InvalidImagePath,
                    span.clone(),
                    format!("chemin d'image invalide « {dest} » : il doit être relatif et rester dans le dossier du document"),
                );
                return None;
            }
        };
        let mut call = format!("image({}", string(&path));
        let alt = plain_text(alt);
        if !alt.trim().is_empty() {
            call.push_str(&format!(", alt: {}", string(alt.trim())));
        }
        if let Some(height) = height {
            call.push_str(&format!(", height: {height}"));
        }
        call.push(')');
        Some(call)
    }

    fn footnote(&mut self, label: &str, span: &Range<usize>) {
        let key = footnote_key(label);
        let Some(&index) = self.footnote_index.get(&key) else {
            self.diags.push(
                Code::UndefinedFootnote,
                span.clone(),
                format!("note de bas de page « {label} » non définie"),
            );
            self.w.push(&markup(&format!("[^{label}]")));
            return;
        };
        let typst_label = string(&format!("{FOOTNOTE_LABEL_PREFIX}{key}"));
        match self.footnote_states[index] {
            FootnoteState::Emitted => self.code(&format!("#footnote(label({typst_label}))")),
            // Une note qui s'appelle elle-même : l'appel interne reste du texte.
            FootnoteState::InProgress => self.w.push(&markup(&format!("[^{label}]"))),
            // Dans un titre, la note est émise sans label, quitte à être
            // dupliquée par un appel ultérieur hors titre.
            FootnoteState::Unused | FootnoteState::InHeading if self.in_heading => {
                self.footnote_body(index, None);
                self.footnote_states[index] = FootnoteState::InHeading;
            }
            FootnoteState::Unused | FootnoteState::InHeading => {
                self.footnote_body(index, Some(&typst_label));
                self.footnote_states[index] = FootnoteState::Emitted;
            }
        }
    }

    /// `#footnote[…]`, suivi de `#label(…)` si `label` est donné.
    fn footnote_body(&mut self, index: usize, label: Option<&str>) {
        self.footnote_states[index] = FootnoteState::InProgress;
        let tag = self.w.tag();
        let definitions = self.footnote_defs;
        self.w.push("#footnote[");
        self.blocks(&definitions[index].blocks, false);
        self.w.push("]");
        if let Some(label) = label {
            self.w.push(&format!("#label({label})"));
        }
        self.w.end_code();
        self.w.set_tag(tag);
    }
}

fn is_citation_key_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | ':')
}

/// Reconnaît `[@a]` ou `[@a; @b]` au début de `s` : clés et longueur.
fn citation(s: &str) -> Option<(Vec<&str>, usize)> {
    let mut pos = s.strip_prefix('[').map(|_| 1)?;
    let mut keys = Vec::new();
    loop {
        pos += s[pos..].strip_prefix('@').map(|_| 1)?;
        let key_len = s[pos..]
            .find(|c: char| !is_citation_key_char(c))
            .unwrap_or(s.len() - pos);
        if key_len == 0 {
            return None;
        }
        keys.push(&s[pos..pos + key_len]);
        pos += key_len;
        pos = s.len() - s[pos..].trim_start().len();
        match s[pos..].chars().next()? {
            ']' => return Some((keys, pos + 1)),
            ';' => pos = s.len() - s[pos + 1..].trim_start().len(),
            _ => return None,
        }
    }
}

/// Reconnaît un appel de note `[^x]` au début de `s` : longueur.
fn footnote_call(s: &str) -> Option<usize> {
    let rest = s.strip_prefix("[^")?;
    let end = rest.find(|c: char| c == ']' || c == '[' || c.is_whitespace())?;
    (end > 0 && rest[end..].starts_with(']')).then_some(2 + end + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn citations_reconnues() {
        assert_eq!(citation("[@a]"), Some((vec!["a"], 4)));
        assert_eq!(citation("[@a] suite"), Some((vec!["a"], 4)));
        assert_eq!(citation("[@a; @b]"), Some((vec!["a", "b"], 8)));
        assert_eq!(citation("[@a ;@b ]"), Some((vec!["a", "b"], 9)));
        assert_eq!(
            citation("[@knuth:1984; @doe-2020.v2]"),
            Some((vec!["knuth:1984", "doe-2020.v2"], 27))
        );
        assert_eq!(citation("[@été]"), Some((vec!["été"], 8)));
    }

    #[test]
    fn citations_refusees() {
        for s in [
            "[a]", "[@]", "[@a", "[@a b]", "[@a; b]", "[@a;]", "[ @a]", "@a", "[@a/b]",
        ] {
            assert_eq!(citation(s), None, "{s}");
        }
    }

    #[test]
    fn appels_de_note() {
        assert_eq!(footnote_call("[^x] suite"), Some(4));
        assert_eq!(footnote_call("[^note-1]"), Some(9));
        assert_eq!(footnote_call("[^]"), None);
        assert_eq!(footnote_call("[^a b]"), None);
        assert_eq!(footnote_call("[^a"), None);
        assert_eq!(footnote_call("[x]"), None);
    }
}
