//! Les commandes `build` et `init` (spécification, sections 5.1 et 5.2).

pub mod build;
pub mod init;

use std::io::Write;
use std::path::Path;

use crate::diagnostics::{Diagnostic, display_path};
use crate::error::{Code, io_message};

/// Écrit `data` dans `path` sans jamais laisser de fichier à moitié écrit :
/// un fichier temporaire du même dossier remplace la cible une fois complet.
pub fn write_atomically(path: &Path, data: &[u8], cwd: &Path) -> Result<(), Diagnostic> {
    let error = |error: std::io::Error| {
        Diagnostic::error(
            Code::Io,
            format!(
                "impossible d'écrire « {} » : {}",
                display_path(path, cwd),
                io_message(&error)
            ),
        )
    };
    let dir = path.parent().unwrap_or(Path::new("."));
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let temporary = dir.join(format!(".{name}.{}.tmp", std::process::id()));
    let result = std::fs::File::create(&temporary)
        .and_then(|mut file| {
            file.write_all(data)?;
            file.sync_all()
        })
        .and_then(|()| std::fs::rename(&temporary, path));
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result.map_err(error)
}
