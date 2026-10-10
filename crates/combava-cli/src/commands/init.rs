//! `combava init` (spécification, section 5.2).

use chrono::{Datelike, NaiveDate};

use crate::args::InitArgs;
use crate::config::paths::Env;
use crate::diagnostics::{Diagnostic, Reporter, display_path};
use crate::error::{Code, Failed};

pub fn run(args: &InitArgs, env: &Env, reporter: &mut Reporter) -> Result<(), Failed> {
    let path = env.argument(&args.file);
    if !args.force && path.exists() {
        return Err(reporter.fail(Diagnostic::error(
            Code::FileExists,
            format!(
                "le fichier « {} » existe déjà ; --force l'écrase",
                display_path(&path, &env.cwd)
            ),
        )));
    }
    let today = chrono::Local::now().date_naive();
    let content = content(today, args.template.as_deref());
    super::write_atomically(&path, content.as_bytes(), &env.cwd).map_err(|d| reporter.fail(d))
}

/// Le fichier créé par `init`.
pub fn content(today: NaiveDate, template: Option<&str>) -> String {
    let mut out = String::from(
        "+++\n\
         title = \"Titre du rapport\"\n\
         subtitle = \"Sous-titre\"\n\
         authors = [\"Prénom Nom\"]\n\
         teachers = [\"Prénom Nom\"]\n",
    );
    out.push_str(&format!("date = \"{}\"\n", french_date(today)));
    out.push_str(
        "toc = true\n\
         list_of_figures = false\n\
         list_of_listings = false\n\
         # header_text = \"Texte en haut de page (par défaut : le titre)\"\n",
    );
    if let Some(template) = template {
        out.push_str(&format!("template = \"{}\"\n", toml_escape(template)));
    }
    out.push_str("+++\n\n# Introduction\n\n");
    out
}

/// `10 octobre 2026`, `1er mai 2026`.
pub fn french_date(date: NaiveDate) -> String {
    const MONTHS: [&str; 12] = [
        "janvier",
        "février",
        "mars",
        "avril",
        "mai",
        "juin",
        "juillet",
        "août",
        "septembre",
        "octobre",
        "novembre",
        "décembre",
    ];
    let day = match date.day() {
        1 => "1er".to_string(),
        day => day.to_string(),
    };
    format!("{day} {} {}", MONTHS[date.month0() as usize], date.year())
}

/// La valeur reste celle donnée en argument une fois relue : seuls `\` et `"`
/// sont échappés, pour que le TOML reste valide (un chemin Windows par exemple).
fn toml_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            c if c.is_control() => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn dates_en_francais() {
        assert_eq!(french_date(date(2026, 10, 10)), "10 octobre 2026");
        assert_eq!(french_date(date(2026, 5, 1)), "1er mai 2026");
        assert_eq!(french_date(date(2027, 2, 2)), "2 février 2027");
        assert_eq!(french_date(date(2026, 8, 31)), "31 août 2026");
        assert_eq!(french_date(date(2026, 12, 25)), "25 décembre 2026");
    }

    #[test]
    fn contenu_exact() {
        assert_eq!(
            content(date(2026, 10, 10), None),
            r#"+++
title = "Titre du rapport"
subtitle = "Sous-titre"
authors = ["Prénom Nom"]
teachers = ["Prénom Nom"]
date = "10 octobre 2026"
toc = true
list_of_figures = false
list_of_listings = false
# header_text = "Texte en haut de page (par défaut : le titre)"
+++

# Introduction

"#
        );
    }

    #[test]
    fn avec_template() {
        let content = content(date(2026, 10, 10), Some(r#"C:\t"mon""#));
        assert!(
            content.contains("\ntemplate = \"C:\\\\t\\\"mon\\\"\"\n+++\n"),
            "{content}"
        );
        let toml = content.split("+++\n").nth(1).unwrap();
        let table = toml::de::DeTable::parse(toml).unwrap();
        let value = table.get_ref().get("template").unwrap();
        assert_eq!(value.get_ref().as_str(), Some(r#"C:\t"mon""#));
    }
}
