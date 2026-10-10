//! Snapshots : `tests/fixtures/<cas>/input.md` → Typst émis et diagnostics.
//!
//! Un cas par sous-section de la section 4 de la spécification. Pour mettre à
//! jour après une modification voulue : `cargo insta review`, ou
//! `INSTA_UPDATE=always cargo test`.

use combava_core::{Config, Diagnostic, transpile};

fn config() -> Config {
    Config {
        title: Some("Fixture".into()),
        authors: vec!["Ada Lovelace".into()],
        bibliography: true,
        ..Config::default()
    }
}

fn render(case: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{case}/input.md",
        env!("CARGO_MANIFEST_DIR")
    );
    let markdown = std::fs::read_to_string(&path).unwrap();
    let (typst, diagnostics) = match transpile(&markdown, &config()) {
        Ok(output) => (output.typst, output.diagnostics),
        Err(error) => ("(échec)\n".to_string(), error.diagnostics),
    };
    let diagnostics: String = diagnostics.iter().map(format_diagnostic).collect();
    format!("{typst}--- diagnostics ---\n{diagnostics}")
}

fn format_diagnostic(d: &Diagnostic) -> String {
    format!(
        "{}:{}: {:?}[{}] : {}\n",
        d.line,
        d.column,
        d.severity,
        d.code.as_str(),
        d.message
    )
}

macro_rules! snapshots {
    ($($case:ident),* $(,)?) => {
        $(
            #[test]
            fn $case() {
                insta::assert_snapshot!(stringify!($case), render(stringify!($case)));
            }
        )*
    };
}

snapshots!(
    texte, titres, liens, images, blocs, code, notes, citations, maths, html
);
