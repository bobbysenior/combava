//! Lecture et fusion des couches de configuration (spécification, sections
//! 6.2 et 6.3).
//!
//! Le TOML est validé à la main plutôt qu'avec `#[serde(deny_unknown_fields)]` :
//! chaque erreur reçoit ainsi son propre code (`unknown-key`, `invalid-type`…),
//! un message en français et la position exacte de la clé ou de la valeur.

use std::ops::Range;
use std::path::{Path, PathBuf};

use toml::Spanned;
use toml::de::{DeTable, DeValue};

use super::paths::Env;
use crate::diagnostics::{Diagnostic, Location};
use crate::error::Code;

/// D'où vient une couche, de la plus prioritaire à la moins prioritaire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Arguments,
    Frontmatter,
    Project,
    Global,
}

/// Une valeur et l'endroit où elle est définie, pour les messages d'erreur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Setting<T> {
    pub value: T,
    pub location: Location,
}

/// Valeur de la clé `template` (section 7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateRef {
    /// Un nom, cherché dans les dossiers de templates puis parmi les
    /// templates embarqués.
    Name(String),
    /// Un chemin vers le dossier du template, déjà résolu.
    Path(PathBuf),
}

impl TemplateRef {
    /// Un chemin si la valeur contient `/` ou `\`, ou commence par `.` ou `~` ;
    /// un nom sinon. Un chemin relatif l'est à `base`.
    pub fn parse(value: &str, base: &Path, env: &Env) -> Self {
        if value.contains(['/', '\\']) || value.starts_with(['.', '~']) {
            Self::Path(env.config_path(value, base))
        } else {
            Self::Name(value.to_string())
        }
    }
}

/// Les clés définies par une couche ; `None` pour une clé absente.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Layer {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub authors: Option<Vec<String>>,
    pub teachers: Option<Vec<String>>,
    pub date: Option<String>,
    pub school: Option<String>,
    pub university: Option<String>,
    pub academic_year: Option<String>,
    pub cohort: Option<String>,
    pub specialization: Option<String>,
    pub subject: Option<String>,
    pub toc: Option<bool>,
    pub list_of_figures: Option<bool>,
    pub list_of_listings: Option<bool>,
    pub header_text: Option<String>,
    /// Chemin du `.bib`, déjà résolu.
    pub bibliography: Option<Setting<PathBuf>>,
    pub template: Option<Setting<TemplateRef>>,
    /// Chemin du fichier de sortie, déjà résolu.
    pub output: Option<Setting<PathBuf>>,
}

impl Layer {
    /// Fusion champ par champ : la valeur de `self` l'emporte sur celle de
    /// `lower`. Une liste est remplacée, jamais concaténée.
    pub fn or(self, lower: Self) -> Self {
        Self {
            title: self.title.or(lower.title),
            subtitle: self.subtitle.or(lower.subtitle),
            authors: self.authors.or(lower.authors),
            teachers: self.teachers.or(lower.teachers),
            date: self.date.or(lower.date),
            school: self.school.or(lower.school),
            university: self.university.or(lower.university),
            academic_year: self.academic_year.or(lower.academic_year),
            cohort: self.cohort.or(lower.cohort),
            specialization: self.specialization.or(lower.specialization),
            subject: self.subject.or(lower.subject),
            toc: self.toc.or(lower.toc),
            list_of_figures: self.list_of_figures.or(lower.list_of_figures),
            list_of_listings: self.list_of_listings.or(lower.list_of_listings),
            header_text: self.header_text.or(lower.header_text),
            bibliography: self.bibliography.or(lower.bibliography),
            template: self.template.or(lower.template),
            output: self.output.or(lower.output),
        }
    }
}

/// Un texte TOML à lire comme une couche.
pub struct Input<'a> {
    pub source: Source,
    /// Le fichier qui contient le TOML, pour les positions.
    pub path: &'a Path,
    /// Le contenu complet de ce fichier.
    pub text: &'a str,
    /// La plage du TOML dans `text` : le frontmatter, ou tout le fichier.
    pub range: Range<usize>,
    /// Le dossier par rapport auquel les chemins relatifs sont résolus.
    pub base: &'a Path,
}

/// Lit une couche. Toutes les erreurs sont rapportées en une fois.
pub fn parse(input: &Input, env: &Env) -> Result<Layer, Vec<Diagnostic>> {
    let toml = &input.text[input.range.clone()];
    let table = DeTable::parse(toml).map_err(|error| {
        let message = error.message().trim().replace('\n', " ; ");
        let offset = error.span().map_or(0, |span| span.start);
        vec![
            Diagnostic::error(Code::InvalidToml, format!("TOML invalide : {message}"))
                .at(input.location(offset)),
        ]
    })?;

    let mut entries: Vec<_> = table.get_ref().iter().collect();
    entries.sort_by_key(|(key, _)| key.span().start);
    let mut reader = Reader {
        input,
        env,
        layer: Layer::default(),
        errors: Vec::new(),
    };
    for (key, value) in entries {
        reader.entry(key, value);
    }
    if reader.errors.is_empty() {
        Ok(reader.layer)
    } else {
        Err(reader.errors)
    }
}

impl Input<'_> {
    fn location(&self, offset_in_toml: usize) -> Location {
        Location::at(self.path, self.text, self.range.start + offset_in_toml)
    }
}

struct Reader<'a, 'b> {
    input: &'a Input<'b>,
    env: &'a Env,
    layer: Layer,
    errors: Vec<Diagnostic>,
}

impl Reader<'_, '_> {
    fn entry(&mut self, key: &Spanned<std::borrow::Cow<'_, str>>, value: &Spanned<DeValue>) {
        let name = key.get_ref().as_ref();
        match name {
            "title" => self.layer.title = self.string(name, value),
            "subtitle" => self.layer.subtitle = self.string(name, value),
            "authors" => self.layer.authors = self.strings(name, value),
            "teachers" => self.layer.teachers = self.strings(name, value),
            "date" => self.layer.date = self.string(name, value),
            "school" => self.layer.school = self.string(name, value),
            "university" => self.layer.university = self.string(name, value),
            "academic_year" => self.layer.academic_year = self.string(name, value),
            "cohort" => self.layer.cohort = self.string(name, value),
            "specialization" => self.layer.specialization = self.string(name, value),
            "subject" => self.layer.subject = self.string(name, value),
            "toc" => self.layer.toc = self.boolean(name, value),
            "list_of_figures" => self.layer.list_of_figures = self.boolean(name, value),
            "list_of_listings" => self.layer.list_of_listings = self.boolean(name, value),
            "header_text" => self.layer.header_text = self.string(name, value),
            "bibliography" => {
                self.layer.bibliography = self.string(name, value).map(|path| Setting {
                    value: self.env.config_path(&path, self.input.base),
                    location: self.input.location(value.span().start),
                });
            }
            "template" => {
                self.layer.template = self.string(name, value).map(|template| Setting {
                    value: TemplateRef::parse(&template, self.input.base, self.env),
                    location: self.input.location(value.span().start),
                });
            }
            "output" if self.input.source == Source::Frontmatter => {
                self.layer.output = self.string(name, value).map(|path| Setting {
                    value: self.env.config_path(&path, self.input.base),
                    location: self.input.location(value.span().start),
                });
            }
            "output" => self.error(
                Code::OutputOutsideFrontmatter,
                "la clé « output » n'est permise que dans le frontmatter du document".into(),
                key.span().start,
            ),
            _ => self.error(
                Code::UnknownKey,
                format!("clé inconnue « {name} »"),
                key.span().start,
            ),
        }
    }

    fn string(&mut self, name: &str, value: &Spanned<DeValue>) -> Option<String> {
        match value.get_ref() {
            DeValue::String(s) => Some(s.to_string()),
            other => {
                self.invalid_type(name, "une chaîne", other, value.span());
                None
            }
        }
    }

    fn strings(&mut self, name: &str, value: &Spanned<DeValue>) -> Option<Vec<String>> {
        let expected = "un tableau de chaînes";
        let DeValue::Array(items) = value.get_ref() else {
            self.invalid_type(name, expected, value.get_ref(), value.span());
            return None;
        };
        let mut strings = Vec::with_capacity(items.len());
        for item in items.iter() {
            match item.get_ref() {
                DeValue::String(s) => strings.push(s.to_string()),
                other => {
                    let found = format!("un tableau contenant {}", type_name(other));
                    self.error(
                        Code::InvalidType,
                        format!("« {name} » doit être {expected}, pas {found}"),
                        item.span().start,
                    );
                    return None;
                }
            }
        }
        Some(strings)
    }

    fn boolean(&mut self, name: &str, value: &Spanned<DeValue>) -> Option<bool> {
        match value.get_ref() {
            DeValue::Boolean(b) => Some(*b),
            other => {
                self.invalid_type(name, "un booléen (true ou false)", other, value.span());
                None
            }
        }
    }

    fn invalid_type(&mut self, name: &str, expected: &str, found: &DeValue, span: Range<usize>) {
        let found = type_name(found);
        self.error(
            Code::InvalidType,
            format!("« {name} » doit être {expected}, pas {found}"),
            span.start,
        );
    }

    fn error(&mut self, code: Code, message: String, offset: usize) {
        self.errors
            .push(Diagnostic::error(code, message).at(self.input.location(offset)));
    }
}

fn type_name(value: &DeValue) -> &'static str {
    match value {
        DeValue::String(_) => "une chaîne",
        DeValue::Integer(_) => "un entier",
        DeValue::Float(_) => "un nombre décimal",
        DeValue::Boolean(_) => "un booléen",
        DeValue::Datetime(_) => "une date TOML (mettre la date entre guillemets)",
        DeValue::Array(_) => "un tableau",
        DeValue::Table(_) => "une table",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env() -> Env {
        Env {
            cwd: "/courant".into(),
            home: Some("/home/ada".into()),
            global_config: None,
        }
    }

    fn parse_as(source: Source, toml: &str) -> Result<Layer, Vec<Diagnostic>> {
        let input = Input {
            source,
            path: Path::new("/p/a.toml"),
            text: toml,
            range: 0..toml.len(),
            base: Path::new("/p"),
        };
        parse(&input, &env())
    }

    fn errors(toml: &str) -> Vec<(String, usize, usize, String)> {
        parse_as(Source::Project, toml)
            .unwrap_err()
            .into_iter()
            .map(|d| match d.location {
                Location::Position { line, column, .. } => {
                    (d.code.to_string(), line, column, d.message)
                }
                _ => panic!("{d:?}"),
            })
            .collect()
    }

    #[test]
    fn toutes_les_cles() {
        let toml = r#"
title = "T"
subtitle = "S"
authors = ["A", "B"]
teachers = []
date = "10 octobre 2026"
school = "É"
university = "U"
academic_year = "2026-2027"
cohort = "C"
specialization = "Sp"
subject = "Su"
toc = false
list_of_figures = true
list_of_listings = true
header_text = "H"
bibliography = "refs.bib"
template = "maison"
output = "out/r.pdf"
"#;
        let layer = parse_as(Source::Frontmatter, toml).unwrap();
        assert_eq!(layer.title.as_deref(), Some("T"));
        assert_eq!(layer.authors, Some(vec!["A".into(), "B".into()]));
        assert_eq!(layer.teachers, Some(vec![]));
        assert_eq!(layer.toc, Some(false));
        assert_eq!(layer.list_of_listings, Some(true));
        assert_eq!(layer.header_text.as_deref(), Some("H"));
        let bibliography = layer.bibliography.unwrap();
        assert_eq!(bibliography.value, Path::new("/p/refs.bib"));
        assert_eq!(
            bibliography.location,
            Location::Position {
                path: "/p/a.toml".into(),
                line: 17,
                column: 16
            }
        );
        assert_eq!(
            layer.template.unwrap().value,
            TemplateRef::Name("maison".into())
        );
        assert_eq!(layer.output.unwrap().value, Path::new("/p/out/r.pdf"));
    }

    #[test]
    fn cles_absentes() {
        assert_eq!(parse_as(Source::Global, "").unwrap(), Layer::default());
    }

    #[test]
    fn noms_et_chemins_de_template() {
        let env = env();
        let base = Path::new("/p");
        let parse = |v| TemplateRef::parse(v, base, &env);
        assert_eq!(parse("default"), TemplateRef::Name("default".into()));
        assert_eq!(
            parse("mon-template_2"),
            TemplateRef::Name("mon-template_2".into())
        );
        assert_eq!(parse("./t"), TemplateRef::Path("/p/t".into()));
        assert_eq!(parse(".t"), TemplateRef::Path("/p/.t".into()));
        assert_eq!(parse("a/b"), TemplateRef::Path("/p/a/b".into()));
        assert_eq!(parse("a\\b"), TemplateRef::Path("/p/a\\b".into()));
        assert_eq!(parse("~/t"), TemplateRef::Path("/home/ada/t".into()));
        assert_eq!(parse("/abs/t"), TemplateRef::Path("/abs/t".into()));
    }

    #[test]
    fn cle_inconnue() {
        assert_eq!(
            errors("title = \"x\"\nautors = []\n[table]\nx = 1\n"),
            [
                ("unknown-key".into(), 2, 1, "clé inconnue « autors »".into()),
                ("unknown-key".into(), 3, 2, "clé inconnue « table »".into()),
            ]
        );
    }

    #[test]
    fn mauvais_types() {
        let found =
            errors("date = 2026-10-10\ntoc = \"oui\"\nauthors = \"A\"\nteachers = [\"A\", 1]\n");
        let found: Vec<_> = found
            .iter()
            .map(|(c, l, col, m)| (c.as_str(), *l, *col, m.as_str()))
            .collect();
        assert_eq!(
            found,
            [
                (
                    "invalid-type",
                    1,
                    8,
                    "« date » doit être une chaîne, pas une date TOML (mettre la date entre guillemets)"
                ),
                (
                    "invalid-type",
                    2,
                    7,
                    "« toc » doit être un booléen (true ou false), pas une chaîne"
                ),
                (
                    "invalid-type",
                    3,
                    11,
                    "« authors » doit être un tableau de chaînes, pas une chaîne"
                ),
                (
                    "invalid-type",
                    4,
                    18,
                    "« teachers » doit être un tableau de chaînes, pas un tableau contenant un entier"
                ),
            ]
        );
    }

    #[test]
    fn toml_invalide() {
        let found = errors("title = \"x\"\ntitle = \"y\"\n");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, "invalid-toml");
        assert_eq!((found[0].1, found[0].2), (2, 1));
        assert!(found[0].3.starts_with("TOML invalide : "), "{found:?}");

        let found = errors("a = \n");
        assert_eq!(found[0].0, "invalid-toml");
        assert!(!found[0].3.contains('\n'));
    }

    #[test]
    fn output_hors_frontmatter() {
        for source in [Source::Project, Source::Global] {
            let error = &parse_as(source, "\noutput = \"a.pdf\"").unwrap_err()[0];
            assert_eq!(error.code, "output-outside-frontmatter");
            assert!(matches!(
                error.location,
                Location::Position {
                    line: 2,
                    column: 1,
                    ..
                }
            ));
        }
    }

    #[test]
    fn positions_dans_le_frontmatter() {
        // Le TOML commence au milieu du fichier : les positions sont celles du fichier.
        let text = "+++\ntitle = \"x\"\n  autors = 1\n+++\n";
        let input = Input {
            source: Source::Frontmatter,
            path: Path::new("/p/r.md"),
            text,
            range: 4..text.len() - 4,
            base: Path::new("/p"),
        };
        let error = &parse(&input, &env()).unwrap_err()[0];
        assert_eq!(
            error.location,
            Location::Position {
                path: "/p/r.md".into(),
                line: 3,
                column: 3
            }
        );
    }

    #[test]
    fn fusion() {
        let high = Layer {
            title: Some("haut".into()),
            authors: Some(vec![]),
            ..Layer::default()
        };
        let low = Layer {
            title: Some("bas".into()),
            subtitle: Some("bas".into()),
            authors: Some(vec!["A".into()]),
            toc: Some(false),
            ..Layer::default()
        };
        let merged = high.or(low);
        assert_eq!(merged.title.as_deref(), Some("haut"));
        assert_eq!(merged.subtitle.as_deref(), Some("bas"));
        // Une liste vide remplace la liste de la couche inférieure.
        assert_eq!(merged.authors, Some(vec![]));
        assert_eq!(merged.toc, Some(false));
        assert_eq!(merged.header_text, None);
    }
}
