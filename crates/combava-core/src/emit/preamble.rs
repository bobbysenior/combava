//! Imports et appel du template (spécification, sections 2.3 et 9.1).

use crate::config::Config;
use crate::escape::string;
use crate::{MITEX_PACKAGE, paths};

pub(super) fn preamble(config: &Config, uses_math: bool) -> String {
    let mut out = format!(
        "// Généré par combava-core {} — ne pas modifier\n",
        env!("CARGO_PKG_VERSION")
    );
    if uses_math {
        out.push_str(&format!("#import {}: mi, mitex\n", string(MITEX_PACKAGE)));
    }
    out.push_str(&format!(
        "#import {}: template, callout\n",
        string(paths::TEMPLATE_ENTRY)
    ));

    let header_text = config.header_text.as_ref().or(config.title.as_ref());
    let arguments = [
        ("title", optional(config.title.as_ref())),
        ("subtitle", optional(config.subtitle.as_ref())),
        ("authors", array(&config.authors)),
        ("teachers", array(&config.teachers)),
        ("date", optional(config.date.as_ref())),
        ("school", optional(config.school.as_ref())),
        ("university", optional(config.university.as_ref())),
        ("academic-year", optional(config.academic_year.as_ref())),
        ("cohort", optional(config.cohort.as_ref())),
        ("specialization", optional(config.specialization.as_ref())),
        ("subject", optional(config.subject.as_ref())),
        ("toc", config.toc.to_string()),
        ("list-of-figures", config.list_of_figures.to_string()),
        ("list-of-listings", config.list_of_listings.to_string()),
        ("header-text", optional(header_text)),
    ];
    out.push_str("#show: template.with(\n");
    for (name, value) in arguments {
        out.push_str(&format!("  {name}: {value},\n"));
    }
    out.push_str(")\n");
    out
}

fn optional(value: Option<&String>) -> String {
    value.map_or_else(|| "none".to_string(), |v| string(v))
}

/// `("a",)` pour un élément : sans la virgule, `("a")` serait une chaîne.
fn array(values: &[String]) -> String {
    match values {
        [] => "()".to_string(),
        [single] => format!("({},)", string(single)),
        _ => format!(
            "({})",
            values
                .iter()
                .map(|v| string(v))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tableaux() {
        assert_eq!(array(&[]), "()");
        assert_eq!(array(&["a".into()]), r#"("a",)"#);
        assert_eq!(array(&["a".into(), "b\"c".into()]), r#"("a", "b\"c")"#);
    }

    #[test]
    fn tous_les_arguments_dans_l_ordre() {
        let preamble = preamble(&Config::default(), false);
        let names: Vec<_> = preamble
            .lines()
            .filter_map(|line| line.strip_prefix("  "))
            .map(|line| line.split(':').next().unwrap())
            .collect();
        assert_eq!(
            names,
            [
                "title",
                "subtitle",
                "authors",
                "teachers",
                "date",
                "school",
                "university",
                "academic-year",
                "cohort",
                "specialization",
                "subject",
                "toc",
                "list-of-figures",
                "list-of-listings",
                "header-text"
            ]
        );
        assert!(preamble.contains("  toc: true,\n"));
        assert!(preamble.contains("  title: none,\n"));
        assert!(!preamble.contains("mitex"));
    }

    #[test]
    fn header_text_par_defaut_egal_au_titre() {
        let config = Config {
            title: Some("Titre".into()),
            ..Config::default()
        };
        assert!(preamble(&config, false).contains(r#"  header-text: "Titre","#));

        let config = Config {
            title: Some("Titre".into()),
            header_text: Some("Haut".into()),
            ..Config::default()
        };
        assert!(preamble(&config, false).contains(r#"  header-text: "Haut","#));
    }

    #[test]
    fn import_de_mitex_seulement_avec_des_maths() {
        let preamble = preamble(&Config::default(), true);
        assert!(preamble.contains("#import \"@preview/mitex:0.2.7\": mi, mitex\n"));
        let mitex = preamble.find("mitex").unwrap();
        let template = preamble.find("template.typ").unwrap();
        assert!(mitex < template);
    }

    #[test]
    fn valeurs_echappees() {
        let config = Config {
            title: Some("*Le* \"titre\"\n#x".into()),
            ..Config::default()
        };
        assert!(preamble(&config, false).contains(r#"  title: "*Le* \"titre\"\n#x","#));
    }
}
