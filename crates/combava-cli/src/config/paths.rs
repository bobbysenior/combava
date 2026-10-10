//! Racine du projet, dossier de configuration globale et résolution des
//! chemins (spécification, sections 6.3 et 6.4).

use std::path::{Component, Path, PathBuf};

/// Les dossiers qui dépendent de l'environnement, injectés pour les tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Env {
    /// Dossier courant, absolu.
    pub cwd: PathBuf,
    /// Dossier personnel, pour `~`.
    pub home: Option<PathBuf>,
    /// Dossier de configuration globale (`~/.config/combava` sous Linux).
    pub global_config: Option<PathBuf>,
}

impl Env {
    /// L'environnement du processus.
    pub fn from_system() -> std::io::Result<Self> {
        Ok(Self {
            cwd: std::env::current_dir()?,
            home: directories::BaseDirs::new().map(|dirs| dirs.home_dir().to_path_buf()),
            global_config: directories::ProjectDirs::from("", "", "combava")
                .map(|dirs| dirs.config_dir().to_path_buf()),
        })
    }

    /// Un chemin donné en argument : relatif au dossier courant.
    pub fn argument(&self, value: &Path) -> PathBuf {
        normalize(&self.cwd.join(value))
    }

    /// Une valeur de configuration : `~` en tête désigne le dossier personnel,
    /// un chemin relatif l'est à `base`.
    pub fn config_path(&self, value: &str, base: &Path) -> PathBuf {
        let expanded = match (strip_tilde(value), &self.home) {
            (Some(rest), Some(home)) => home.join(rest),
            _ => PathBuf::from(value),
        };
        normalize(&base.join(expanded))
    }
}

/// `~` seul ou suivi d'un séparateur : le reste du chemin.
fn strip_tilde(value: &str) -> Option<&str> {
    let rest = value.strip_prefix('~')?;
    if rest.is_empty() {
        Some("")
    } else {
        rest.strip_prefix(['/', '\\'])
    }
}

/// Supprime les `.` et résout les `..` sans accéder au disque.
pub fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            other => out.push(other),
        }
    }
    out
}

/// Premier dossier contenant un dossier `.combava/`, en remontant depuis
/// `start`.
pub fn project_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|dir| dir.join(".combava").is_dir())
        .map(Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env() -> Env {
        Env {
            cwd: "/courant".into(),
            home: Some("/home/ada".into()),
            global_config: Some("/home/ada/.config/combava".into()),
        }
    }

    #[test]
    fn chemins_de_configuration() {
        let env = env();
        let base = Path::new("/projet/docs");
        assert_eq!(
            env.config_path("refs.bib", base),
            Path::new("/projet/docs/refs.bib")
        );
        assert_eq!(
            env.config_path("../refs.bib", base),
            Path::new("/projet/refs.bib")
        );
        assert_eq!(
            env.config_path("./a/./b", base),
            Path::new("/projet/docs/a/b")
        );
        assert_eq!(env.config_path("/abs/x.bib", base), Path::new("/abs/x.bib"));
        assert_eq!(
            env.config_path("~/refs.bib", base),
            Path::new("/home/ada/refs.bib")
        );
        assert_eq!(env.config_path("~", base), Path::new("/home/ada"));
        // `~ada` n'est pas le dossier personnel.
        assert_eq!(
            env.config_path("~ada/x", base),
            Path::new("/projet/docs/~ada/x")
        );
    }

    #[test]
    fn arguments() {
        assert_eq!(env().argument(Path::new("../a.pdf")), Path::new("/a.pdf"));
        assert_eq!(env().argument(Path::new("/x/a.pdf")), Path::new("/x/a.pdf"));
    }

    #[test]
    fn sans_dossier_personnel() {
        let env = Env {
            home: None,
            ..env()
        };
        assert_eq!(env.config_path("~/x", Path::new("/b")), Path::new("/b/~/x"));
    }

    #[test]
    fn racine_du_projet() {
        let dir = tempfile::tempdir().unwrap();
        let docs = dir.path().join("a/b");
        std::fs::create_dir_all(&docs).unwrap();
        assert_eq!(project_root(&docs), None);
        // Un fichier `.combava` n'est pas un dossier.
        std::fs::write(docs.join(".combava"), "").unwrap();
        assert_eq!(project_root(&docs), None);
        std::fs::create_dir(dir.path().join("a/.combava")).unwrap();
        assert_eq!(project_root(&docs), Some(dir.path().join("a")));
    }
}
