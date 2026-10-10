//! Info string des blocs de code : langage et attributs `{caption="…"}`
//! (spécification, section 4.6).

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct CodeInfo {
    pub lang: Option<String>,
    pub caption: Option<String>,
    /// Bloc ` ```{=typst} ` recopié tel quel.
    pub typst: bool,
}

/// Analyse l'info string. Les messages renvoyés deviennent des avertissements
/// `invalid-code-attributes`.
pub(super) fn parse(info: &str) -> (CodeInfo, Vec<String>) {
    let info = info.trim();
    if info == "{=typst}" {
        return (
            CodeInfo {
                typst: true,
                ..CodeInfo::default()
            },
            Vec::new(),
        );
    }

    let lang_end = info
        .find(|c: char| c.is_whitespace() || c == '{')
        .unwrap_or(info.len());
    let (lang, rest) = info.split_at(lang_end);
    let mut code_info = CodeInfo {
        lang: (!lang.is_empty()).then(|| lang.to_string()),
        ..CodeInfo::default()
    };

    let rest = rest.trim();
    if rest.is_empty() {
        return (code_info, Vec::new());
    }
    let malformed = || vec![format!("attributs de bloc de code mal formés « {rest} »")];
    let Some(inner) = rest.strip_prefix('{').and_then(|r| r.strip_suffix('}')) else {
        return (code_info, malformed());
    };
    let Some(attributes) = attributes(inner) else {
        return (code_info, malformed());
    };

    let mut messages = Vec::new();
    for (key, value) in attributes {
        if key == "caption" {
            code_info.caption = Some(value);
        } else {
            messages.push(format!("attribut de bloc de code inconnu « {key} »"));
        }
    }
    (code_info, messages)
}

/// `cle="valeur" …` ; `None` si la syntaxe est invalide.
fn attributes(inner: &str) -> Option<Vec<(String, String)>> {
    let mut chars = inner.chars().peekable();
    let mut attributes = Vec::new();
    loop {
        while chars.next_if(|c| c.is_whitespace()).is_some() {}
        if chars.peek().is_none() {
            return Some(attributes);
        }

        let mut key = String::new();
        while let Some(c) = chars.next_if(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_')) {
            key.push(c);
        }
        if key.is_empty() || chars.next() != Some('=') || chars.next() != Some('"') {
            return None;
        }

        let mut value = String::new();
        loop {
            match chars.next()? {
                '"' => break,
                '\\' => match chars.next()? {
                    c @ ('"' | '\\') => value.push(c),
                    c => {
                        value.push('\\');
                        value.push(c);
                    }
                },
                c => value.push(c),
            }
        }
        attributes.push((key, value));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(lang: Option<&str>, caption: Option<&str>) -> CodeInfo {
        CodeInfo {
            lang: lang.map(str::to_string),
            caption: caption.map(str::to_string),
            typst: false,
        }
    }

    #[test]
    fn langage_seul() {
        assert_eq!(parse("rust"), (info(Some("rust"), None), vec![]));
        assert_eq!(parse(""), (info(None, None), vec![]));
        assert_eq!(parse("  python  "), (info(Some("python"), None), vec![]));
    }

    #[test]
    fn legende() {
        assert_eq!(
            parse(r#"rust {caption="Tri rapide"}"#),
            (info(Some("rust"), Some("Tri rapide")), vec![])
        );
        assert_eq!(
            parse(r#"rust{caption="a"}"#),
            (info(Some("rust"), Some("a")), vec![])
        );
        assert_eq!(
            parse(r#"{caption="Sans langage"}"#),
            (info(None, Some("Sans langage")), vec![])
        );
        assert_eq!(
            parse(r#"c { caption="Le \"main\" \\ fin" }"#),
            (info(Some("c"), Some(r#"Le "main" \ fin"#)), vec![])
        );
    }

    #[test]
    fn bloc_typst() {
        let (code_info, messages) = parse(" {=typst} ");
        assert!(code_info.typst);
        assert!(messages.is_empty());
    }

    #[test]
    fn attribut_inconnu_ignore_legende_conservee() {
        let (code_info, messages) = parse(r#"rust {caption="a" numbers="yes"}"#);
        assert_eq!(code_info, info(Some("rust"), Some("a")));
        assert_eq!(messages, ["attribut de bloc de code inconnu « numbers »"]);
    }

    #[test]
    fn syntaxe_invalide_ignore_tout() {
        for raw in [
            r#"rust {caption="a""#,
            r#"rust {caption=a}"#,
            r#"rust {caption="a}"#,
            r#"rust {="a"}"#,
            r#"rust caption="a""#,
            "typst {=typst}",
        ] {
            let (code_info, messages) = parse(raw);
            assert_eq!(code_info.caption, None, "{raw}");
            assert_eq!(messages.len(), 1, "{raw}");
            assert!(
                messages[0].starts_with("attributs de bloc de code mal formés"),
                "{raw}"
            );
        }
    }
}
