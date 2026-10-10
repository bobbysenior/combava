//! Définition de la ligne de commande (spécification, section 5).
//!
//! Les textes générés par `clap` sont remplacés par des textes en français :
//! drapeaux `--help` et `--version`, sous-commande `help`, titres de l'aide.

use std::path::PathBuf;

use clap::{ArgAction, Args, Parser, Subcommand};

const HELP_TEMPLATE: &str = "\
{about-with-newline}
Utilisation : {usage}

{all-args}{after-help}";

#[derive(Debug, Parser)]
#[command(
    name = "combava",
    version,
    about = "Transforme un fichier markdown en rapport PDF, via Typst",
    help_template = HELP_TEMPLATE,
    subcommand_help_heading = "Commandes",
    subcommand_value_name = "COMMANDE",
    disable_help_flag = true,
    disable_version_flag = true,
    disable_help_subcommand = true,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Affiche l'aide
    #[arg(short, long, action = ArgAction::Help, global = true)]
    help: Option<bool>,

    /// Affiche la version
    #[arg(short = 'V', long, action = ArgAction::Version)]
    version: Option<bool>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Compile un fichier markdown en PDF
    #[command(help_template = HELP_TEMPLATE, disable_help_flag = true)]
    Build(BuildArgs),

    /// Crée un fichier markdown prêt à remplir
    #[command(help_template = HELP_TEMPLATE, disable_help_flag = true)]
    Init(InitArgs),

    /// Affiche l'aide de combava ou d'une commande
    #[command(help_template = HELP_TEMPLATE, disable_help_flag = true)]
    Help {
        /// La commande dont afficher l'aide
        #[arg(value_name = "COMMANDE")]
        command: Option<String>,
    },
}

#[derive(Debug, Args)]
pub struct BuildArgs {
    /// Le fichier markdown à compiler
    #[arg(value_name = "FILE")]
    pub file: PathBuf,

    /// Le fichier de sortie (par défaut : le nom du fichier markdown, en .pdf)
    #[arg(short, long, value_name = "FILE")]
    pub out: Option<PathBuf>,

    /// Le template : un nom, ou le chemin d'un dossier contenant template.typ
    #[arg(short, long, value_name = "TEMPLATE")]
    pub template: Option<String>,

    /// Écrit le code Typst généré (.typ) au lieu du PDF, pour le débogage
    #[arg(long)]
    pub transpile_only: bool,
}

#[derive(Debug, Args)]
pub struct InitArgs {
    /// Le fichier à créer
    #[arg(value_name = "FILE", default_value = "rapport.md")]
    pub file: PathBuf,

    /// Ajoute la clé template au frontmatter
    #[arg(short, long, value_name = "TEMPLATE")]
    pub template: Option<String>,

    /// Écrase le fichier s'il existe déjà
    #[arg(long)]
    pub force: bool,
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use super::*;

    #[test]
    fn definition_valide() {
        Cli::command().debug_assert();
    }
}
