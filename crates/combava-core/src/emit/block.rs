//! Émission des blocs (spécification, sections 4.2, 4.5 et 4.6).

use super::Emitter;
use super::code::{self, CodeInfo};
use crate::ast::{Align, Block, BlockKind, Definition, Inline, InlineKind, ListItem};
use crate::escape::{markup, string};
use crate::output::Code;

impl Emitter<'_, '_, '_> {
    /// Blocs séparés par une ligne vide, ou par un simple saut de ligne après
    /// un contenu en ligne de liste compacte.
    pub(super) fn blocks(&mut self, blocks: &[Block], top_level: bool) {
        let mut previous_plain = false;
        for (i, block) in blocks.iter().enumerate() {
            if i > 0 {
                self.w.push("\n");
                if !previous_plain {
                    self.w.set_tag(None);
                    self.w.push("\n");
                }
            }
            self.block(block, top_level);
            previous_plain = matches!(block.kind, BlockKind::Plain(_));
        }
    }

    fn block(&mut self, block: &Block, top_level: bool) {
        self.w.set_tag(Some(self.line(block.span.start)));
        match &block.kind {
            BlockKind::Paragraph(inlines) => match lone_image(inlines) {
                Some(image) => self.figure(image),
                None => self.inlines(inlines),
            },
            BlockKind::Plain(inlines) => self.inlines(inlines),
            BlockKind::Heading { level, content, .. } => {
                self.w.push(&format!("#heading(level: {level})["));
                let outer = std::mem::replace(&mut self.in_heading, true);
                self.inlines(content);
                self.in_heading = outer;
                self.w.push("]");
                if let Some(label) = self.labels.heading(block.span.start) {
                    let label = format!("#label({})", string(label));
                    self.w.push(&label);
                }
                self.w.end_code();
            }
            BlockKind::Quote(callout, inner) => {
                match callout {
                    Some(kind) => self.w.push(&format!("#callout({})[\n", string(kind))),
                    None => self.w.push("#quote(block: true)[\n"),
                }
                self.blocks(inner, false);
                self.w.push("\n]");
                self.w.end_code();
            }
            BlockKind::Code {
                info,
                text,
                text_start,
            } => self.code_block(block, info.as_deref(), text, *text_start),
            BlockKind::List { start, items } => self.list(*start, items),
            BlockKind::Definitions(definitions) => self.definitions(definitions),
            BlockKind::Table {
                aligns,
                header,
                rows,
            } => self.table(aligns, header, rows),
            // Un saut de page est interdit dans un conteneur Typst (citation, callout…).
            BlockKind::Rule if top_level => {
                self.w.push("#pagebreak()");
                self.w.end_code();
            }
            BlockKind::Rule => {
                self.w.push("#line(length: 100%)");
                self.w.end_code();
            }
        }
    }

    fn figure(&mut self, image: &Inline) {
        let InlineKind::Image { dest, title, alt } = &image.kind else {
            return;
        };
        let Some(call) = self.image_call(dest, alt, &image.span, None) else {
            return;
        };
        let title = title.trim();
        if title.is_empty() {
            self.w.push(&format!("#figure({call})"));
        } else {
            self.w
                .push(&format!("#figure({call}, caption: [{}])", markup(title)));
        }
        self.w.end_code();
    }

    fn code_block(&mut self, block: &Block, info: Option<&str>, text: &str, text_start: usize) {
        let (code_info, messages) = info.map(code::parse).unwrap_or_default();
        for message in messages {
            self.diags
                .push(Code::InvalidCodeAttributes, block.span.clone(), message);
        }
        // En CommonMark, `\r` seul est aussi une fin de ligne.
        let text = text.replace("\r\n", "\n").replace('\r', "\n");
        let text = text.strip_suffix('\n').unwrap_or(&text);

        if code_info.typst {
            // Recopié ligne à ligne, chacune rattachée à sa ligne du markdown.
            let first_line = self.line(text_start);
            for (k, line) in text.split('\n').enumerate() {
                if k > 0 {
                    self.w.push("\n");
                }
                self.w.set_tag(Some(first_line + k));
                self.w.push(line);
            }
            return;
        }

        let CodeInfo { lang, caption, .. } = code_info;
        let lang = lang.as_deref().map_or_else(|| "none".to_string(), string);
        let raw = format!("raw(block: true, lang: {lang}, {})", string(text));
        match caption {
            Some(caption) => self
                .w
                .push(&format!("#figure({raw}, caption: [{}])", markup(&caption))),
            None => self.w.push(&format!("#{raw}")),
        }
        self.w.end_code();
    }

    fn list(&mut self, start: Option<u64>, items: &[ListItem]) {
        let tight = items.iter().all(|item| {
            !item
                .blocks
                .iter()
                .any(|b| matches!(b.kind, BlockKind::Paragraph(_)))
        });
        match start {
            Some(start) => self
                .w
                .push(&format!("#enum(start: {start}, tight: {tight},\n")),
            None => self.w.push(&format!("#list(tight: {tight},\n")),
        }
        for item in items {
            self.w.set_tag(Some(self.line(item.span.start)));
            self.w.push("[");
            self.blocks(&item.blocks, false);
            self.w.push("],\n");
        }
        self.w.push(")");
        self.w.end_code();
    }

    fn definitions(&mut self, definitions: &[Definition]) {
        self.w.push("#terms(\n");
        for definition in definitions {
            if let Some(first) = definition.term.first() {
                self.w.set_tag(Some(self.line(first.span.start)));
            }
            self.w.push("terms.item[");
            self.inlines(&definition.term);
            self.w.push("][");
            for (i, details) in definition.details.iter().enumerate() {
                if i > 0 {
                    self.w.push("\n\n");
                }
                self.blocks(details, false);
            }
            self.w.push("],\n");
        }
        self.w.push(")");
        self.w.end_code();
    }

    fn table(&mut self, aligns: &[Align], header: &[Vec<Inline>], rows: &[Vec<Vec<Inline>>]) {
        let columns = aligns.len().max(1);
        let aligns: Vec<_> = (0..columns)
            .map(|i| match aligns.get(i) {
                Some(Align::Center) => "center",
                Some(Align::Right) => "right",
                Some(Align::Left) | None => "left",
            })
            .collect();
        self.w.push(&format!(
            "#table(\n  columns: {columns},\n  align: ({},),\n  table.header(",
            aligns.join(", ")
        ));
        self.cells(header, columns);
        self.w.push("),\n");
        for row in rows {
            if let Some(first) = row.iter().flatten().next() {
                self.w.set_tag(Some(self.line(first.span.start)));
            }
            self.w.push("  ");
            self.cells(row, columns);
            self.w.push("\n");
        }
        self.w.push(")");
        self.w.end_code();
    }

    /// Exactement `columns` cellules : complète ou tronque la ligne.
    fn cells(&mut self, cells: &[Vec<Inline>], columns: usize) {
        for i in 0..columns {
            if i > 0 {
                self.w.push(" ");
            }
            self.w.push("[");
            if let Some(cell) = cells.get(i) {
                self.inlines(cell);
            }
            self.w.push("],");
        }
    }
}

/// L'image d'un paragraphe qui ne contient qu'elle, aux espaces près.
fn lone_image(inlines: &[Inline]) -> Option<&Inline> {
    let mut found = None;
    for inline in inlines {
        match &inline.kind {
            InlineKind::Text(text) if text.trim().is_empty() => {}
            InlineKind::SoftBreak => {}
            InlineKind::Image { .. } if found.is_none() => found = Some(inline),
            _ => return None,
        }
    }
    found
}
