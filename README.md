<p align="center">
  <img src="assets/banner.svg" alt="Combava: Markdown in. PDF out. No knobs." width="100%">
</p>

<p align="center">
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-6a9a1f"></a>
  <img alt="Status: early development" src="https://img.shields.io/badge/status-early%20development-orange">
  <img alt="Built with Rust" src="https://img.shields.io/badge/built%20with-Rust-b7410e">
</p>

**Combava is an opinionated Markdown to PDF generator.** You write Markdown with a small TOML header. Combava gives you a finished PDF report. There is no style to tune and no template to set up.

> *Avec un peu de combava, ça passe mieux !*

## Why

Combava started in computer science engineering studies, which mean a lot of reports. Each time, we copied the same LaTeX or Word template and fixed it by hand. We wanted a tool where we only write the content.

We chose Markdown because it is the most "gitable" text format. A diff of a report is easy to read, and a merge is rarely painful.

Combava is a small tool, made first for our own school work. If it fits your needs, use it. It will not try to fit every need.

## The name

The *combava* (kaffir lime) is a small, bumpy, very strong citrus fruit. You use a little, and it changes the whole dish. This is what the tool should do for your reports.

## Opinions

Combava decides these points for you. They are not bugs, and they are not on the roadmap.

| Combava decides | Why |
|---|---|
| **One template, one look.** The default template has no style options. | A report should be about its content. Each style option is a choice you must make again for each report. |
| **The document language is French.** Quotation marks and typography follow French rules. | Our reports are in French. One language means no language setting. |
| **A closed list of header keys.** An unknown key is an error, not a silent no-op. | A typo like `autors = [...]` must stop the build. It must not give you a PDF without an author. |
| **Mistakes are errors.** Remote images, broken internal links, and citations without a bibliography stop the build. | A PDF that builds with a missing image is a PDF you send by mistake. |
| **Raw HTML is ignored, with a warning.** | A PDF is not a web page. Combava does not guess what the HTML should look like. |
| **A failed build never destroys the previous PDF.** | The last good PDF stays in place until the new one is ready. |
| **Typst is built in.** You do not install LaTeX, Pandoc, or a Typst CLI. | One binary does the whole job. |
| **Same input, same output.** The transpiler is a pure function. | You can test it, and you can trust it. |

If you want a different look, you do not add options. You replace the template: it is a folder with a `template.typ` file, and it follows a [small, fixed contract](docs/specification.md#9-contrat-du-template). The code that reads your Markdown stays the same.

## How it works

```
 report.md ──► combava-cli ──► combava-core ──► combava-cli ──► report.pdf
              (config, I/O)   (md → Typst)     (Typst → PDF)
```

1. `combava-cli` reads your file. It splits the `+++` header from the body and merges the configuration layers.
2. `combava-core` transpiles the Markdown body to Typst code. It reports each problem with the exact line and column of your Markdown.
3. `combava-cli` compiles the Typst code with the template and writes the PDF. Typst errors point back to your Markdown lines.

## What a document looks like

````markdown
+++
title = "Network security"
subtitle = "Lab report 3"
authors = ["Ada Lovelace", "Alan Turing"]
teachers = ["Grace Hopper"]
date = "10 octobre 2026"
toc = true
bibliography = "references.bib"
+++

# Introduction

Combava turns Markdown into a PDF through Typst [@typst2023].

> [!WARNING]
> Callouts use the GitHub syntax.

```rust {caption="A captioned listing"}
fn main() {
    println!("Hello, combava!");
}
```

Maths use LaTeX: $E = mc^2$.
````

Supported Markdown: headings with labels, links and wikilinks, images with captions, tables, footnotes, task lists, definition lists, callouts, math, code blocks with captions, and citations. The full list, with the exact Typst output for each case, is in section 4 of the [specification](docs/specification.md#4-correspondance-markdown--typst). A complete reference document is in [`examples/report.md`](examples/report.md).

If Markdown cannot say what you need, a fenced block marked ` ```{=typst} ` is copied as Typst code. This is the only escape hatch.

## Installation

You need Rust (stable) and `make`. On Linux:

```bash
make install
```

This command installs three items:

- the binary in `~/.local/bin/combava`
- the global configuration in `~/.config/combava/config.toml`. All keys are in the file. The keys that do not have a default are commented out.
- the default template in `~/.config/combava/templates/default/`. This copy replaces the template in the binary. Put your logos in its `images/` folder, with the same file names.

`make install` does not overwrite a configuration or a template that you installed before. To replace them, use `make install FORCE=1`. To remove the binary, use `make uninstall`.

The first document with math downloads a Typst package, so you need a network connection one time.

## Usage

```bash
combava init report.md      # create a document with a starter header
combava build report.md     # write report.pdf next to report.md
```

Settings come from four layers. The first layer that sets a key wins:

1. command-line arguments (`-o`, `-t`)
2. the header of the document
3. `.combava/config.toml` in the project
4. `config.toml` in the global configuration directory

The [user guide](docs/guide.md) (in French) explains the header keys, the supported Markdown, the templates, and each error message.

## Status

Combava is under heavy development.

- [x] **`combava-core`**: the Markdown to Typst transpiler, with diagnostics and a line map. It has 80 tests: snapshots, diagnostics, API contract, and robustness on random input.
- [x] **Specification and architecture**: [`docs/specification.md`](docs/specification.md) and [`docs/architecture.md`](docs/architecture.md)
- [x] **`combava-cli`**: header parsing, configuration layers, Typst compilation, PDF export
- [x] **Default template**: cover page, tables, callouts, bibliography style
- [ ] First release

Out of scope for v1: a `watch` mode, free variables for templates, subscript and superscript, automatic French spacing, remote images, HTML, multi-file documents, and PDF/A.

## Repository layout

```
crates/combava-core/   Markdown → Typst transpiler (library, no I/O)
crates/combava-cli/    the `combava` binary
templates/default/     the default template, embedded in the binary
examples/              reference documents
config/config.toml     the global configuration that `make install` installs
docs/                  user guide, specification, and architecture
assets/                banner and logo
```

## Development

```bash
cargo test -p combava-core
```

The user guide, the specification, and the architecture notes are written in French. The specification wins if the two documents disagree.

Commit messages follow Conventional Commits, written in French, with a space before the colon: `feat(core) : transpiler le markdown en Typst`.

## License

[MIT](LICENSE)

<p align="center">
  <img src="assets/logo.svg" alt="" width="72">
</p>
