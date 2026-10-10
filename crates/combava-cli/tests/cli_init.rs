//! `combava init` de bout en bout (spécification, section 5.2).

mod common;

use combava_cli::commands::init;
use common::{Sandbox, arg, minimal_template};

fn today() -> chrono::NaiveDate {
    chrono::Local::now().date_naive()
}

#[test]
fn fichier_par_defaut() {
    let sandbox = Sandbox::new();
    let (code, stderr) = sandbox.run(&["init"]);
    assert_eq!((code, stderr.as_str()), (0, ""));
    assert_eq!(sandbox.read("rapport.md"), init::content(today(), None));
    assert!(
        sandbox
            .read("rapport.md")
            .contains(&format!("\ndate = \"{}\"\n", init::french_date(today())))
    );
}

#[test]
fn nom_et_template() {
    let sandbox = Sandbox::new();
    sandbox.write("docs/.garde", "");
    let (code, _) = sandbox.run(&["init", "docs/tp.md", "-t", "../maison"]);
    assert_eq!(code, 0);
    let content = sandbox.read("docs/tp.md");
    assert_eq!(content, init::content(today(), Some("../maison")));
    assert!(content.contains("\ntemplate = \"../maison\"\n+++\n"));
}

#[test]
fn file_exists() {
    let sandbox = Sandbox::new();
    sandbox.write("rapport.md", "à garder");
    let (code, stderr) = sandbox.run(&["init"]);
    assert_eq!(code, 1);
    assert_eq!(
        stderr,
        "combava: erreur[file-exists] : le fichier « rapport.md » existe déjà ; --force l'écrase\n"
    );
    assert_eq!(sandbox.read("rapport.md"), "à garder");

    let (code, _) = sandbox.run(&["init", "--force"]);
    assert_eq!(code, 0);
    assert_eq!(sandbox.read("rapport.md"), init::content(today(), None));
}

#[test]
fn dossier_absent() {
    let sandbox = Sandbox::new();
    let (code, stderr) = sandbox.run(&["init", "absent/r.md"]);
    assert_eq!(code, 1);
    assert!(
        stderr.starts_with("combava: erreur[io] : impossible d'écrire « absent/r.md » : "),
        "{stderr}"
    );
}

#[test]
fn le_fichier_cree_se_compile() {
    let sandbox = Sandbox::new();
    assert_eq!(sandbox.run(&["init"]).0, 0);
    let template = minimal_template();
    let (code, stderr) = sandbox.run(&["build", "rapport.md", "-t", arg(&template)]);
    assert_eq!((code, stderr.as_str()), (0, ""));
    assert!(
        std::fs::read(sandbox.path("rapport.pdf"))
            .unwrap()
            .starts_with(b"%PDF-")
    );
}

#[test]
#[ignore = "en attente du template par défaut (lot template, section 11)"]
fn le_fichier_cree_se_compile_avec_le_template_par_defaut() {
    let sandbox = Sandbox::new();
    assert_eq!(sandbox.run(&["init"]).0, 0);
    let (code, stderr) = sandbox.run(&["build", "rapport.md"]);
    assert_eq!((code, stderr.as_str()), (0, ""));
}
