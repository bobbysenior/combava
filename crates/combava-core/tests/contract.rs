//! Garanties de l'API envers le CLI (spécification, sections 2.1 à 2.3).

use combava_core::{Config, MITEX_PACKAGE, paths, transpile};

fn typst(markdown: &str, config: &Config) -> String {
    transpile(markdown, config).unwrap().typst
}

#[test]
fn constantes_partagees() {
    assert_eq!(paths::MAIN, "/__combava__/main.typ");
    assert_eq!(paths::TEMPLATE_DIR, "/__combava__/template");
    assert_eq!(paths::TEMPLATE_ENTRY, "/__combava__/template/template.typ");
    assert_eq!(paths::BIBLIOGRAPHY, "/__combava__/bibliography.bib");
    assert_eq!(MITEX_PACKAGE, "@preview/mitex:0.2.7");
}

#[test]
fn config_par_defaut() {
    let config = Config::default();
    assert!(config.toc);
    assert!(!config.list_of_figures && !config.list_of_listings && !config.bibliography);
    assert!(config.title.is_none() && config.authors.is_empty());
}

#[test]
fn document_vide() {
    let out = typst("", &Config::default());
    assert!(out.starts_with("// Généré par combava-core "));
    assert!(out.ends_with(")\n"));
    assert!(out.contains("#import \"/__combava__/template/template.typ\": template, callout\n"));
}

#[test]
fn structure_du_code_genere() {
    let config = Config {
        bibliography: true,
        ..Config::default()
    };
    let out = typst("Texte $x$.", &config);
    let mitex = out
        .find("#import \"@preview/mitex:0.2.7\": mi, mitex")
        .unwrap();
    let template = out
        .find("#import \"/__combava__/template/template.typ\"")
        .unwrap();
    let show = out.find("#show: template.with(").unwrap();
    let body = out.find("Texte #mi(\"x\")").unwrap();
    let bibliography = out
        .find("#bibliography(\"/__combava__/bibliography.bib\")\n")
        .unwrap();
    assert!(mitex < template && template < show && show < body && body < bibliography);
    assert!(out.ends_with("\n"));

    let out = typst("Texte.", &Config::default());
    assert!(!out.contains("mitex"));
    assert!(!out.contains("#bibliography"));
}

#[test]
fn fins_de_ligne_lf_uniquement() {
    let markdown =
        "# Titre\r\n\r\nLigne\r\nsuite\r\n\r\n```{=typst}\r\n#let a = 1\r#let b = 2\r\n```\r\n";
    let out = typst(markdown, &Config::default());
    assert!(!out.contains('\r'));
    assert!(out.contains("#let a = 1\n#let b = 2"));
    assert!(out.ends_with('\n'));
}

#[test]
fn deterministe() {
    let markdown = "# A\n\nB[^1] [[A]]\n\n[^1]: n\n";
    assert_eq!(
        typst(markdown, &Config::default()),
        typst(markdown, &Config::default())
    );
}

#[test]
fn aucune_injection_hors_bloc_typst() {
    let markdown = "#import \"x.typ\": *\n\n`#eval(\"1\")` *#strong[x]* [lien](\"x\")\n";
    let out = typst(markdown, &Config::default());
    let body = &out[out.find(")\n\n").unwrap()..];
    assert!(body.contains(r"\#import"));
    // Hors chaîne et une fois les `\#` retirés, il ne reste aucun appel venu du texte.
    let unescaped = body.replace(r"\#", "").replace(r##""#eval(\"1\")""##, "");
    assert!(!unescaped.contains("#import"));
    assert!(!unescaped.contains("#eval"));
    assert!(body.contains(r##"#raw("#eval(\"1\")")"##));
}

#[test]
fn valeurs_de_config_echappees() {
    let config = Config {
        title: Some("Le \"titre\" \\ #x".into()),
        authors: vec!["Seule".into()],
        ..Config::default()
    };
    let out = typst("", &config);
    assert!(out.contains(r#"  title: "Le \"titre\" \\ #x","#));
    assert!(out.contains(r#"  authors: ("Seule",),"#));
    assert!(out.contains(r#"  header-text: "Le \"titre\" \\ #x","#));
}

#[test]
fn correspondance_des_lignes() {
    let markdown =
        "# Titre\n\nParagraphe\nsur deux lignes.\n\n```{=typst}\n#let a = 1\n\n#let b = 2\n```\n";
    let output = transpile(markdown, &Config::default()).unwrap();
    let lines: Vec<_> = output.typst.lines().collect();
    let line_of = |needle: &str| lines.iter().position(|l| l.contains(needle)).unwrap() + 1;
    let map = &output.source_map;

    assert_eq!(map.markdown_line(1), None);
    assert_eq!(map.markdown_line(line_of("#show: template.with(")), None);
    assert_eq!(map.markdown_line(line_of("#heading")), Some(1));
    assert_eq!(
        map.markdown_line(line_of("Paragraphe sur deux lignes.")),
        Some(3)
    );
    assert_eq!(map.markdown_line(line_of("#let a = 1")), Some(7));
    assert_eq!(map.markdown_line(line_of("#let a = 1") + 1), Some(8));
    assert_eq!(map.markdown_line(line_of("#let b = 2")), Some(9));
    // Ligne de séparation entre deux blocs.
    assert_eq!(map.markdown_line(line_of("#heading") + 1), None);
    assert_eq!(map.markdown_line(0), None);
    assert_eq!(map.markdown_line(10_000), None);
}

#[test]
fn frontmatter_remplace_par_des_espaces() {
    // Ce que le CLI transmet : chaque octet du frontmatter devient une espace, sauf `\n`.
    let file = "+++\ntitle = \"é\"\n+++\n\n<b>x</b>\n";
    let blanked: String = file
        .char_indices()
        .map(|(i, c)| {
            if i < 20 && c != '\n' {
                " ".repeat(c.len_utf8())
            } else {
                c.to_string()
            }
        })
        .collect();
    assert_eq!(blanked.len(), file.len());
    let output = transpile(&blanked, &Config::default()).unwrap();
    let html = &output.diagnostics[0];
    assert_eq!((html.line, html.column), (5, 1));
    assert_eq!(&file[html.span.clone()], "<b>");
}
