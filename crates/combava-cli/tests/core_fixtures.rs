//! Intégration des lots (spécification, section 10) : le Typst émis par le
//! core pour chacune de ses fixtures se compile avec un template conforme à la
//! section 9, sans erreur ni avertissement.
//!
//! Vérifie à la fois le code émis par le core et le respect du contrat de
//! template.

mod common;

use std::path::Path;

use combava_cli::compile::{self, Job};
use combava_cli::diagnostics::Diagnostic;
use combava_cli::template::Template;
use combava_core::{Config, transpile};

const FIXTURES: [&str; 10] = [
    "texte",
    "titres",
    "liens",
    "images",
    "blocs",
    "code",
    "notes",
    "citations",
    "maths",
    "html",
];

/// Images citées par les fixtures du core.
const IMAGES: [&str; 4] = [
    "images/logo.svg",
    "images/schema.svg",
    "images/icone.svg",
    "mon image.svg",
];

const SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="10" height="10"><rect width="10" height="10"/></svg>"#;

/// Les diagnostics de compilation de chaque fixture, rendus en texte.
fn compile_fixtures(template: Template) -> Vec<String> {
    let root = tempfile::tempdir().unwrap();
    for image in IMAGES {
        let path = root.path().join(image);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, SVG).unwrap();
    }
    let bibliography = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/references.bib");
    // Même configuration que les snapshots du core.
    let config = Config {
        title: Some("Fixture".into()),
        authors: vec!["Ada Lovelace".into()],
        bibliography: true,
        ..Config::default()
    };

    let mut problems = Vec::new();
    for case in FIXTURES {
        let fixture = common::workspace().join(format!(
            "crates/combava-core/tests/fixtures/{case}/input.md"
        ));
        let markdown = std::fs::read_to_string(&fixture).unwrap();
        let output = transpile(&markdown, &config).unwrap();
        let compiled = compile::compile(Job {
            typst: output.typst,
            source_map: &output.source_map,
            markdown: &root.path().join(format!("{case}.md")),
            template: template.clone(),
            bibliography: Some(bibliography.clone()),
        });
        problems.extend(
            compiled
                .diagnostics
                .iter()
                .map(|d: &Diagnostic| d.render(root.path())),
        );
        if compiled.pdf.is_none() {
            problems.push(format!("{case} : pas de PDF"));
        }
    }
    problems
}

#[test]
fn fixtures_du_core_avec_un_template_conforme() {
    let problems = compile_fixtures(Template::Disk(common::minimal_template()));
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
#[ignore = "en attente du template par défaut (lot template, section 11)"]
fn fixtures_du_core_avec_le_template_par_defaut() {
    let template = combava_cli::template::resolve(
        &combava_cli::config::layers::TemplateRef::Name("default".into()),
        None,
        None,
    )
    .unwrap();
    let problems = compile_fixtures(template);
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}
