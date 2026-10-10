//! Chaque code de diagnostic de la section 2.4 : déclencheur, ligne et colonne.

use combava_core::{Code, Config, Diagnostic, Severity, transpile};

/// Tous les diagnostics, que la transpilation réussisse ou non.
fn diagnostics(markdown: &str, config: &Config) -> Vec<Diagnostic> {
    match transpile(markdown, config) {
        Ok(output) => output.diagnostics,
        Err(error) => error.diagnostics,
    }
}

/// Vérifie que `code` est signalé en `line:column` et que la gravité décide
/// du résultat.
fn assert_reported(markdown: &str, config: &Config, code: Code, line: usize, column: usize) {
    let found = diagnostics(markdown, config);
    let diagnostic = found
        .iter()
        .find(|d| d.code == code)
        .unwrap_or_else(|| panic!("{} absent de {found:?}", code.as_str()));
    assert_eq!(
        (diagnostic.line, diagnostic.column),
        (line, column),
        "{found:?}"
    );
    assert_eq!(diagnostic.severity, code.severity());
    assert!(!diagnostic.message.is_empty());
    assert!(!diagnostic.message.ends_with('.'));
    assert_eq!(
        transpile(markdown, config).is_err(),
        code.severity() == Severity::Error
    );
}

fn default() -> Config {
    Config::default()
}

#[test]
fn remote_image() {
    assert_reported(
        "texte\n\n![x](https://a.fr/i.png)",
        &default(),
        Code::RemoteImage,
        3,
        1,
    );
}

#[test]
fn invalid_image_path() {
    assert_reported(
        "Été ![x](../i.png)",
        &default(),
        Code::InvalidImagePath,
        1,
        5,
    );
    assert_reported("![x](/etc/i.png)", &default(), Code::InvalidImagePath, 1, 1);
}

#[test]
fn broken_link() {
    assert_reported(
        "# Titre\n\nVoir [x](#absent).",
        &default(),
        Code::BrokenLink,
        3,
        6,
    );
    assert_reported("Voir [[Absent]].", &default(), Code::BrokenLink, 1, 6);
}

#[test]
fn duplicate_label() {
    assert_reported(
        "# A {#x}\n\n# B {#x}\n",
        &default(),
        Code::DuplicateLabel,
        3,
        1,
    );
}

#[test]
fn citation_without_bibliography() {
    // « Été » : 3 caractères mais 4 octets, la colonne compte des caractères.
    assert_reported(
        "Été [@knuth] ok",
        &default(),
        Code::CitationWithoutBibliography,
        1,
        5,
    );
    let config = Config {
        bibliography: true,
        ..Config::default()
    };
    assert!(transpile("Été [@knuth] ok", &config).is_ok());
}

#[test]
fn raw_html() {
    assert_reported("a\n\nb <span>c</span>", &default(), Code::RawHtml, 3, 3);
    assert_reported("<div>\nbloc\n</div>\n", &default(), Code::RawHtml, 1, 1);
}

#[test]
fn commentaires_html_silencieux() {
    assert!(diagnostics("a <!-- x -->\n\n<!-- bloc -->\n", &default()).is_empty());
}

#[test]
fn invalid_label() {
    assert_reported("texte\n\n## T {#a!b}", &default(), Code::InvalidLabel, 3, 1);
}

#[test]
fn invalid_code_attributes() {
    assert_reported(
        "\n```rust {x=1}\nfn\n```\n",
        &default(),
        Code::InvalidCodeAttributes,
        2,
        1,
    );
}

#[test]
fn undefined_footnote() {
    assert_reported(
        "texte[^nope] fin",
        &default(),
        Code::UndefinedFootnote,
        1,
        6,
    );
}

#[test]
fn unused_footnote() {
    assert_reported(
        "a\n\n[^n]: jamais appelée\n",
        &default(),
        Code::UnusedFootnote,
        3,
        1,
    );
}

#[test]
fn echec_avec_erreurs_et_avertissements_tries() {
    let markdown = "<b>x</b>\n\n![a](https://x/i.png)\n\n[[Absent]]\n";
    let error = transpile(markdown, &default()).unwrap_err();
    let codes: Vec<_> = error.diagnostics.iter().map(|d| d.code).collect();
    assert_eq!(
        codes,
        [
            Code::RawHtml,
            Code::RawHtml,
            Code::RemoteImage,
            Code::BrokenLink
        ]
    );
    assert_eq!(error.to_string(), "le document contient des erreurs");
}
