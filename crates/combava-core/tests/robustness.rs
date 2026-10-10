//! `transpile` ne panique jamais et respecte ses garanties, quel que soit le
//! texte reçu (spécification, sections 2.1 et 10).

use combava_core::{Config, Severity, transpile};

/// Fragments de markdown combinés au hasard, choisis pour toucher toutes les
/// constructions et leurs cas limites.
const FRAGMENTS: &[&str] = &[
    "#",
    "# ",
    "## ",
    "*",
    "**",
    "_",
    "~~",
    "`",
    "```",
    "```{=typst}",
    "```rust {caption=\"a\"}",
    "[",
    "]",
    "(",
    ")",
    "![",
    "](",
    "[[",
    "]]",
    "|",
    "|:-:|",
    "@",
    "[@a]",
    "[@a; @b]",
    "[^1]",
    "[^1]: ",
    "$",
    "$$",
    "\\",
    ">",
    "> [!NOTE]",
    "- ",
    "1. ",
    "3) ",
    "- [ ] ",
    ": ",
    "---",
    "***",
    "+++",
    "{#id}",
    "{#a!b}",
    "<b>",
    "</b>",
    "<!--",
    "-->",
    "\n",
    "\n\n",
    "    ",
    "\t",
    "\r\n",
    "\r",
    " ",
    "a",
    "é",
    "2026.",
    "x.png",
    "../y.png",
    "https://z",
    "#ancre",
    "&amp;",
    "\u{0}",
];

/// Générateur pseudo-aléatoire déterministe (xorshift), pour des échecs reproductibles.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

fn check(markdown: &str, config: &Config) {
    // Comme en CommonMark : `\n`, `\r\n` ou `\r` seul terminent une ligne.
    let line_count = markdown.replace("\r\n", "\n").split(['\n', '\r']).count();
    match transpile(markdown, config) {
        Ok(output) => {
            assert!(output.typst.ends_with('\n'), "{markdown:?}");
            assert!(!output.typst.contains('\r'), "{markdown:?}");
            assert!(
                output
                    .diagnostics
                    .iter()
                    .all(|d| d.severity == Severity::Warning)
            );
            for line in 1..=output.typst.lines().count() {
                if let Some(md_line) = output.source_map.markdown_line(line) {
                    assert!((1..=line_count).contains(&md_line), "{markdown:?}");
                }
            }
        }
        Err(error) => {
            assert!(
                error
                    .diagnostics
                    .iter()
                    .any(|d| d.severity == Severity::Error)
            );
        }
    }
    let diagnostics = match transpile(markdown, config) {
        Ok(output) => output.diagnostics,
        Err(error) => error.diagnostics,
    };
    for d in &diagnostics {
        assert!(
            d.span.start <= d.span.end && d.span.end <= markdown.len(),
            "{markdown:?}"
        );
        assert!(
            d.line >= 1 && d.line <= line_count && d.column >= 1,
            "{markdown:?}"
        );
        assert_eq!(d.severity, d.code.severity());
    }
    assert!(
        diagnostics
            .windows(2)
            .all(|w| w[0].span.start <= w[1].span.start)
    );
}

#[test]
fn entrees_aleatoires() {
    let configs = [
        Config::default(),
        Config {
            bibliography: true,
            ..Config::default()
        },
    ];
    let mut rng = Rng(0x2545_f491_4f6c_dd1d);
    for i in 0..4000 {
        let len = (rng.next() % 60) as usize;
        let markdown: String = (0..len)
            .map(|_| FRAGMENTS[(rng.next() % FRAGMENTS.len() as u64) as usize])
            .collect();
        check(&markdown, &configs[i % 2]);
    }
}

#[test]
fn imbrications_profondes() {
    let config = Config::default();
    check(&(">".repeat(20_000) + " a"), &config);
    check(&"*a ".repeat(5_000), &config);
    check(&"[".repeat(20_000), &config);
    check(&"![".repeat(5_000), &config);
    let nested_list: String = (0..2_000)
        .map(|i| format!("{}- a\n", "  ".repeat(i)))
        .collect();
    check(&nested_list, &config);
    let nested_quotes_lists: String = (0..2_000)
        .map(|i| format!("{}> - a\n", "> ".repeat(i)))
        .collect();
    check(&nested_quotes_lists, &config);
}

#[test]
fn notes_qui_s_appellent_elles_memes() {
    check(
        "a[^1]\n\n[^1]: b[^1] c[^2]\n\n[^2]: d[^1]\n",
        &Config::default(),
    );
}
