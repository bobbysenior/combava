// Génère le site statique dans site/dist :
//   - la page d'accueil (src/index.html) ;
//   - une page HTML par document de ../docs, converti depuis le markdown.
// Tous les liens sont relatifs : le site fonctionne à la racine d'un domaine
// comme dans le sous-dossier d'un projet GitHub Pages.

import { execFileSync } from "node:child_process";
import { cp, mkdir, readFile, readdir, rm, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import hljs from "highlight.js/lib/core";
import bash from "highlight.js/lib/languages/bash";
import python from "highlight.js/lib/languages/python";
import rust from "highlight.js/lib/languages/rust";
import toml from "highlight.js/lib/languages/ini";
import markdownLang from "highlight.js/lib/languages/markdown";
import { Marked } from "marked";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SRC = join(ROOT, "site", "src");
// Aperçus HTML des exemples de docs/markdown.md, un fichier par exemple.
const PREVIEWS = join(SRC, "previews");
const DIST = join(ROOT, "site", "dist");

const REPO_URL = "https://github.com/bobbysenior/combava";
const PORTFOLIO_URL = "https://thomas.mezino.re";

// URL publique du site, sans « / » final : le sitemap, les liens canoniques et
// les images des cartes sociales exigent des URL absolues. SITE_URL permet de
// générer le site pour une autre adresse (test, domaine différent).
const SITE_URL = (process.env.SITE_URL ?? "https://thomas.mezino.re/combava").replace(/\/+$/, "");
const BASE_PATH = new URL(SITE_URL).pathname.replace(/\/+$/, "");

const SITE_NAME = "Combava";
const SOCIAL_IMAGE = { file: "og-image.png", width: 1200, height: 630, alt: "Combava : du markdown au PDF, sans réglage." };

// Ordre d'affichage dans la navigation.
const DOCS = [
  { file: "guide", label: "Guide d'utilisation" },
  { file: "markdown", label: "Référence markdown" },
  { file: "specification", label: "Spécification" },
  { file: "architecture", label: "Architecture" },
];

hljs.registerLanguage("bash", bash);
hljs.registerLanguage("python", python);
hljs.registerLanguage("rust", rust);
hljs.registerLanguage("toml", toml);
hljs.registerLanguage("markdown", markdownLang);

const CALLOUTS = {
  NOTE: "Note",
  TIP: "Astuce",
  IMPORTANT: "Important",
  WARNING: "Attention",
  CAUTION: "Danger",
};

const escapeHtml = (s) =>
  s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");

// Même algorithme que GitHub, pour que les ancres des documents (qui sont
// aussi lues sur GitHub) restent valides : minuscules, ponctuation retirée,
// espaces remplacés par des tirets, suffixe -1, -2… en cas de doublon.
function slugger() {
  const seen = new Map();
  return (text) => {
    const base = text
      .toLowerCase()
      .replace(/[^\p{L}\p{N}\p{M}\s_-]/gu, "")
      .trim()
      .replace(/\s/g, "-");
    const n = seen.get(base) ?? 0;
    seen.set(base, n + 1);
    return n === 0 ? base : `${base}-${n}`;
  };
}

// Texte brut d'un titre markdown (sans code, emphase ni lien).
const plainText = (md) =>
  md
    .replace(/\[([^\]]*)\]\([^)]*\)/g, "$1")
    .replace(/[`*]/g, "")
    .replace(/(^|\W)_|_(\W|$)/g, "$1$2");

// Un bloc ```markdown {preview=nom} est suivi de l'aperçu `previews/nom.html`.
// Sur GitHub, l'étiquette est ignorée : le bloc reste un simple bloc de code.
function render(markdown, file, previews, usedPreviews) {
  const slug = slugger();
  const headings = [];
  const links = [];
  const marked = new Marked({
    gfm: true,
    renderer: {
      heading({ tokens, depth, text }) {
        const id = slug(plainText(text));
        const inner = this.parser.parseInline(tokens);
        headings.push({ depth, id, text: plainText(text) });
        return `<h${depth} id="${id}"><a class="anchor" href="#${id}">${inner}</a></h${depth}>\n`;
      },
      code({ text, lang }) {
        const name = (lang ?? "").split(/\s/)[0];
        const preview = (lang ?? "").match(/\{preview=([\w-]+)\}/)?.[1];
        const code = hljs.getLanguage(name)
          ? hljs.highlight(text, { language: name }).value
          : escapeHtml(text);
        const cls = hljs.getLanguage(name) ? ` language-${name}` : "";
        const block = `<pre><code class="hljs${cls}">${code}</code></pre>\n`;
        if (!preview) return block;
        if (!(preview in previews)) throw new Error(`${file}.md : aperçu introuvable « ${preview} »`);
        usedPreviews.add(preview);
        return (
          `<div class="example">${block}` +
          `<div class="example-render"><p class="example-label">Rendu</p>` +
          `<div class="pdf">${previews[preview]}</div></div></div>\n`
        );
      },
      link({ href, title, tokens }) {
        const inner = this.parser.parseInline(tokens);
        const local = href.match(/^([a-z]+)\.md(#.*)?$/);
        let target = href;
        if (local) {
          links.push({ doc: local[1], fragment: local[2]?.slice(1) });
          target = `${local[1]}.html${local[2] ?? ""}`;
        } else if (href.startsWith("#")) {
          links.push({ doc: file, fragment: href.slice(1) });
        }
        const t = title ? ` title="${escapeHtml(title)}"` : "";
        return `<a href="${escapeHtml(target)}"${t}>${inner}</a>`;
      },
      blockquote({ tokens }) {
        const body = this.parser.parse(tokens);
        const m = body.match(/^<p>\[!(NOTE|TIP|IMPORTANT|WARNING|CAUTION)\]\s*(?:<br>)?\s*/);
        if (!m) return `<blockquote>\n${body}</blockquote>\n`;
        const kind = m[1].toLowerCase();
        return (
          `<div class="callout callout-${kind}">` +
          `<p class="callout-title">${CALLOUTS[m[1]]}</p>\n<p>${body.slice(m[0].length)}</div>\n`
        );
      },
      table(token) {
        // Évite que les grands tableaux débordent de la page sur mobile.
        const head = this.tablerow({ text: token.header.map((c) => this.tablecell(c)).join("") });
        const rows = token.rows
          .map((r) => this.tablerow({ text: r.map((c) => this.tablecell(c)).join("") }))
          .join("");
        return `<div class="table-wrap"><table>\n<thead>\n${head}</thead>\n<tbody>${rows}</tbody>\n</table></div>\n`;
      },
    },
  });
  const html = marked.parse(markdown);
  return { html, headings, links };
}

const read = (path) => readFile(path, "utf8");

// path : chemin de la page dans le site (« index.html », « docs/guide.html »),
// qui donne le lien canonique et l'URL des cartes sociales. Sans path, la page
// est exclue de l'indexation (404).
function layout({ title, description, body, root, current, path, type = "website" }) {
  const url = path === undefined ? undefined : path === "index.html" ? `${SITE_URL}/` : `${SITE_URL}/${path}`;
  const t = escapeHtml(title);
  const d = escapeHtml(description);
  const image = `${SITE_URL}/${SOCIAL_IMAGE.file}`;
  const social = url === undefined ? "" : `
  <link rel="canonical" href="${url}">
  <meta property="og:type" content="${type}">
  <meta property="og:site_name" content="${SITE_NAME}">
  <meta property="og:locale" content="fr_FR">
  <meta property="og:title" content="${t}">
  <meta property="og:description" content="${d}">
  <meta property="og:url" content="${url}">
  <meta property="og:image" content="${image}">
  <meta property="og:image:width" content="${SOCIAL_IMAGE.width}">
  <meta property="og:image:height" content="${SOCIAL_IMAGE.height}">
  <meta property="og:image:alt" content="${escapeHtml(SOCIAL_IMAGE.alt)}">
  <meta name="twitter:card" content="summary_large_image">
  <meta name="twitter:title" content="${t}">
  <meta name="twitter:description" content="${d}">
  <meta name="twitter:image" content="${image}">
  <meta name="twitter:image:alt" content="${escapeHtml(SOCIAL_IMAGE.alt)}">`;
  const robots = url === undefined ? '\n  <meta name="robots" content="noindex">' : "";
  const nav = DOCS.map(
    (d) =>
      `<a href="${root}docs/${d.file}.html"${d.file === current ? ' aria-current="page"' : ""}>${d.label}</a>`,
  ).join("\n        ");
  return `<!doctype html>
<html lang="fr">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>${escapeHtml(title)}</title>
  <meta name="description" content="${d}">
  <meta name="author" content="bobbysenior">
  <meta name="theme-color" content="#3a7a1a">${robots}${social}
  <link rel="icon" type="image/svg+xml" href="${root}logo.svg">
  <link rel="icon" type="image/png" sizes="32x32" href="${root}favicon-32.png">
  <link rel="apple-touch-icon" href="${root}apple-touch-icon.png">
  <link rel="stylesheet" href="${root}style.css">
</head>
<body>
  <header class="topbar">
    <div class="wrap">
      <a class="brand" href="${root}index.html"><img src="${root}logo.svg" alt="" width="28" height="28">combava</a>
      <nav aria-label="Principale">
        ${nav}
        <a href="${REPO_URL}">GitHub</a>
      </nav>
    </div>
  </header>
${body}
  <footer class="footer">
    <div class="wrap">
      <span>Combava est publié sous <a href="${REPO_URL}/blob/main/LICENSE">licence MIT</a>.</span>
      <span><a href="${REPO_URL}">Code source</a> · <a href="${PORTFOLIO_URL}" rel="author">Autres projets</a></span>
    </div>
  </footer>
</body>
</html>
`;
}

function docPage({ file, label, html, headings }, summaries) {
  const title = headings.find((h) => h.depth === 1)?.text ?? label;
  const sections = headings.filter((h) => h.depth === 2);
  const toc = sections.map((h) => `<li><a href="#${h.id}">${escapeHtml(h.text)}</a></li>`).join("\n");
  const others = DOCS.map(
    (d) =>
      `<li><a href="${d.file}.html"${d.file === file ? ' aria-current="page"' : ""}>${d.label}</a></li>`,
  ).join("\n");
  const body = `  <div class="wrap doc">
    <aside class="sidebar" aria-label="Sommaire">
      <p class="sidebar-title">Documentation</p>
      <ul>
${others}
      </ul>
      <p class="sidebar-title">Dans ce document</p>
      <ul class="toc">
${toc}
      </ul>
      <p class="sidebar-source"><a href="${REPO_URL}/blob/main/docs/${file}.md">Voir le source sur GitHub</a></p>
    </aside>
    <main class="prose">
${html}
    </main>
  </div>`;
  return layout({
    title: `${title} – combava`,
    description: summaries[file],
    body,
    root: "../",
    current: file,
    path: `docs/${file}.html`,
    type: "article",
  });
}

async function main() {
  await rm(DIST, { recursive: true, force: true });
  await mkdir(join(DIST, "docs", "previews"), { recursive: true });

  const summaries = {
    guide: "Installer combava, écrire un document, régler l'en-tête, comprendre les erreurs.",
    markdown: "Tout le markdown pris en charge par combava, avec un exemple et son rendu pour chaque élément.",
    specification: "Le comportement exact de combava : markdown pris en charge, diagnostics, contrat du template.",
    architecture: "L'organisation du code : combava-core, combava-cli et le template.",
  };

  const previews = {};
  for (const f of await readdir(PREVIEWS)) {
    if (f.endsWith(".html")) previews[f.slice(0, -5)] = (await read(join(PREVIEWS, f))).trim();
  }
  const usedPreviews = new Set();

  const pages = {};
  for (const d of DOCS) {
    const md = await read(join(ROOT, "docs", `${d.file}.md`));
    pages[d.file] = { ...d, ...render(md, d.file, previews, usedPreviews) };
  }

  const unused = Object.keys(previews).filter((name) => !usedPreviews.has(name));
  if (unused.length > 0) {
    console.error(`aperçus jamais utilisés : ${unused.join(", ")}`);
    process.exit(1);
  }

  // Liens internes cassés : le build s'arrête, comme combava le ferait.
  const broken = [];
  for (const page of Object.values(pages)) {
    for (const { doc, fragment } of page.links) {
      const target = pages[doc];
      if (!target) broken.push(`${page.file}.md : document inconnu « ${doc}.md »`);
      else if (fragment && !target.headings.some((h) => h.id === fragment))
        broken.push(`${page.file}.md : ancre introuvable « ${doc}.md#${fragment} »`);
    }
  }
  if (broken.length > 0) {
    console.error(`liens internes cassés :\n  ${broken.join("\n  ")}`);
    process.exit(1);
  }

  for (const page of Object.values(pages)) {
    await writeFile(join(DIST, "docs", `${page.file}.html`), docPage(page, summaries));
  }

  // La page d'accueil porte son propre contenu ; seuls la barre de navigation
  // et le pied de page sont partagés avec les pages de documentation.
  const home = await read(join(SRC, "index.html"));
  const shell = layout({
    title: "Combava – du markdown au PDF, sans réglage",
    description:
      "Combava est un générateur de rapports PDF à partir de markdown. Un seul template, aucune option : on écrit le contenu, combava fait la mise en page.",
    body: home,
    root: "",
    current: "",
    path: "index.html",
  });
  await writeFile(join(DIST, "index.html"), shell);

  // GitHub Pages sert 404.html pour toute adresse inconnue du site, à n'importe
  // quelle profondeur : ses liens sont donc absolus (depuis la racine du site).
  const notFound = layout({
    title: "Page introuvable – combava",
    description: "Cette page n'existe pas.",
    body: `  <main class="wrap not-found">
    <p class="not-found-code">404</p>
    <h1>Page introuvable</h1>
    <p>Cette adresse ne mène à rien. Retournez à l'accueil ou ouvrez la documentation.</p>
    <p class="actions">
      <a class="button primary" href="${BASE_PATH}/index.html">Accueil</a>
      <a class="button" href="${BASE_PATH}/docs/guide.html">Guide d'utilisation</a>
    </p>
  </main>`,
    root: `${BASE_PATH}/`,
    current: "",
  });
  await writeFile(join(DIST, "404.html"), notFound);

  // Sitemap : la date d'une page est celle du dernier commit qui touche sa source.
  const lastModified = (...paths) => {
    try {
      const out = execFileSync("git", ["log", "-1", "--format=%cs", "--", ...paths], { cwd: ROOT, encoding: "utf8" });
      return out.trim() || undefined;
    } catch {
      return undefined;
    }
  };
  const entries = [
    { loc: `${SITE_URL}/`, date: lastModified("site/src/index.html", "site/src/style.css") },
    ...DOCS.map((d) => ({ loc: `${SITE_URL}/docs/${d.file}.html`, date: lastModified(`docs/${d.file}.md`) })),
  ];
  const urls = entries
    .map((e) => `  <url>\n    <loc>${e.loc}</loc>${e.date ? `\n    <lastmod>${e.date}</lastmod>` : ""}\n  </url>`)
    .join("\n");
  await writeFile(
    join(DIST, "sitemap.xml"),
    `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${urls}\n</urlset>\n`,
  );
  await writeFile(join(DIST, "robots.txt"), `User-agent: *\nAllow: /\n\nSitemap: ${SITE_URL}/sitemap.xml\n`);

  await cp(join(SRC, "style.css"), join(DIST, "style.css"));
  await cp(join(ROOT, "examples", "images", "pipeline.svg"), join(DIST, "docs", "previews", "pipeline.svg"));
  await cp(join(ROOT, "assets", "logo.svg"), join(DIST, "logo.svg"));
  for (const f of [SOCIAL_IMAGE.file, "favicon-32.png", "apple-touch-icon.png"]) {
    await cp(join(SRC, f), join(DIST, f));
  }
  await writeFile(join(DIST, ".nojekyll"), "");

  console.log(`site généré dans ${DIST}`);
}

main();
