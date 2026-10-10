//! Le binaire `combava` : configuration, accès aux fichiers et compilation
//! PDF autour de `combava-core`.
//!
//! La bibliothèque n'existe que pour les tests d'intégration : son API n'est
//! pas stable.

pub mod args;
pub mod commands;
pub mod compile;
pub mod config;
pub mod diagnostics;
pub mod error;
pub mod frontmatter;
pub mod template;

use std::process::ExitCode;

use clap::{CommandFactory, Parser};

use args::{Cli, Command};
use config::paths::Env;
use diagnostics::{Diagnostic, Reporter};
use error::{Code, io_message};

/// Exécute la ligne de commande du processus. Code de sortie : 0 en cas de
/// succès, 1 en cas d'erreur, 2 pour une mauvaise utilisation (section 5.5).
pub fn run() -> ExitCode {
    let cli = Cli::parse();
    let env = match Env::from_system() {
        Ok(env) => env,
        Err(error) => {
            let message = format!("dossier courant inaccessible : {}", io_message(&error));
            let mut reporter = Reporter::new(Default::default());
            reporter.report(&Diagnostic::error(Code::Io, message));
            return ExitCode::FAILURE;
        }
    };
    let mut reporter = Reporter::new(env.cwd.clone());
    let result = match &cli.command {
        Command::Build(args) => commands::build::run(args, &env, &mut reporter),
        Command::Init(args) => commands::init::run(args, &env, &mut reporter),
        Command::Help { command } => {
            help(command.as_deref());
            Ok(())
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error::Failed) => ExitCode::FAILURE,
    }
}

/// `combava help [COMMANDE]`.
fn help(command: Option<&str>) {
    let mut cli = Cli::command();
    cli.build();
    let result = match command {
        None => cli.print_help(),
        Some(name) => match cli.find_subcommand_mut(name) {
            Some(sub) => sub.print_help(),
            None => cli
                .error(
                    clap::error::ErrorKind::InvalidSubcommand,
                    format!("commande inconnue « {name} »"),
                )
                .exit(),
        },
    };
    // Une sortie standard fermée n'est pas une erreur de combava.
    let _ = result;
}
