//! Fusion des couches de configuration et résolution des chemins
//! (spécification, sections 6.3 et 6.4), dans des dossiers temporaires.

mod common;

use std::path::PathBuf;

use combava_cli::config::paths::Env;
use combava_cli::config::{self, Overrides, Settings};
use combava_cli::diagnostics::Diagnostic;
use combava_cli::frontmatter;
use combava_cli::template::Template;
use tempfile::TempDir;

/// Un projet : `projet/.combava/`, un document dans `projet/docs/`, une
/// configuration globale et un dossier personnel.
struct Fixture {
    dir: TempDir,
}

impl Fixture {
    fn new() -> Self {
        let fixture = Self {
            dir: tempfile::tempdir().unwrap(),
        };
        for dir in [
            "projet/.combava",
            "projet/docs",
            "globale",
            "maison",
            "courant",
        ] {
            std::fs::create_dir_all(fixture.path(dir)).unwrap();
        }
        fixture
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.dir.path().join(relative)
    }

    fn write(&self, relative: &str, content: &str) {
        let path = self.path(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    fn env(&self) -> Env {
        Env {
            cwd: self.path("courant"),
            home: Some(self.path("maison")),
            global_config: Some(self.path("globale")),
        }
    }

    fn load_with(
        &self,
        markdown: &str,
        overrides: &Overrides,
    ) -> Result<Settings, Vec<Diagnostic>> {
        let path = self.path("projet/docs/r.md");
        std::fs::write(&path, markdown).unwrap();
        let document = frontmatter::split(markdown, &path).unwrap();
        config::load(
            &path,
            markdown,
            document.frontmatter,
            overrides,
            &self.env(),
        )
    }

    fn load(&self, markdown: &str) -> Settings {
        self.load_with(markdown, &Overrides::default()).unwrap()
    }
}

#[test]
fn defauts() {
    let fixture = Fixture::new();
    let settings = fixture.load("# Titre\n");
    assert_eq!(settings.config, combava_core::Config::default());
    assert!(matches!(settings.template, Template::Embedded { ref name, .. } if name == "default"));
    assert_eq!(settings.bibliography, None);
    assert_eq!(settings.output, fixture.path("projet/docs/r.pdf"));
}

#[test]
fn priorite_des_couches() {
    let fixture = Fixture::new();
    fixture.write(
        "globale/config.toml",
        "title = \"globale\"\nsubtitle = \"globale\"\ndate = \"globale\"\nschool = \"globale\"\ntoc = false\n",
    );
    fixture.write(
        "projet/.combava/config.toml",
        "title = \"projet\"\nsubtitle = \"projet\"\nauthors = [\"Projet\"]\nlist_of_figures = true\n",
    );
    let settings = fixture.load("+++\ntitle = \"frontmatter\"\nauthors = []\n+++\n");
    let config = settings.config;
    assert_eq!(config.title.as_deref(), Some("frontmatter"));
    assert_eq!(config.subtitle.as_deref(), Some("projet"));
    assert_eq!(config.date.as_deref(), Some("globale"));
    assert_eq!(config.school.as_deref(), Some("globale"));
    // Une liste vide dans le frontmatter remplace celle du projet.
    assert!(config.authors.is_empty());
    assert!(!config.toc);
    assert!(config.list_of_figures);
    // Absent partout : laissé à `None` pour que le core mette le titre.
    assert_eq!(config.header_text, None);
}

#[test]
fn arguments_prioritaires() {
    let fixture = Fixture::new();
    fixture.write("projet/.combava/templates/projet/template.typ", "");
    fixture.write("courant/arg/template.typ", "");
    let markdown = "+++\ntemplate = \"projet\"\noutput = \"fm.pdf\"\n+++\n";
    assert_eq!(
        fixture.load(markdown).output,
        fixture.path("projet/docs/fm.pdf")
    );

    let overrides = Overrides {
        template: Some("./arg".into()),
        output: Some("sortie.pdf".into()),
    };
    let settings = fixture.load_with(markdown, &overrides).unwrap();
    // Les arguments sont relatifs au dossier courant.
    assert_eq!(settings.output, fixture.path("courant/sortie.pdf"));
    assert!(
        matches!(settings.template, Template::Disk(ref dir) if *dir == fixture.path("courant/arg"))
    );
}

#[test]
fn chemins_relatifs_a_leur_couche() {
    let fixture = Fixture::new();
    for bib in [
        "projet/docs/fm.bib",
        "projet/projet.bib",
        "globale/globale.bib",
        "maison/maison.bib",
    ] {
        fixture.write(bib, "");
    }
    fixture.write("globale/config.toml", "bibliography = \"globale.bib\"\n");
    assert_eq!(
        fixture.load("").bibliography,
        Some(fixture.path("globale/globale.bib"))
    );

    fixture.write(
        "projet/.combava/config.toml",
        "bibliography = \"projet.bib\"\n",
    );
    assert_eq!(
        fixture.load("").bibliography,
        Some(fixture.path("projet/projet.bib"))
    );

    let settings = fixture.load("+++\nbibliography = \"fm.bib\"\n+++\n");
    assert_eq!(
        settings.bibliography,
        Some(fixture.path("projet/docs/fm.bib"))
    );
    assert!(settings.config.bibliography);

    let settings = fixture.load("+++\nbibliography = \"~/maison.bib\"\n+++\n");
    assert_eq!(
        settings.bibliography,
        Some(fixture.path("maison/maison.bib"))
    );

    let settings = fixture.load("+++\nbibliography = \"../projet.bib\"\n+++\n");
    assert_eq!(
        settings.bibliography,
        Some(fixture.path("projet/projet.bib"))
    );
}

#[test]
fn racine_du_projet_en_remontant() {
    let fixture = Fixture::new();
    fixture.write("projet/.combava/config.toml", "title = \"projet\"\n");
    let path = fixture.path("projet/docs/a/b/r.md");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let settings = config::load(&path, "", None, &Overrides::default(), &fixture.env()).unwrap();
    assert_eq!(settings.config.title.as_deref(), Some("projet"));

    // Sans dossier `.combava/` au-dessus du document : pas de couche projet.
    let path = fixture.path("ailleurs/r.md");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let settings = config::load(&path, "", None, &Overrides::default(), &fixture.env()).unwrap();
    assert_eq!(settings.config.title, None);
}

#[test]
fn sans_configuration_globale() {
    let fixture = Fixture::new();
    let env = Env {
        global_config: None,
        ..fixture.env()
    };
    let path = fixture.path("projet/docs/r.md");
    assert!(config::load(&path, "", None, &Overrides::default(), &env).is_ok());
}

#[test]
fn erreurs_de_toutes_les_couches() {
    let fixture = Fixture::new();
    fixture.write("globale/config.toml", "inconnue = 1\n");
    fixture.write("projet/.combava/config.toml", "output = \"x.pdf\"\n");
    let errors = fixture
        .load_with("+++\ntoc = \"oui\"\n+++\n", &Overrides::default())
        .unwrap_err();
    let codes: Vec<_> = errors.iter().map(|d| d.code).collect();
    assert_eq!(
        codes,
        ["invalid-type", "output-outside-frontmatter", "unknown-key"]
    );
}

#[test]
fn erreurs_de_resolution_en_une_fois() {
    let fixture = Fixture::new();
    let markdown = "+++\ntemplate = \"absent\"\nbibliography = \"absent.bib\"\noutput = \"absent/r.pdf\"\n+++\n";
    let errors = fixture
        .load_with(markdown, &Overrides::default())
        .unwrap_err();
    let codes: Vec<_> = errors.iter().map(|d| d.code).collect();
    assert_eq!(
        codes,
        ["template-not-found", "bibliography-not-found", "io"]
    );
}

#[test]
fn configuration_globale_du_systeme() {
    // Sous Linux, la configuration globale suit `XDG_CONFIG_HOME`.
    if !cfg!(target_os = "linux") {
        return;
    }
    let sandbox = common::Sandbox::new();
    std::fs::create_dir_all(sandbox.global_config()).unwrap();
    sandbox.write(
        "config-globale/combava/config.toml",
        "title = \"Titre global\"\n",
    );
    sandbox.write("r.md", "Texte.\n");
    let (code, stderr) = sandbox.run(&["build", "r.md", "--transpile-only"]);
    assert_eq!((code, stderr.as_str()), (0, ""));
    let arguments = common::template_arguments(&sandbox.read("r.typ"));
    assert!(
        arguments.contains(&"title: \"Titre global\",".to_string()),
        "{arguments:?}"
    );
}

#[test]
fn config_transmise_au_core() {
    let sandbox = common::Sandbox::new();
    sandbox.write(
        ".combava/config.toml",
        "school = \"École\"\nauthors = [\"A\", \"B\"]\n",
    );
    sandbox.write(
        "r.md",
        "+++\ntitle = \"T\"\nteachers = [\"P\"]\ntoc = false\n+++\n",
    );
    assert_eq!(sandbox.run(&["build", "r.md", "--transpile-only"]).0, 0);
    let arguments = common::template_arguments(&sandbox.read("r.typ"));
    for expected in [
        "title: \"T\",",
        "authors: (\"A\", \"B\"),",
        "teachers: (\"P\",),",
        "school: \"École\",",
        "toc: false,",
        "header-text: \"T\",",
    ] {
        assert!(
            arguments.iter().any(|a| a == expected),
            "{expected} absent de {arguments:?}"
        );
    }
}
