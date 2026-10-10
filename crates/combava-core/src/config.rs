/// Configuration d'un document, déjà fusionnée par le CLI.
///
/// Les champs correspondent aux clés TOML de la section 6.2 de la spécification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub authors: Vec<String>,
    pub teachers: Vec<String>,
    pub date: Option<String>,
    pub school: Option<String>,
    pub university: Option<String>,
    pub academic_year: Option<String>,
    pub cohort: Option<String>,
    pub specialization: Option<String>,
    pub subject: Option<String>,
    pub toc: bool,
    pub list_of_figures: bool,
    pub list_of_listings: bool,
    /// Texte en haut de page ; `None` est remplacé par `title`.
    pub header_text: Option<String>,
    /// Vrai si le CLI monte un fichier `.bib` à `paths::BIBLIOGRAPHY`.
    pub bibliography: bool,
}

/// `toc` vaut `true` ; tous les autres champs sont `None`, vides ou `false`.
impl Default for Config {
    fn default() -> Self {
        Config {
            title: None,
            subtitle: None,
            authors: Vec::new(),
            teachers: Vec::new(),
            date: None,
            school: None,
            university: None,
            academic_year: None,
            cohort: None,
            specialization: None,
            subject: None,
            toc: true,
            list_of_figures: false,
            list_of_listings: false,
            header_text: None,
            bibliography: false,
        }
    }
}
