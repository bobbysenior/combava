//! Arbre du document, construit à partir des événements de pulldown-cmark.
//!
//! Chaque nœud porte sa plage d'octets dans le markdown, pour les diagnostics
//! et la correspondance des lignes.

use std::ops::Range;

pub(crate) struct Document {
    pub blocks: Vec<Block>,
    /// Définitions de notes, retirées du flux, dans l'ordre du fichier.
    pub footnotes: Vec<FootnoteDef>,
}

pub(crate) struct FootnoteDef {
    pub label: String,
    pub blocks: Vec<Block>,
    pub span: Range<usize>,
}

pub(crate) struct Block {
    pub kind: BlockKind,
    pub span: Range<usize>,
}

pub(crate) enum BlockKind {
    Paragraph(Vec<Inline>),
    /// Contenu en ligne sans paragraphe : élément de liste compacte, définition.
    Plain(Vec<Inline>),
    Heading {
        level: u8,
        id: Option<String>,
        content: Vec<Inline>,
    },
    /// Citation en bloc ; `Some` pour un callout GFM (`"note"`, `"tip"`…).
    Quote(Option<&'static str>, Vec<Block>),
    Code {
        /// Info string d'un bloc clôturé ; `None` pour un bloc indenté.
        info: Option<String>,
        text: String,
        /// Octet du début du contenu, pour les lignes des blocs ` ```{=typst} `.
        text_start: usize,
    },
    List {
        start: Option<u64>,
        items: Vec<ListItem>,
    },
    Definitions(Vec<Definition>),
    Table {
        aligns: Vec<Align>,
        header: Vec<Vec<Inline>>,
        rows: Vec<Vec<Vec<Inline>>>,
    },
    Rule,
}

pub(crate) struct ListItem {
    pub blocks: Vec<Block>,
    pub span: Range<usize>,
}

pub(crate) struct Definition {
    pub term: Vec<Inline>,
    pub details: Vec<Vec<Block>>,
}

#[derive(Clone, Copy)]
pub(crate) enum Align {
    Left,
    Center,
    Right,
}

pub(crate) struct Inline {
    pub kind: InlineKind,
    pub span: Range<usize>,
}

pub(crate) enum InlineKind {
    /// Texte, événements consécutifs fusionnés.
    Text(String),
    Code(String),
    SoftBreak,
    HardBreak,
    Emph(Vec<Inline>),
    Strong(Vec<Inline>),
    Strike(Vec<Inline>),
    /// Contenu affiché sans mise en forme (exposant, indice : options désactivées).
    Group(Vec<Inline>),
    Link {
        target: LinkTarget,
        content: Vec<Inline>,
    },
    Image {
        dest: String,
        title: String,
        alt: Vec<Inline>,
    },
    FootnoteRef(String),
    Math {
        display: bool,
        source: String,
    },
    Task(bool),
}

pub(crate) enum LinkTarget {
    Url(String),
    /// `#cible`, sans le `#`, non décodée.
    Anchor(String),
    /// Cible d'un wikilink `[[cible]]`.
    Wiki(String),
}

/// Texte sans mise en forme : texte alternatif des images, slugs des titres.
pub(crate) fn plain_text(inlines: &[Inline]) -> String {
    let mut out = String::new();
    push_plain_text(inlines, &mut out);
    out
}

fn push_plain_text(inlines: &[Inline], out: &mut String) {
    for inline in inlines {
        match &inline.kind {
            InlineKind::Text(text) | InlineKind::Code(text) => out.push_str(text),
            InlineKind::Math { source, .. } => out.push_str(source),
            InlineKind::SoftBreak | InlineKind::HardBreak => out.push(' '),
            InlineKind::Emph(children)
            | InlineKind::Strong(children)
            | InlineKind::Strike(children)
            | InlineKind::Group(children)
            | InlineKind::Link {
                content: children, ..
            }
            | InlineKind::Image { alt: children, .. } => push_plain_text(children, out),
            InlineKind::FootnoteRef(_) | InlineKind::Task(_) => {}
        }
    }
}
