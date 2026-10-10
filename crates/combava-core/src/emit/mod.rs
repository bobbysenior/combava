//! Émission du code Typst à partir de l'arbre du document.

mod block;
mod code;
mod inline;
mod preamble;

use std::collections::HashMap;

use crate::ast::{Document, FootnoteDef};
use crate::config::Config;
use crate::diag::Diagnostics;
use crate::escape;
use crate::label::Labels;
use crate::output::Code;
use crate::paths;
use crate::source_map::{SourceMap, Writer};

/// Produit le code Typst complet (spécification, section 2.3) et sa correspondance de lignes.
pub(crate) fn document(
    document: &Document,
    config: &Config,
    source: &str,
    diags: &mut Diagnostics,
) -> (String, SourceMap) {
    let labels = Labels::collect(document, diags);
    let mut emitter = Emitter::new(document, config, source, diags, labels);
    emitter.blocks(&document.blocks, true);
    if !emitter.w.is_empty() {
        emitter.w.push("\n");
    }
    emitter.report_unused_footnotes();

    let uses_math = emitter.uses_math;
    let body = emitter.w;

    let mut out = Writer::new();
    out.push(&preamble::preamble(config, uses_math));
    if !body.is_empty() {
        out.push("\n");
        out.append(body);
    }
    if config.bibliography {
        out.set_tag(None);
        out.push("\n");
        out.push(&format!(
            "#bibliography({})\n",
            escape::string(paths::BIBLIOGRAPHY)
        ));
    }
    out.finish()
}

/// Préfixe réservé des labels de notes de bas de page.
const FOOTNOTE_LABEL_PREFIX: &str = "combava-fn-";

#[derive(Clone, Copy, PartialEq, Eq)]
enum FootnoteState {
    Unused,
    InProgress,
    /// Émise seulement dans des titres, sans label.
    InHeading,
    Emitted,
}

pub(crate) struct Emitter<'a, 'd, 's> {
    w: Writer,
    source: &'a str,
    config: &'a Config,
    diags: &'d mut Diagnostics<'s>,
    labels: Labels,
    footnote_defs: &'a [FootnoteDef],
    /// Clé normalisée → indice de la première définition.
    footnote_index: HashMap<String, usize>,
    footnote_states: Vec<FootnoteState>,
    /// Typst recopie les titres dans la table des matières : un label y serait en double.
    in_heading: bool,
    uses_math: bool,
}

impl<'a, 'd, 's> Emitter<'a, 'd, 's> {
    fn new(
        document: &'a Document,
        config: &'a Config,
        source: &'a str,
        diags: &'d mut Diagnostics<'s>,
        labels: Labels,
    ) -> Self {
        let mut footnote_index = HashMap::new();
        for (i, footnote) in document.footnotes.iter().enumerate() {
            footnote_index
                .entry(footnote_key(&footnote.label))
                .or_insert(i);
        }
        Emitter {
            w: Writer::new(),
            source,
            config,
            diags,
            labels,
            footnote_defs: &document.footnotes,
            footnote_index,
            footnote_states: vec![FootnoteState::Unused; document.footnotes.len()],
            in_heading: false,
            uses_math: false,
        }
    }

    /// Ligne markdown de l'octet `offset`, pour la correspondance des lignes.
    fn line(&self, offset: usize) -> usize {
        self.diags.line(offset)
    }

    fn report_unused_footnotes(&mut self) {
        for (footnote, state) in self.footnote_defs.iter().zip(&self.footnote_states) {
            if *state == FootnoteState::Unused {
                self.diags.push(
                    Code::UnusedFootnote,
                    footnote.span.clone(),
                    format!("note de bas de page « {} » jamais appelée", footnote.label),
                );
            }
        }
    }
}

/// pulldown-cmark associe appels et définitions sans tenir compte de la casse.
fn footnote_key(label: &str) -> String {
    label.to_lowercase()
}
