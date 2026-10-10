//! `combava build` (spécification, section 5.1).

use std::path::Path;

use crate::args::BuildArgs;
use crate::compile::{self, Job};
use crate::config::paths::Env;
use crate::config::{self, Overrides};
use crate::diagnostics::{Diagnostic, Location, Reporter, display_path};
use crate::error::{Code, Failed, io_message};
use crate::frontmatter;

pub fn run(args: &BuildArgs, env: &Env, reporter: &mut Reporter) -> Result<(), Failed> {
    // 1) Lecture.
    let markdown = env.argument(&args.file);
    let text = read_utf8(&markdown, env).map_err(|d| reporter.fail(d))?;

    // 2) Frontmatter remplacé par des espaces.
    let document = frontmatter::split(&text, &markdown).map_err(|d| reporter.fail(d))?;

    // 3) et 4) Couches de configuration, template, bibliographie, sortie.
    let overrides = Overrides {
        template: args.template.clone(),
        output: args.out.clone(),
    };
    let settings = config::load(&markdown, &text, document.frontmatter, &overrides, env).map_err(
        |diagnostics| {
            reporter.report_all(&diagnostics);
            Failed
        },
    )?;
    let output = if args.transpile_only {
        settings.output.with_extension("typ")
    } else {
        settings.output.clone()
    };
    if output == markdown {
        return Err(reporter.fail(Diagnostic::error(
            Code::Io,
            format!(
                "le fichier de sortie « {} » est le fichier markdown lui-même",
                display_path(&output, &env.cwd)
            ),
        )));
    }

    // 5) et 6) Transpilation.
    let transpiled = match combava_core::transpile(&document.markdown, &settings.config) {
        Ok(transpiled) => transpiled,
        Err(error) => {
            let diagnostics: Vec<_> = error
                .diagnostics
                .iter()
                .map(|d| Diagnostic::from_core(d, &markdown))
                .collect();
            return reporter.check(&diagnostics).and(Err(Failed));
        }
    };
    let warnings: Vec<_> = transpiled
        .diagnostics
        .iter()
        .map(|d| Diagnostic::from_core(d, &markdown))
        .collect();
    reporter.check(&warnings)?;

    // 7) Code Typst seul.
    if args.transpile_only {
        return write(&output, transpiled.typst.as_bytes(), env, reporter);
    }

    // 8) Compilation.
    let compiled = compile::compile(Job {
        typst: transpiled.typst,
        source_map: &transpiled.source_map,
        markdown: &markdown,
        template: settings.template,
        bibliography: settings.bibliography,
    });
    reporter.check(&compiled.diagnostics)?;

    // 9) Écriture du PDF.
    let pdf = compiled
        .pdf
        .expect("une compilation sans erreur produit un PDF");
    write(&output, &pdf, env, reporter)
}

/// Lit le fichier markdown, qui doit être en UTF-8.
fn read_utf8(path: &Path, env: &Env) -> Result<String, Diagnostic> {
    let bytes = std::fs::read(path).map_err(|error| {
        Diagnostic::error(
            Code::Io,
            format!(
                "impossible de lire « {} » : {}",
                display_path(path, &env.cwd),
                io_message(&error)
            ),
        )
    })?;
    String::from_utf8(bytes).map_err(|error| {
        let valid = error.utf8_error().valid_up_to();
        let bytes = error.as_bytes();
        let before = std::str::from_utf8(&bytes[..valid]).unwrap_or_default();
        Diagnostic::error(
            Code::InvalidUtf8,
            "le fichier n'est pas encodé en UTF-8 : caractère invalide",
        )
        .at(Location::at(path, before, valid))
    })
}

fn write(path: &Path, data: &[u8], env: &Env, reporter: &mut Reporter) -> Result<(), Failed> {
    super::write_atomically(path, data, &env.cwd).map_err(|d| reporter.fail(d))
}
