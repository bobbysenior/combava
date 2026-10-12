//! Labels des titres et des tableaux : `{#id}` explicites et slugs automatiques
//! des titres (spécification, sections 4.2 et 4.5).

use std::collections::{HashMap, HashSet};
use std::ops::Range;

use crate::ast::{Block, BlockKind, Document, plain_text};
use crate::diag::Diagnostics;
use crate::output::Code;

/// Slug calculé comme GitHub, pour que `[voir](#ma-section)` fonctionne aussi
/// dans l'aperçu.
pub(crate) fn slug(text: &str) -> String {
    let mut slug = String::new();
    for c in text.chars().flat_map(char::to_lowercase) {
        if c.is_alphanumeric() || c == '-' || c == '_' {
            slug.push(c);
        } else if c == ' ' {
            slug.push('-');
        }
    }
    if slug.is_empty() {
        "section".to_string()
    } else {
        slug
    }
}

/// Un `{#id}` explicite n'est accepté que s'il reste lisible dans un label Typst.
pub(crate) fn is_valid_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | '.' | ':'))
}

/// Labels de tous les titres et tableaux du document.
pub(crate) struct Labels {
    /// Label de chaque titre, indexé par l'octet de début du titre.
    by_heading: HashMap<usize, String>,
    /// Labels valides des tableaux.
    tables: HashSet<String>,
    explicit: HashSet<String>,
    all: HashSet<String>,
}

/// Un élément qui peut porter un label.
struct Target<'a> {
    /// Le titre entier, ou le `{#id}` d'un tableau.
    span: Range<usize>,
    id: Option<&'a str>,
    kind: TargetKind,
}

enum TargetKind {
    /// Texte du titre, pour le slug automatique.
    Heading(String),
    /// Un tableau n'a pas de label automatique.
    Table,
}

impl Labels {
    pub(crate) fn collect(document: &Document, diags: &mut Diagnostics) -> Self {
        let mut targets = Vec::new();
        collect_targets(&document.blocks, &mut targets);
        for footnote in &document.footnotes {
            collect_targets(&footnote.blocks, &mut targets);
        }

        // Les `{#id}` explicites sont réservés avant le calcul des slugs.
        let mut explicit = HashSet::new();
        let mut by_heading = HashMap::new();
        let mut tables = HashSet::new();
        let mut pending = Vec::new();
        for target in &targets {
            match (target.id, &target.kind) {
                (Some(id), kind) if is_valid_id(id) => {
                    if !explicit.insert(id.to_string()) {
                        diags.push(
                            Code::DuplicateLabel,
                            target.span.clone(),
                            format!("label « {id} » déjà utilisé par un autre titre ou tableau"),
                        );
                    }
                    match kind {
                        TargetKind::Heading(_) => {
                            by_heading.insert(target.span.start, id.to_string());
                        }
                        TargetKind::Table => {
                            tables.insert(id.to_string());
                        }
                    }
                }
                (invalid, TargetKind::Heading(text)) => pending.push((target, invalid, text)),
                (id, TargetKind::Table) => diags.push(
                    Code::InvalidLabel,
                    target.span.clone(),
                    format!(
                        "label « {} » invalide : seuls les lettres, les chiffres, « - », « _ », « . » et « : » sont autorisés ; le tableau n'a pas de label",
                        id.unwrap_or_default()
                    ),
                ),
            }
        }

        let mut all = explicit.clone();
        for (heading, invalid, text) in pending {
            let base = slug(text);
            let mut candidate = base.clone();
            let mut n = 0;
            while all.contains(&candidate) {
                n += 1;
                candidate = format!("{base}-{n}");
            }
            if let Some(id) = invalid {
                diags.push(
                    Code::InvalidLabel,
                    heading.span.clone(),
                    format!(
                        "label « {id} » invalide : seuls les lettres, les chiffres, « - », « _ », « . » et « : » sont autorisés ; le label « {candidate} » est utilisé"
                    ),
                );
            }
            all.insert(candidate.clone());
            by_heading.insert(heading.span.start, candidate);
        }

        Labels {
            by_heading,
            tables,
            explicit,
            all,
        }
    }

    /// Label du titre qui commence à l'octet `start`.
    pub(crate) fn heading(&self, start: usize) -> Option<&str> {
        self.by_heading.get(&start).map(String::as_str)
    }

    /// `label` est celui d'un tableau : `[@label]` y renvoie par son numéro.
    pub(crate) fn is_table(&self, label: &str) -> bool {
        self.tables.contains(label)
    }

    pub(crate) fn contains(&self, label: &str) -> bool {
        self.all.contains(label)
    }

    /// Label visé par `[[cible]]` : `cible` si c'est un `{#id}`, sinon son slug.
    pub(crate) fn wiki_target(&self, target: &str) -> String {
        if self.explicit.contains(target) {
            target.to_string()
        } else {
            slug(target)
        }
    }
}

fn collect_targets<'a>(blocks: &'a [Block], out: &mut Vec<Target<'a>>) {
    for block in blocks {
        match &block.kind {
            BlockKind::Heading { id, content, .. } => out.push(Target {
                span: block.span.clone(),
                id: id.as_deref(),
                kind: TargetKind::Heading(plain_text(content)),
            }),
            BlockKind::Table {
                label: Some((id, span)),
                ..
            } => out.push(Target {
                span: span.clone(),
                id: Some(id),
                kind: TargetKind::Table,
            }),
            BlockKind::Quote(_, inner) => collect_targets(inner, out),
            BlockKind::List { items, .. } => {
                for item in items {
                    collect_targets(&item.blocks, out);
                }
            }
            BlockKind::Definitions(definitions) => {
                for details in definitions.iter().flat_map(|d| &d.details) {
                    collect_targets(details, out);
                }
            }
            BlockKind::Paragraph(_)
            | BlockKind::Plain(_)
            | BlockKind::Code { .. }
            | BlockKind::Table { .. }
            | BlockKind::Rule => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugs_comme_github() {
        assert_eq!(slug("Ma Section"), "ma-section");
        assert_eq!(slug("Première partie : l'été !"), "première-partie--lété-");
        assert_eq!(slug("API_v2 - Détails"), "api_v2---détails");
        assert_eq!(slug("Étape 1.2"), "étape-12");
        assert_eq!(slug("?!"), "section");
        assert_eq!(slug(""), "section");
    }

    #[test]
    fn ids_explicites() {
        for id in ["intro", "partie-1", "a_b", "fig.1", "sec:intro", "été"] {
            assert!(is_valid_id(id), "{id}");
        }
        for id in ["", "a b", "é!x", "a/b", "a\"b"] {
            assert!(!is_valid_id(id), "{id}");
        }
    }

    fn labels_of(markdown: &str) -> (Vec<String>, Vec<Code>) {
        let mut diags = Diagnostics::new(markdown);
        let document = crate::parser::parse(markdown, &mut diags);
        let labels = Labels::collect(&document, &mut diags);
        let found = document
            .blocks
            .iter()
            .filter_map(|b| labels.heading(b.span.start).map(str::to_string))
            .collect();
        let codes = diags.into_sorted().into_iter().map(|d| d.code).collect();
        (found, codes)
    }

    #[test]
    fn doublons_numerotes_dans_l_ordre() {
        let (labels, codes) = labels_of("# Intro\n\n# Intro\n\n# Intro\n");
        assert_eq!(labels, ["intro", "intro-1", "intro-2"]);
        assert!(codes.is_empty());
    }

    #[test]
    fn id_explicite_reserve_avant_les_slugs() {
        let (labels, _) = labels_of("# Intro\n\n# Autre {#intro}\n");
        assert_eq!(labels, ["intro-1", "intro"]);
    }

    #[test]
    fn id_invalide_remplace_par_le_slug() {
        let (labels, codes) = labels_of("# Titre {#é!x}\n");
        assert_eq!(labels, ["titre"]);
        assert_eq!(codes, [Code::InvalidLabel]);
    }

    fn table_labels_of(markdown: &str) -> (Vec<&'static str>, Vec<Code>) {
        let mut diags = Diagnostics::new(markdown);
        let document = crate::parser::parse(markdown, &mut diags);
        let labels = Labels::collect(&document, &mut diags);
        let found = ["a", "b", "x", "x-1", "a b"]
            .into_iter()
            .filter(|id| labels.is_table(id))
            .collect();
        let codes = diags.into_sorted().into_iter().map(|d| d.code).collect();
        (found, codes)
    }

    #[test]
    fn labels_de_tableaux() {
        let table = "| a |\n|---|\n| 1 |\n\n";
        let (found, codes) = table_labels_of(&format!(
            "{table}: A {{#a}}\n\n> {}: {{#b}}\n\n{table}: Sans label\n",
            table.replace('\n', "\n> ").trim_end()
        ));
        assert_eq!(found, ["a", "b"]);
        assert!(codes.is_empty());

        let (found, codes) = table_labels_of(&format!("{table}: L {{#a b}}\n"));
        assert!(found.is_empty());
        assert_eq!(codes, [Code::InvalidLabel]);
    }

    #[test]
    fn label_de_tableau_reserve_avant_les_slugs() {
        let table = "| a |\n|---|\n| 1 |\n\n";
        let markdown = format!("# X\n\n{table}: {{#x}}\n");
        let (titles, codes) = labels_of(&markdown);
        assert_eq!(titles, ["x-1"]);
        assert!(codes.is_empty());
        let (_, codes) = table_labels_of(&format!("# T {{#x}}\n\n{table}: {{#x}}\n"));
        assert_eq!(codes, [Code::DuplicateLabel]);
    }

    #[test]
    fn id_explicite_en_double() {
        let (labels, codes) = labels_of("# A {#x}\n\n# B {#x}\n");
        assert_eq!(labels, ["x", "x"]);
        assert_eq!(codes, [Code::DuplicateLabel]);
    }
}
