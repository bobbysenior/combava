//! Échappement du texte en markup Typst, littéraux de chaîne et décodage `%XX`.

/// Caractères qui ont un sens en markup Typst (spécification, section 4.1).
const MARKUP_SPECIAL: &[char] = &[
    '\\', '#', '$', '*', '_', '@', '<', '>', '[', ']', '`', '~', '/', '=', '-', '+',
];

/// Échappe un texte pour qu'il s'affiche tel quel en markup Typst.
///
/// Les sauts de ligne deviennent des espaces : le texte d'un paragraphe ne
/// doit pas créer de structure.
pub(crate) fn markup(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut previous = None;
    for c in text.chars() {
        let escaped = MARKUP_SPECIAL.contains(&c)
            || (c == '.' && previous.is_some_and(|p: char| p.is_ascii_digit()));
        if escaped {
            out.push('\\');
        }
        out.push(if c == '\n' || c == '\r' { ' ' } else { c });
        previous = Some(c);
    }
    out
}

/// Littéral de chaîne Typst, guillemets compris.
pub(crate) fn string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => out.push_str(&format!("\\u{{{:x}}}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Décode les séquences `%XX`. Une séquence invalide est conservée telle quelle.
pub(crate) fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = |b: u8| (b as char).to_digit(16);
        match (
            bytes[i],
            bytes.get(i + 1).copied().and_then(hex),
            bytes.get(i + 2).copied().and_then(hex),
        ) {
            (b'%', Some(high), Some(low)) => {
                out.push((high * 16 + low) as u8);
                i += 3;
            }
            (b, _, _) => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| text.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chaque_caractere_special_est_echappe() {
        for c in MARKUP_SPECIAL {
            assert_eq!(markup(&c.to_string()), format!("\\{c}"));
        }
        assert_eq!(
            markup(r"\ # $ * _ @ < > [ ] ` ~ / = - +"),
            r"\\ \# \$ \* \_ \@ \< \> \[ \] \` \~ \/ \= \- \+"
        );
    }

    #[test]
    fn point_apres_un_chiffre() {
        assert_eq!(markup("2026. fin"), r"2026\. fin");
        assert_eq!(markup("3.14"), r"3\.14");
        assert_eq!(markup("fin."), "fin.");
        assert_eq!(markup("a.b"), "a.b");
    }

    #[test]
    fn guillemets_et_apostrophes_intacts() {
        assert_eq!(markup(r#"l'"été""#), r#"l'"été""#);
    }

    #[test]
    fn texte_courant_inchange() {
        assert_eq!(
            markup("Bonjour à tous : ça va ?"),
            "Bonjour à tous : ça va ?"
        );
    }

    #[test]
    fn adresse_mail_et_url() {
        assert_eq!(markup("a@b.fr"), r"a\@b.fr");
        assert_eq!(markup("https://x.fr"), r"https:\/\/x.fr");
    }

    #[test]
    fn sauts_de_ligne_remplaces() {
        assert_eq!(markup("a\nb\r\nc"), "a b  c");
    }

    #[test]
    fn litteral_de_chaine() {
        assert_eq!(string("abc"), r#""abc""#);
        assert_eq!(string(r#"a"b\c"#), r#""a\"b\\c""#);
        assert_eq!(string("a\nb\rc\td"), r#""a\nb\rc\td""#);
        assert_eq!(string("\u{1}"), r#""\u{1}""#);
        assert_eq!(string("*titre* é"), r#""*titre* é""#);
    }

    #[test]
    fn decodage_pourcent() {
        assert_eq!(percent_decode("mon%20image.png"), "mon image.png");
        assert_eq!(percent_decode("%C3%A9t%C3%A9"), "été");
        assert_eq!(percent_decode("100%"), "100%");
        assert_eq!(percent_decode("%zz"), "%zz");
        assert_eq!(percent_decode("%FF"), "%FF");
    }
}
