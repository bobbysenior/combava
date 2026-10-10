//! Parsing du markdown (pulldown-cmark) et construction de l'arbre du document.

use std::iter::Peekable;
use std::ops::Range;

use pulldown_cmark::{
    Alignment, BlockQuoteKind, CodeBlockKind, Event, HeadingLevel, LinkType, OffsetIter, Options,
    Parser, Tag,
};

use crate::ast::{
    Align, Block, BlockKind, Definition, Document, FootnoteDef, Inline, InlineKind, LinkTarget,
    ListItem,
};
use crate::diag::Diagnostics;
use crate::output::Code;

/// Profondeur d'imbrication au-delà de laquelle le contenu est aplati en texte.
///
/// Garantit que ni la construction ni l'émission ne débordent la pile, quel
/// que soit le document reçu.
const MAX_DEPTH: usize = 32;

/// Options de la section 3 de la spécification, activées une par une.
pub(crate) fn options() -> Options {
    Options::ENABLE_TABLES
        | Options::ENABLE_FOOTNOTES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_HEADING_ATTRIBUTES
        | Options::ENABLE_DEFINITION_LIST
        | Options::ENABLE_GFM
        | Options::ENABLE_MATH
        | Options::ENABLE_WIKILINKS
}

pub(crate) fn parse(markdown: &str, diags: &mut Diagnostics) -> Document {
    let mut builder = Builder {
        events: Parser::new_ext(markdown, options())
            .into_offset_iter()
            .peekable(),
        footnotes: Vec::new(),
        diags,
        depth: 0,
    };
    let blocks = builder.blocks();
    Document {
        blocks,
        footnotes: builder.footnotes,
    }
}

struct Builder<'a, 'd, 's> {
    events: Peekable<OffsetIter<'a>>,
    footnotes: Vec<FootnoteDef>,
    diags: &'d mut Diagnostics<'s>,
    depth: usize,
}

impl Builder<'_, '_, '_> {
    /// Blocs jusqu'à la fin du conteneur courant (consommée) ou du document.
    fn blocks(&mut self) -> Vec<Block> {
        let mut blocks = Vec::new();
        while let Some((event, _)) = self.events.peek() {
            match event {
                Event::End(_) => {
                    self.events.next();
                    break;
                }
                Event::Start(tag) if !is_inline_tag(tag) => blocks.extend(self.block()),
                Event::Rule => {
                    if let Some((_, span)) = self.events.next() {
                        blocks.push(Block {
                            kind: BlockKind::Rule,
                            span,
                        });
                    }
                }
                Event::Html(_) => {
                    self.events.next();
                }
                _ => blocks.extend(self.plain()),
            }
        }
        blocks
    }

    /// Suite d'éléments en ligne hors paragraphe (liste compacte, définition).
    fn plain(&mut self) -> Option<Block> {
        let mut inlines = Vec::new();
        while let Some((event, _)) = self.events.peek() {
            let inline_level = match event {
                Event::Start(tag) => is_inline_tag(tag),
                Event::End(_) | Event::Rule | Event::Html(_) => false,
                _ => true,
            };
            if !inline_level {
                break;
            }
            inlines.extend(self.inline());
        }
        let span = inlines.first()?.span.start..inlines.last()?.span.end;
        Some(Block {
            kind: BlockKind::Plain(inlines),
            span,
        })
    }

    fn block(&mut self) -> Option<Block> {
        let (Event::Start(tag), span) = self.events.next()? else {
            return None;
        };
        let container = matches!(
            tag,
            Tag::BlockQuote(_)
                | Tag::List(_)
                | Tag::Item
                | Tag::FootnoteDefinition(_)
                | Tag::DefinitionList
                | Tag::Table(_)
        );
        if container && self.depth >= MAX_DEPTH {
            return Some(self.flattened(span));
        }

        let kind = match tag {
            Tag::Paragraph => BlockKind::Paragraph(self.inlines()),
            Tag::Heading { level, id, .. } => BlockKind::Heading {
                level: heading_level(level),
                id: id.map(|id| id.to_string()),
                content: self.inlines(),
            },
            Tag::BlockQuote(kind) => {
                BlockKind::Quote(kind.map(callout_kind), self.nested(Self::blocks))
            }
            Tag::CodeBlock(kind) => {
                let (text, text_start) = self.code_text(span.start);
                BlockKind::Code {
                    info: match kind {
                        CodeBlockKind::Fenced(info) => Some(info.to_string()),
                        CodeBlockKind::Indented => None,
                    },
                    text,
                    text_start,
                }
            }
            Tag::HtmlBlock => {
                let html = self.html_text();
                if !is_comment(&html) {
                    self.diags.push(Code::RawHtml, span, "HTML brut ignoré");
                }
                return None;
            }
            Tag::List(start) => BlockKind::List {
                start,
                items: self.nested(Self::items),
            },
            // Un élément hors liste n'arrive pas en pratique : traité comme une liste d'un élément.
            Tag::Item => BlockKind::List {
                start: None,
                items: vec![ListItem {
                    blocks: self.nested(Self::blocks),
                    span: span.clone(),
                }],
            },
            Tag::FootnoteDefinition(label) => {
                let blocks = self.nested(Self::blocks);
                self.footnotes.push(FootnoteDef {
                    label: label.to_string(),
                    blocks,
                    span,
                });
                return None;
            }
            Tag::DefinitionList => BlockKind::Definitions(self.nested(Self::definitions)),
            Tag::Table(aligns) => {
                let (header, rows) = self.nested(Self::table);
                BlockKind::Table {
                    aligns: aligns.into_iter().map(align).collect(),
                    header,
                    rows,
                }
            }
            _ => {
                self.skip();
                return None;
            }
        };
        Some(Block { kind, span })
    }

    fn items(&mut self) -> Vec<ListItem> {
        let mut items = Vec::new();
        while let Some((event, span)) = self.events.next() {
            match event {
                Event::Start(Tag::Item) => items.push(ListItem {
                    blocks: self.blocks(),
                    span,
                }),
                Event::Start(_) => {
                    self.skip();
                }
                Event::End(_) => break,
                _ => {}
            }
        }
        items
    }

    fn definitions(&mut self) -> Vec<Definition> {
        let mut definitions: Vec<Definition> = Vec::new();
        while let Some((event, _)) = self.events.next() {
            match event {
                Event::Start(Tag::DefinitionListTitle) => definitions.push(Definition {
                    term: self.inlines(),
                    details: Vec::new(),
                }),
                Event::Start(Tag::DefinitionListDefinition) => {
                    let blocks = self.blocks();
                    match definitions.last_mut() {
                        Some(definition) => definition.details.push(blocks),
                        None => definitions.push(Definition {
                            term: Vec::new(),
                            details: vec![blocks],
                        }),
                    }
                }
                Event::Start(_) => {
                    self.skip();
                }
                Event::End(_) => break,
                _ => {}
            }
        }
        definitions
    }

    #[allow(clippy::type_complexity)]
    fn table(&mut self) -> (Vec<Vec<Inline>>, Vec<Vec<Vec<Inline>>>) {
        let mut header = Vec::new();
        let mut rows = Vec::new();
        while let Some((event, _)) = self.events.next() {
            match event {
                Event::Start(Tag::TableHead) => header = self.cells(),
                Event::Start(Tag::TableRow) => rows.push(self.cells()),
                Event::Start(_) => {
                    self.skip();
                }
                Event::End(_) => break,
                _ => {}
            }
        }
        (header, rows)
    }

    fn cells(&mut self) -> Vec<Vec<Inline>> {
        let mut cells = Vec::new();
        while let Some((event, _)) = self.events.next() {
            match event {
                Event::Start(Tag::TableCell) => cells.push(self.inlines()),
                Event::Start(_) => {
                    self.skip();
                }
                Event::End(_) => break,
                _ => {}
            }
        }
        cells
    }

    /// Éléments en ligne jusqu'à la fin du conteneur courant (consommée).
    fn inlines(&mut self) -> Vec<Inline> {
        let mut inlines = Vec::new();
        while let Some((event, _)) = self.events.peek() {
            if matches!(event, Event::End(_)) {
                self.events.next();
                break;
            }
            inlines.extend(self.inline());
        }
        inlines
    }

    /// Un élément en ligne ; `None` s'il est ignoré (HTML). Consomme toujours
    /// au moins un événement.
    fn inline(&mut self) -> Option<Inline> {
        let (event, span) = self.events.next()?;
        let kind = match event {
            Event::Text(text) => {
                let mut text = text.into_string();
                let mut end = span.end;
                while let Some((Event::Text(_), _)) = self.events.peek() {
                    if let Some((Event::Text(more), more_span)) = self.events.next() {
                        text.push_str(&more);
                        end = more_span.end;
                    }
                }
                return Some(Inline {
                    kind: InlineKind::Text(text),
                    span: span.start..end,
                });
            }
            Event::Code(code) => InlineKind::Code(code.into_string()),
            Event::InlineMath(source) => InlineKind::Math {
                display: false,
                source: source.into_string(),
            },
            Event::DisplayMath(source) => InlineKind::Math {
                display: true,
                source: source.into_string(),
            },
            Event::Html(html) | Event::InlineHtml(html) => {
                if !is_comment(&html) {
                    self.diags.push(Code::RawHtml, span, "HTML brut ignoré");
                }
                return None;
            }
            Event::FootnoteReference(label) => InlineKind::FootnoteRef(label.into_string()),
            Event::SoftBreak => InlineKind::SoftBreak,
            Event::HardBreak => InlineKind::HardBreak,
            Event::TaskListMarker(checked) => InlineKind::Task(checked),
            Event::Rule | Event::End(_) => return None,
            Event::Start(_) if self.depth >= MAX_DEPTH => InlineKind::Text(self.skip()),
            Event::Start(tag) => match tag {
                Tag::Emphasis => InlineKind::Emph(self.nested(Self::inlines)),
                Tag::Strong => InlineKind::Strong(self.nested(Self::inlines)),
                Tag::Strikethrough => InlineKind::Strike(self.nested(Self::inlines)),
                Tag::Superscript | Tag::Subscript => InlineKind::Group(self.nested(Self::inlines)),
                Tag::Link {
                    link_type,
                    dest_url,
                    ..
                } => {
                    let content = self.nested(Self::inlines);
                    let dest = dest_url.into_string();
                    let target = match link_type {
                        LinkType::WikiLink { .. } => LinkTarget::Wiki(dest),
                        LinkType::Email => LinkTarget::Url(format!("mailto:{dest}")),
                        _ => match dest.strip_prefix('#') {
                            Some(anchor) => LinkTarget::Anchor(anchor.to_string()),
                            None => LinkTarget::Url(dest),
                        },
                    };
                    InlineKind::Link { target, content }
                }
                Tag::Image {
                    dest_url, title, ..
                } => InlineKind::Image {
                    dest: dest_url.into_string(),
                    title: title.into_string(),
                    alt: self.nested(Self::inlines),
                },
                _ => {
                    self.skip();
                    return None;
                }
            },
        };
        Some(Inline { kind, span })
    }

    fn code_text(&mut self, fallback: usize) -> (String, usize) {
        let mut text = String::new();
        let mut start = None;
        for (event, span) in self.events.by_ref() {
            match event {
                Event::Text(more) => {
                    start.get_or_insert(span.start);
                    text.push_str(&more);
                }
                Event::End(_) => break,
                _ => {}
            }
        }
        (text, start.unwrap_or(fallback))
    }

    fn html_text(&mut self) -> String {
        let mut html = String::new();
        for (event, _) in self.events.by_ref() {
            match event {
                Event::Html(more) => html.push_str(&more),
                Event::End(_) => break,
                _ => {}
            }
        }
        html
    }

    fn nested<T>(&mut self, f: impl FnOnce(&mut Self) -> T) -> T {
        self.depth += 1;
        let result = f(self);
        self.depth -= 1;
        result
    }

    /// Conteneur trop profond : son texte devient un paragraphe.
    fn flattened(&mut self, span: Range<usize>) -> Block {
        let text = self.skip();
        Block {
            kind: BlockKind::Paragraph(vec![Inline {
                kind: InlineKind::Text(text),
                span: span.clone(),
            }]),
            span,
        }
    }

    /// Consomme le reste d'un élément dont le début est déjà lu, sans
    /// récursion, et renvoie son texte.
    fn skip(&mut self) -> String {
        let mut text = String::new();
        let mut depth = 1usize;
        for (event, _) in self.events.by_ref() {
            match event {
                Event::Start(_) => depth += 1,
                Event::End(_) => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                Event::Text(more)
                | Event::Code(more)
                | Event::InlineMath(more)
                | Event::DisplayMath(more) => text.push_str(&more),
                Event::SoftBreak | Event::HardBreak => text.push(' '),
                _ => {}
            }
        }
        text
    }
}

fn is_inline_tag(tag: &Tag) -> bool {
    matches!(
        tag,
        Tag::Emphasis
            | Tag::Strong
            | Tag::Strikethrough
            | Tag::Superscript
            | Tag::Subscript
            | Tag::Link { .. }
            | Tag::Image { .. }
    )
}

fn is_comment(html: &str) -> bool {
    html.trim_start().starts_with("<!--")
}

fn heading_level(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

fn callout_kind(kind: BlockQuoteKind) -> &'static str {
    match kind {
        BlockQuoteKind::Note => "note",
        BlockQuoteKind::Tip => "tip",
        BlockQuoteKind::Important => "important",
        BlockQuoteKind::Warning => "warning",
        BlockQuoteKind::Caution => "caution",
    }
}

fn align(alignment: Alignment) -> Align {
    match alignment {
        Alignment::None | Alignment::Left => Align::Left,
        Alignment::Center => Align::Center,
        Alignment::Right => Align::Right,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_str(markdown: &str) -> (Document, Vec<crate::output::Diagnostic>) {
        let mut diags = Diagnostics::new(markdown);
        let document = parse(markdown, &mut diags);
        (document, diags.into_sorted())
    }

    #[test]
    fn options_de_la_specification() {
        let options = options();
        for disabled in [
            Options::ENABLE_YAML_STYLE_METADATA_BLOCKS,
            Options::ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS,
            Options::ENABLE_SMART_PUNCTUATION,
            Options::ENABLE_SUBSCRIPT,
            Options::ENABLE_SUPERSCRIPT,
        ] {
            assert!(!options.contains(disabled));
        }
        // ENABLE_OLD_FOOTNOTES inclut le bit de ENABLE_FOOTNOTES : on teste le sien seul.
        assert!(!options.contains(Options::ENABLE_OLD_FOOTNOTES));
        assert!(options.contains(Options::ENABLE_FOOTNOTES));
    }

    #[test]
    fn texte_decoupe_fusionne() {
        let (document, _) = parse_str("voir [@a; @b] ici");
        let BlockKind::Paragraph(inlines) = &document.blocks[0].kind else {
            panic!("paragraphe attendu");
        };
        assert_eq!(inlines.len(), 1);
        let InlineKind::Text(text) = &inlines[0].kind else {
            panic!("texte attendu");
        };
        assert_eq!(text, "voir [@a; @b] ici");
        assert_eq!(inlines[0].span, 0..17);
    }

    #[test]
    fn liste_compacte_en_plain_liste_aeree_en_paragraphes() {
        let (document, _) = parse_str("- a\n- b\n\n1. x\n\n   y\n");
        let BlockKind::List { items, .. } = &document.blocks[0].kind else {
            panic!("liste attendue");
        };
        assert!(matches!(items[0].blocks[0].kind, BlockKind::Plain(_)));
        let BlockKind::List { start, items } = &document.blocks[1].kind else {
            panic!("liste attendue");
        };
        assert_eq!(*start, Some(1));
        assert_eq!(items[0].blocks.len(), 2);
        assert!(matches!(items[0].blocks[0].kind, BlockKind::Paragraph(_)));
    }

    #[test]
    fn notes_retirees_du_flux() {
        let (document, _) = parse_str("a[^n]\n\n[^n]: texte\n");
        assert_eq!(document.blocks.len(), 1);
        assert_eq!(document.footnotes.len(), 1);
        assert_eq!(document.footnotes[0].label, "n");
    }

    #[test]
    fn html_signale_sauf_commentaires() {
        let (document, diags) = parse_str("<div>x</div>\n\n<!-- c -->\n\na <b>b</b> <!-- d -->\n");
        assert_eq!(document.blocks.len(), 1);
        let codes: Vec<_> = diags.iter().map(|d| (d.code, d.line)).collect();
        assert_eq!(
            codes,
            [(Code::RawHtml, 1), (Code::RawHtml, 5), (Code::RawHtml, 5)]
        );
    }

    #[test]
    fn imbrication_profonde_aplatie() {
        let markdown = ">".repeat(10_000) + " fond";
        let (document, _) = parse_str(&markdown);
        let mut depth = 0;
        let mut blocks = &document.blocks;
        while let Some(Block {
            kind: BlockKind::Quote(_, inner),
            ..
        }) = blocks.first()
        {
            depth += 1;
            blocks = inner;
        }
        assert!(depth <= MAX_DEPTH);
        assert!(matches!(blocks[0].kind, BlockKind::Paragraph(_)));
    }
}
