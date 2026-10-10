//! Outils partagés par les tests d'intégration du CLI.

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use assert_cmd::Command;
use tempfile::TempDir;

/// Racine du dépôt.
pub fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Template de test conforme à la section 9 de la spécification.
pub fn minimal_template() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/templates/minimal")
}

/// Un dossier temporaire dans lequel lancer `combava`.
pub struct Sandbox {
    dir: TempDir,
}

impl Sandbox {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join("config-globale")).unwrap();
        Self { dir }
    }

    pub fn path(&self, relative: &str) -> PathBuf {
        self.dir.path().join(relative)
    }

    /// Crée `relative` et ses dossiers parents.
    pub fn write(&self, relative: &str, content: impl AsRef<[u8]>) -> PathBuf {
        let path = self.path(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, content).unwrap();
        path
    }

    pub fn read(&self, relative: &str) -> String {
        std::fs::read_to_string(self.path(relative)).unwrap()
    }

    /// Le dossier de configuration globale vu par `combava` sous Linux.
    pub fn global_config(&self) -> PathBuf {
        self.path("config-globale/combava")
    }

    /// `combava <args>`, lancé dans le dossier temporaire.
    ///
    /// Sous Linux, la configuration globale de la machine est remplacée par
    /// celle du dossier temporaire, vide par défaut.
    pub fn combava(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_combava"));
        command
            .args(args)
            .current_dir(self.dir.path())
            .env("XDG_CONFIG_HOME", self.path("config-globale"));
        command
    }

    /// Lance `combava` et renvoie son code de sortie et sa sortie d'erreur.
    pub fn run(&self, args: &[&str]) -> (i32, String) {
        let output = self.combava(args).output().unwrap();
        let stderr = String::from_utf8(output.stderr).unwrap();
        (output.status.code().unwrap(), stderr)
    }
}

/// Le chemin d'un dossier, sous forme de chaîne pour les arguments.
pub fn arg(path: &Path) -> &str {
    path.to_str().unwrap()
}

/// Les lignes du bloc `#show: template.with(…)` du code généré, sans
/// l'indentation : `title: "…",`, `authors: (…),`…
pub fn template_arguments(typst: &str) -> Vec<String> {
    let start = typst.find("#show: template.with(\n").unwrap();
    typst[start..]
        .lines()
        .skip(1)
        .take_while(|line| *line != ")")
        .map(|line| line.trim().to_string())
        .collect()
}
