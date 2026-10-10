//! Chemins des images (spécification, section 4.4).

use crate::escape::percent_decode;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ImagePathError {
    /// Destination `http://` ou `https://`.
    Remote,
    /// Chemin absolu, vide, avec un schéma, ou qui sort du dossier du `.md`.
    Invalid,
}

/// Convertit la destination d'une image en chemin Typst relatif à la racine,
/// avec un `/` de tête et sans `..`.
pub(crate) fn resolve_image(dest: &str) -> Result<String, ImagePathError> {
    let dest = dest.trim();
    let lower = dest.to_ascii_lowercase();
    if lower.starts_with("http://") || lower.starts_with("https://") {
        return Err(ImagePathError::Remote);
    }

    let decoded = percent_decode(dest);
    if decoded.starts_with(['/', '\\']) || has_scheme(&decoded) {
        return Err(ImagePathError::Invalid);
    }

    let mut segments = Vec::new();
    for segment in decoded.split('/') {
        match segment {
            "" | "." => {}
            ".." => {
                segments.pop().ok_or(ImagePathError::Invalid)?;
            }
            segment => segments.push(segment),
        }
    }
    if segments.is_empty() {
        return Err(ImagePathError::Invalid);
    }
    Ok(format!("/{}", segments.join("/")))
}

/// Vrai pour `C:…`, `file:…`, `data:…` : des lettres puis `:` avant tout `/`.
fn has_scheme(path: &str) -> bool {
    match path.split_once(':') {
        Some((scheme, _)) => !scheme.is_empty() && scheme.chars().all(|c| c.is_ascii_alphabetic()),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chemins_relatifs_normalises() {
        assert_eq!(resolve_image("img.png").unwrap(), "/img.png");
        assert_eq!(resolve_image("./images/a.png").unwrap(), "/images/a.png");
        assert_eq!(resolve_image("a/../b/./c.png").unwrap(), "/b/c.png");
        assert_eq!(resolve_image("mon%20image.png").unwrap(), "/mon image.png");
    }

    #[test]
    fn images_distantes() {
        assert_eq!(
            resolve_image("https://x.fr/a.png"),
            Err(ImagePathError::Remote)
        );
        assert_eq!(
            resolve_image("HTTP://x.fr/a.png"),
            Err(ImagePathError::Remote)
        );
    }

    #[test]
    fn chemins_interdits() {
        for dest in [
            "/etc/a.png",
            "\\a.png",
            "../a.png",
            "a/../../b.png",
            "C:/a.png",
            "file:a.png",
            "",
            ".",
            "a/..",
        ] {
            assert_eq!(resolve_image(dest), Err(ImagePathError::Invalid), "{dest}");
        }
    }
}
