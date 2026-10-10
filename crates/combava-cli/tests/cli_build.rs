//! `combava build` de bout en bout (spécification, sections 5.1, 5.4 à 5.6).

mod common;

use common::{Sandbox, arg, minimal_template, workspace};

/// Un document simple, sans maths : il compile sans réseau.
const DOC: &str = "+++\ntitle = \"Titre\"\n+++\n\n# Introduction\n\nTexte.\n";

fn build(sandbox: &Sandbox, file: &str, extra: &[&str]) -> (i32, String) {
    let template = minimal_template();
    let mut args = vec!["build", file, "-t", arg(&template)];
    args.extend_from_slice(extra);
    sandbox.run(&args)
}

#[test]
fn document_de_reference() {
    let sandbox = Sandbox::new();
    let report = workspace().join("examples/report.md");
    let out = sandbox.path("report.pdf");
    let (code, stderr) = build(&sandbox, arg(&report), &["-o", arg(&out)]);
    assert_eq!((code, stderr.as_str()), (0, ""));
    assert!(std::fs::read(&out).unwrap().starts_with(b"%PDF-"));
}

#[test]
fn document_de_reference_avec_le_template_par_defaut() {
    let sandbox = Sandbox::new();
    let report = workspace().join("examples/report.md");
    let out = sandbox.path("report.pdf");
    let (code, stderr) = sandbox.run(&["build", arg(&report), "-o", arg(&out)]);
    assert_eq!((code, stderr.as_str()), (0, ""));
    assert!(std::fs::read(&out).unwrap().starts_with(b"%PDF-"));
}

#[test]
fn template_par_defaut_sans_aucun_champ() {
    // Le core passe `none` ou `()` pour chaque argument absent (section 9.1).
    let sandbox = Sandbox::new();
    sandbox.write("r.md", "+++\ntoc = false\n+++\n\nTexte.\n");
    let (code, stderr) = sandbox.run(&["build", "r.md"]);
    assert_eq!((code, stderr.as_str()), (0, ""));
}

#[test]
fn template_par_defaut_titres_dans_des_conteneurs() {
    // Un titre de niveau 1 ouvre une page, ce que Typst interdit dans un
    // conteneur.
    let sandbox = Sandbox::new();
    sandbox.write(
        "r.md",
        "> # Citation\n\n> [!NOTE]\n> # Callout\n\n- # Liste\n\n1. # Numérotée\n\n\
         Terme\n: # Définition\n\nUne note[^n].\n\n[^n]: # Note\n",
    );
    let (code, stderr) = sandbox.run(&["build", "r.md"]);
    assert_eq!((code, stderr.as_str()), (0, ""));
}

#[test]
fn template_par_defaut_callout_inconnu() {
    let sandbox = Sandbox::new();
    sandbox.write("r.md", "```{=typst}\n#callout(\"autre\")[x]\n```\n");
    let (code, stderr) = sandbox.run(&["build", "r.md"]);
    assert_eq!(code, 1);
    assert!(
        stderr.contains("callout : type « autre » inconnu, attendu : « note », « tip »"),
        "{stderr}"
    );
}

#[test]
fn sortie_par_defaut_a_cote_du_markdown() {
    let sandbox = Sandbox::new();
    sandbox.write("docs/rapport.md", DOC);
    let (code, stderr) = build(&sandbox, "docs/rapport.md", &[]);
    assert_eq!((code, stderr.as_str()), (0, ""));
    let pdf = std::fs::read(sandbox.path("docs/rapport.pdf")).unwrap();
    assert!(pdf.starts_with(b"%PDF-"));
}

#[test]
fn output_du_frontmatter_relatif_au_markdown() {
    let sandbox = Sandbox::new();
    sandbox.write("docs/sortie/.garde", "");
    sandbox.write(
        "docs/r.md",
        "+++\noutput = \"sortie/final.pdf\"\n+++\n\nTexte.\n",
    );
    let (code, stderr) = build(&sandbox, "docs/r.md", &[]);
    assert_eq!((code, stderr.as_str()), (0, ""));
    assert!(sandbox.path("docs/sortie/final.pdf").is_file());
    // `-o` l'emporte, relatif au dossier courant.
    let (code, _) = build(&sandbox, "docs/r.md", &["-o", "ici.pdf"]);
    assert_eq!(code, 0);
    assert!(sandbox.path("ici.pdf").is_file());
}

#[test]
fn transpile_only() {
    let sandbox = Sandbox::new();
    sandbox.write("r.md", DOC);
    let (code, stderr) = sandbox.run(&["build", "r.md", "--transpile-only", "-o", "x.pdf"]);
    assert_eq!((code, stderr.as_str()), (0, ""));
    assert!(
        sandbox
            .read("x.typ")
            .starts_with("// Généré par combava-core ")
    );
    assert!(!sandbox.path("x.pdf").exists());

    let (code, _) = sandbox.run(&["build", "r.md", "--transpile-only"]);
    assert_eq!(code, 0);
    assert!(sandbox.path("r.typ").exists());
}

#[test]
fn avertissement_du_core() {
    let sandbox = Sandbox::new();
    sandbox.write("r.md", "# Titre\n\nTexte <b>gras</b>.\n");
    let (code, stderr) = build(&sandbox, "r.md", &[]);
    assert_eq!(code, 0);
    assert_eq!(
        stderr,
        "r.md:3:7: avertissement[raw-html] : HTML brut ignoré\n\
         r.md:3:14: avertissement[raw-html] : HTML brut ignoré\n"
    );
    assert!(sandbox.path("r.pdf").exists());
}

#[test]
fn erreur_du_core_et_pdf_precedent_conserve() {
    let sandbox = Sandbox::new();
    sandbox.write("r.pdf", "ancien PDF");
    sandbox.write("r.md", "+++\ntitle = \"x\"\n+++\n\nVoir [[Absent]].\n");
    let (code, stderr) = build(&sandbox, "r.md", &[]);
    assert_eq!(code, 1);
    assert!(
        stderr.starts_with("r.md:5:6: erreur[broken-link] : "),
        "{stderr}"
    );
    assert_eq!(sandbox.read("r.pdf"), "ancien PDF");
}

#[test]
fn erreur_typst_ramenee_dans_le_markdown() {
    let sandbox = Sandbox::new();
    sandbox.write("r.pdf", "ancien PDF");
    sandbox.write(
        "r.md",
        "# Titre\n\n```{=typst}\n#let a = 1\n#inconnue()\n```\n",
    );
    let (code, stderr) = build(&sandbox, "r.md", &[]);
    assert_eq!(code, 1);
    assert!(
        stderr.starts_with("r.md:5:1: erreur[typst] : unknown variable: inconnue"),
        "{stderr}"
    );
    // L'erreur vient du document, pas du template.
    assert!(!stderr.contains("contrat"), "{stderr}");
    assert_eq!(sandbox.read("r.pdf"), "ancien PDF");
}

/// L'indication ajoutée aux erreurs du préambule pour le template `nom`.
fn template_hint(name: &str) -> String {
    format!(
        "  aide : le template « {name} » doit définir « template » et « callout », \
         et accepter tous les arguments du contrat (spécification, section 9)\n"
    )
}

#[test]
fn template_vide() {
    // Typst signale un `unresolved import` par nom importé, à deux colonnes de
    // la même ligne du préambule : un seul diagnostic est affiché.
    let sandbox = Sandbox::new();
    sandbox.write("t/template.typ", "");
    sandbox.write("r.md", DOC);
    let (code, stderr) = sandbox.run(&["build", "r.md", "-t", "./t"]);
    assert_eq!(code, 1);
    assert_eq!(
        stderr,
        format!(
            "r.md: erreur[typst] : unresolved import (ligne 2 du code généré, voir --transpile-only)\n{}",
            template_hint("t")
        )
    );
}

#[test]
fn template_sans_callout() {
    let sandbox = Sandbox::new();
    sandbox.write(
        ".combava/templates/maison/template.typ",
        "#let template(..args, body) = body\n",
    );
    sandbox.write("r.md", DOC);
    let (code, stderr) = sandbox.run(&["build", "r.md", "-t", "maison"]);
    assert_eq!(code, 1);
    assert_eq!(
        stderr,
        format!(
            "r.md: erreur[typst] : unresolved import (ligne 2 du code généré, voir --transpile-only)\n{}",
            template_hint("maison")
        )
    );
}

#[test]
fn template_sans_un_argument_du_contrat() {
    let sandbox = Sandbox::new();
    let minimal = std::fs::read_to_string(minimal_template().join("template.typ")).unwrap();
    let without_cohort = minimal
        .replace("  cohort: none,\n", "")
        .replace("cohort, ", "");
    assert_ne!(without_cohort, minimal);
    sandbox.write("t/template.typ", without_cohort);
    sandbox.write("r.md", DOC);
    let (code, stderr) = sandbox.run(&["build", "r.md", "-t", "./t"]);
    assert_eq!(code, 1);
    assert!(
        stderr.starts_with("r.md: erreur[typst] : unexpected argument: cohort (ligne "),
        "{stderr}"
    );
    assert!(stderr.ends_with(&template_hint("t")), "{stderr}");
    assert_eq!(stderr.lines().count(), 2, "{stderr}");
}

#[test]
fn erreur_typst_dans_le_template() {
    let sandbox = Sandbox::new();
    sandbox.write(
        "t/template.typ",
        "#let template(..args, body) = {\n  body\n  1 + \"a\"\n}\n#let callout(kind, body) = body\n",
    );
    sandbox.write("r.md", DOC);
    let (code, stderr) = sandbox.run(&["build", "r.md", "-t", "./t"]);
    assert_eq!(code, 1);
    assert!(
        stderr.starts_with("t/template.typ:3:3: erreur[typst] : cannot add integer and string"),
        "{stderr}"
    );
}

#[test]
fn avertissement_typst_avec_aide() {
    let sandbox = Sandbox::new();
    sandbox.write(
        "r.md",
        "# Titre\n\n```{=typst}\n#text(font: \"Police Inexistante\")[x]\n```\n",
    );
    let (code, stderr) = build(&sandbox, "r.md", &[]);
    assert_eq!(code, 0, "{stderr}");
    assert!(
        stderr.starts_with(
            "r.md:4:1: avertissement[typst] : unknown font family: police inexistante"
        ),
        "{stderr}"
    );
    assert!(sandbox.path("r.pdf").exists());
}

#[test]
fn invalid_utf8() {
    let sandbox = Sandbox::new();
    sandbox.write("r.md", b"# Titre\n\nJ\xe9r\xf4me\n");
    let (code, stderr) = build(&sandbox, "r.md", &[]);
    assert_eq!(code, 1);
    assert_eq!(
        stderr,
        "r.md:3:2: erreur[invalid-utf8] : le fichier n'est pas encodé en UTF-8 : caractère invalide\n"
    );
}

#[test]
fn unclosed_frontmatter() {
    let sandbox = Sandbox::new();
    sandbox.write("r.md", "+++\ntitle = \"x\"\n\n# Titre\n");
    let (code, stderr) = build(&sandbox, "r.md", &[]);
    assert_eq!(code, 1);
    assert!(
        stderr.starts_with("r.md:1:1: erreur[unclosed-frontmatter] : "),
        "{stderr}"
    );
}

#[test]
fn invalid_toml() {
    let sandbox = Sandbox::new();
    sandbox.write("r.md", "+++\ntitle = \"x\"\ntitle = \"y\"\n+++\n");
    let (code, stderr) = build(&sandbox, "r.md", &[]);
    assert_eq!(code, 1);
    assert!(
        stderr.starts_with("r.md:3:1: erreur[invalid-toml] : TOML invalide : "),
        "{stderr}"
    );
}

#[test]
fn unknown_key() {
    let sandbox = Sandbox::new();
    let fixture =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/unknown-key.md");
    sandbox.write("unknown-key.md", std::fs::read(fixture).unwrap());
    let (code, stderr) = build(&sandbox, "unknown-key.md", &[]);
    assert_eq!(code, 1);
    assert_eq!(
        stderr,
        "unknown-key.md:3:1: erreur[unknown-key] : clé inconnue « autors »\n"
    );
}

#[test]
fn invalid_type() {
    let sandbox = Sandbox::new();
    sandbox.write("r.md", "+++\ndate = 2026-10-10\n+++\n");
    let (code, stderr) = build(&sandbox, "r.md", &[]);
    assert_eq!(code, 1);
    assert!(
        stderr.starts_with("r.md:2:8: erreur[invalid-type] : « date » doit être une chaîne"),
        "{stderr}"
    );
}

#[test]
fn toutes_les_erreurs_de_configuration_en_une_fois() {
    let sandbox = Sandbox::new();
    sandbox.write(".combava/config.toml", "output = \"x.pdf\"\n");
    sandbox.write("r.md", "+++\nautors = []\ntoc = 1\n+++\n");
    let (code, stderr) = build(&sandbox, "r.md", &[]);
    assert_eq!(code, 1);
    assert_eq!(
        stderr,
        "r.md:2:1: erreur[unknown-key] : clé inconnue « autors »\n\
         r.md:3:7: erreur[invalid-type] : « toc » doit être un booléen (true ou false), pas un entier\n\
         .combava/config.toml:1:1: erreur[output-outside-frontmatter] : la clé « output » n'est permise que dans le frontmatter du document\n"
    );
}

#[test]
fn template_not_found() {
    let sandbox = Sandbox::new();
    sandbox.write(".combava/templates/.garde", "");
    sandbox.write("r.md", "+++\ntemplate = \"absent\"\n+++\n");
    let (code, stderr) = sandbox.run(&["build", "r.md"]);
    assert_eq!(code, 1);
    assert!(stderr.starts_with("r.md:2:12: erreur[template-not-found] : template « absent » introuvable ; emplacements essayés : « "), "{stderr}");
    assert!(stderr.contains(".combava/templates/absent"), "{stderr}");
    assert!(stderr.contains("les templates embarqués"), "{stderr}");

    // Un chemin sans template.typ.
    sandbox.write("vide/.garde", "");
    let (code, stderr) = sandbox.run(&["build", "r.md", "-t", "./vide"]);
    assert_eq!(code, 1);
    assert!(
        stderr.starts_with(
            "combava: erreur[template-not-found] : template introuvable : le dossier « "
        ),
        "{stderr}"
    );
    assert!(
        stderr.contains("ne contient pas de template.typ"),
        "{stderr}"
    );
}

#[test]
fn bibliography_not_found() {
    let sandbox = Sandbox::new();
    sandbox.write("refs.txt", "");
    sandbox.write("a.md", "+++\nbibliography = \"absent.bib\"\n+++\n");
    sandbox.write("b.md", "+++\nbibliography = \"refs.txt\"\n+++\n");
    let (code, stderr) = build(&sandbox, "a.md", &[]);
    assert_eq!(code, 1);
    assert_eq!(
        stderr,
        "a.md:2:16: erreur[bibliography-not-found] : fichier de bibliographie « absent.bib » introuvable\n"
    );
    let (code, stderr) = build(&sandbox, "b.md", &[]);
    assert_eq!(code, 1);
    assert_eq!(
        stderr,
        "b.md:2:16: erreur[bibliography-not-found] : le fichier de bibliographie « refs.txt » n'a pas l'extension .bib\n"
    );
}

#[test]
fn bibliographie_et_citations() {
    let sandbox = Sandbox::new();
    sandbox.write("refs.bib", "@book{knuth1984, author = {Knuth, Donald}, title = {The TeXbook}, year = {1984}, publisher = {AW}}\n");
    sandbox.write(
        "r.md",
        "+++\nbibliography = \"refs.bib\"\n+++\n\nVoir [@knuth1984].\n",
    );
    let (code, stderr) = build(&sandbox, "r.md", &[]);
    assert_eq!((code, stderr.as_str()), (0, ""));
}

#[test]
fn io() {
    let sandbox = Sandbox::new();
    let (code, stderr) = build(&sandbox, "absent.md", &[]);
    assert_eq!(code, 1);
    assert_eq!(
        stderr,
        "combava: erreur[io] : impossible de lire « absent.md » : fichier ou dossier introuvable\n"
    );

    sandbox.write("r.md", DOC);
    let (code, stderr) = build(&sandbox, "r.md", &["-o", "absent/r.pdf"]);
    assert_eq!(code, 1);
    assert_eq!(
        stderr,
        "combava: erreur[io] : le dossier « absent » du fichier de sortie n'existe pas\n"
    );

    let (code, stderr) = build(&sandbox, "r.md", &["-o", "r.md"]);
    assert_eq!(code, 1);
    assert!(
        stderr.contains("est le fichier markdown lui-même"),
        "{stderr}"
    );
    assert_eq!(sandbox.read("r.md"), DOC);
}

#[test]
fn chemin_hors_du_dossier_courant() {
    let sandbox = Sandbox::new();
    sandbox.write("docs/r.md", "Texte <b>x</b>\n");
    let docs = sandbox.path("docs");
    let mut command = sandbox.combava(&["build", "../docs/r.md", "--transpile-only"]);
    sandbox.write("autre/.garde", "");
    let output = command.current_dir(sandbox.path("autre")).output().unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.starts_with(&format!(
            "{}:1:7: avertissement[raw-html]",
            docs.join("r.md").display()
        )),
        "{stderr}"
    );
}

#[test]
fn mauvaise_utilisation() {
    let sandbox = Sandbox::new();
    assert_eq!(sandbox.run(&["build"]).0, 2);
    assert_eq!(sandbox.run(&["inconnue"]).0, 2);
    assert_eq!(sandbox.run(&["build", "r.md", "--option-inconnue"]).0, 2);
    assert_eq!(sandbox.run(&[]).0, 2);
}

#[test]
fn aide_et_version() {
    let sandbox = Sandbox::new();
    for args in [
        &["--help"][..],
        &["help"],
        &["help", "build"],
        &["build", "--help"],
        &["init", "-h"],
    ] {
        let output = sandbox.combava(args).output().unwrap();
        assert_eq!(output.status.code(), Some(0), "{args:?}");
        assert!(
            String::from_utf8(output.stdout)
                .unwrap()
                .contains("Utilisation : combava")
        );
    }
    let output = sandbox.combava(&["--version"]).output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("combava {}\n", env!("CARGO_PKG_VERSION"))
    );
    assert_eq!(sandbox.run(&["help", "inconnue"]).0, 2);
}

#[test]
fn package_download() {
    // Sous Linux : caches de packages vides et proxy injoignable, pour simuler
    // un premier usage des maths sans connexion.
    if !cfg!(target_os = "linux") {
        return;
    }
    let sandbox = Sandbox::new();
    sandbox.write("r.md", "Une formule $x$.\n");
    let template = minimal_template();
    let output = sandbox
        .combava(&["build", "r.md", "-t", arg(&template)])
        .env("XDG_CACHE_HOME", sandbox.path("cache"))
        .env("XDG_DATA_HOME", sandbox.path("data"))
        .env("HTTPS_PROXY", "http://127.0.0.1:9")
        .env("https_proxy", "http://127.0.0.1:9")
        .output()
        .unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        stderr.starts_with(
            "combava: erreur[package-download] : impossible de télécharger le package « @preview/mitex:0.2.7 » : "
        ),
        "{stderr}"
    );
    assert!(stderr.ends_with(
        "\n  aide : une connexion est nécessaire au premier usage des maths ; le package est ensuite gardé en cache\n"
    ));
    assert_eq!(stderr.lines().count(), 2, "{stderr}");
    assert!(!sandbox.path("r.pdf").exists());
}
