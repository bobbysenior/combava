# Spécification de Combava

Ce document fait foi pour le comportement de Combava et pour l'interface entre `combava-core` et `combava-cli`. [architecture.md](architecture.md) décrit l'organisation du code ; en cas de désaccord, ce document l'emporte.

- conventions
    - « doit » : obligatoire ; « peut » : laissé au choix de l'implémentation
    - toutes les lignes et colonnes sont numérotées **à partir de 1**
    - une ligne se termine par `\n`, `\r\n` ou `\r` seul, comme en CommonMark
    - une colonne compte des caractères Unicode (`char`), pas des octets
    - un `span` est une plage d'**octets** `start..end` dans le texte reçu
    - les messages destinés à l'utilisateur sont en français, commencent par une minuscule, n'ont pas de point final et citent les valeurs entre « »

## 1. Répartition du travail

- lot **core** : `crates/combava-core`, sections 2 à 4
- lot **CLI** : `crates/combava-cli`, sections 5 à 8
- lot **template** : `templates/default/`, section 9, à attribuer une fois les deux premiers lots avancés
- fichiers partagés
    - `examples/report.md` : document de référence qui utilise toutes les fonctionnalités de la section 4
        - écrit par le lot core, utilisé par le lot CLI pour les tests de bout en bout
- l'interface entre les lots est **gelée** : sections 2, 3, 6.2 et 9
    - toute modification passe par une PR qui modifie aussi ce document, relue par l'autre développeur

## 2. API publique du core

- le core expose **exactement** les éléments ci-dessous ; tout le reste est privé (`pub(crate)`)
- dépendances autorisées : `pulldown-cmark`, `thiserror` ; pas de `serde`, pas de `typst`

```rust
// crates/combava-core/src/lib.rs

/// Transpile le corps markdown d'un document en code Typst complet.
pub fn transpile(markdown: &str, config: &Config) -> Result<Output, TranspileError>;

/// Chemins virtuels partagés avec le `World` du CLI.
pub mod paths {
    /// Le code généré.
    pub const MAIN: &str = "/__combava__/main.typ";
    /// Le dossier du template résolu.
    pub const TEMPLATE_DIR: &str = "/__combava__/template";
    /// Le point d'entrée du template, importé par le code généré.
    pub const TEMPLATE_ENTRY: &str = "/__combava__/template/template.typ";
    /// Le fichier de bibliographie résolu.
    pub const BIBLIOGRAPHY: &str = "/__combava__/bibliography.bib";
}

/// Package Typst utilisé pour les maths, version épinglée.
pub const MITEX_PACKAGE: &str = "@preview/mitex:0.2.7";
```

```rust
// crates/combava-core/src/config.rs

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
    pub header_text: Option<String>,
    pub bibliography: bool,
}

/// `toc` vaut `true` ; tous les autres champs sont `None`, vides ou `false`.
impl Default for Config { /* … */ }
```

```rust
// crates/combava-core/src/output.rs

#[derive(Debug, Clone)]
pub struct Output {
    /// Le code Typst complet, à écrire tel quel dans `paths::MAIN`.
    pub typst: String,
    /// Uniquement des avertissements, triés par `span.start`.
    pub diagnostics: Vec<Diagnostic>,
    pub source_map: SourceMap,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: Code,
    pub message: String,
    pub span: std::ops::Range<usize>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Warning,
    Error,
}

/// Liste en section 2.4. `non_exhaustive` : le CLI ne doit pas faire de `match` exhaustif.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Code { /* … */ }

impl Code {
    /// Identifiant stable en kebab-case, par exemple `"raw-html"`.
    pub fn as_str(self) -> &'static str;
    /// Gravité fixe du code.
    pub fn severity(self) -> Severity;
}
```

```rust
// crates/combava-core/src/source_map.rs

#[derive(Debug, Clone)]
pub struct SourceMap { /* privé */ }

impl SourceMap {
    /// Ligne markdown qui a produit la ligne `typst_line` du code généré.
    /// `None` pour le préambule, les lignes de séparation et la bibliographie.
    pub fn markdown_line(&self, typst_line: usize) -> Option<usize>;
}
```

```rust
// crates/combava-core/src/error.rs

#[derive(Debug, Clone, thiserror::Error)]
#[error("le document contient des erreurs")]
pub struct TranspileError {
    /// Erreurs **et** avertissements, triés par `span.start`. Au moins une erreur.
    pub diagnostics: Vec<Diagnostic>,
}
```

### 2.1 Garanties du core

- `transpile` est une fonction pure
    - pas de lecture ni d'écriture de fichier, pas de variable d'environnement, pas d'horloge, pas de réseau
    - même entrée → même sortie, à l'octet près
- `transpile` ne panique jamais, quel que soit le texte reçu
- elle analyse tout le document avant de répondre : toutes les erreurs sont rapportées en une fois
- `Ok` ⇔ aucun diagnostic de gravité `Error`
- `Output::typst`
    - UTF-8, fins de ligne `\n` uniquement, se termine par un `\n`
    - ne fait référence à aucun fichier en dehors de :
        - `paths::TEMPLATE_ENTRY`, `paths::BIBLIOGRAPHY`, `MITEX_PACKAGE`
        - les images du document, sous la forme `/chemin/relatif` (jamais de `..`)
        - ce que l'utilisateur écrit dans un bloc ` ```{=typst} `
    - aucun texte markdown ne peut injecter de code Typst, sauf par un bloc ` ```{=typst} `
- `Diagnostic`
    - `span`, `line` et `column` désignent une position dans `markdown`
    - `severity == code.severity()`

### 2.2 Obligations de l'appelant (le CLI)

- `markdown` est le contenu **complet** du fichier, dans lequel le CLI a remplacé par une espace ASCII chaque octet du frontmatter (délimiteurs et BOM compris), **sauf les `\n`**
    - pourquoi : les octets, lignes et colonnes du core sont alors ceux du fichier, sans conversion
    - c'est sans danger : en CommonMark, une ligne d'espaces est une ligne vide
- `config.header_text` reste `None` si l'utilisateur ne l'a pas défini : le core le remplace par `title`
- `config.bibliography` vaut `true` si et seulement si un fichier `.bib` est monté à `paths::BIBLIOGRAPHY`
- le CLI ne lit ni ne modifie `Output::typst` : il l'écrit tel quel

### 2.3 Structure du code généré

- dans cet ordre :
    1) une ligne de commentaire : `// Généré par combava-core <version> — ne pas modifier`
    2) `#import "<MITEX_PACKAGE>": mi, mitex` **si et seulement si** le document contient des maths
    3) `#import "<paths::TEMPLATE_ENTRY>": template, callout`
    4) `#show: template.with(…)` avec **tous** les arguments de la section 9.1, dans cet ordre
    5) le corps transpilé
    6) `#bibliography("<paths::BIBLIOGRAPHY>")` si `config.bibliography`
- sérialisation des valeurs de `Config` en littéraux Typst
    - `Option<String>` : `none`, ou chaîne entre `"` avec `\` → `\\`, `"` → `\"`, saut de ligne → `\n`, retour chariot → `\r`, tabulation → `\t`, autre caractère de contrôle → `\u{…}`
    - `Vec<String>` : `()` si vide, `("a",)` pour un élément (la virgule finale est **obligatoire** : `("a")` est une chaîne), `("a", "b")` sinon
    - `bool` : `true` ou `false`
    - les valeurs sont du texte brut : `*titre*` s'affiche avec les astérisques
- la mise en forme du corps (indentation, retours à la ligne, forme `=` ou `#heading`) est libre : seule la sémantique de la section 4 est imposée

### 2.4 Codes de diagnostic du core

| Code | Gravité | Déclencheur |
|---|---|---|
| `remote-image` | erreur | image dont la destination commence par `http://` ou `https://` |
| `invalid-image-path` | erreur | image avec un chemin absolu, ou qui sort du dossier du `.md` après normalisation des `..` |
| `broken-link` | erreur | lien interne ou wikilink vers un label qui n'existe pas |
| `duplicate-label` | erreur | deux titres avec le même `{#id}` explicite |
| `citation-without-bibliography` | erreur | citation `[@cle]` alors que `config.bibliography` vaut `false` |
| `raw-html` | avertissement | HTML brut, en bloc ou en ligne (les commentaires `<!-- … -->` ne déclenchent rien) |
| `invalid-label` | avertissement | `{#id}` contenant un caractère interdit ; le slug automatique est utilisé à la place |
| `invalid-code-attributes` | avertissement | attributs `{…}` d'un bloc de code mal formés ou inconnus ; ils sont ignorés |
| `undefined-footnote` | avertissement | appel `[^x]` sans définition ; l'appel est affiché comme du texte |
| `unused-footnote` | avertissement | définition `[^x]: …` jamais appelée |

## 3. Parsing

- pulldown-cmark, options activées une par une (jamais `Options::all()`)
    - activées : `ENABLE_TABLES`, `ENABLE_FOOTNOTES`, `ENABLE_STRIKETHROUGH`, `ENABLE_TASKLISTS`, `ENABLE_HEADING_ATTRIBUTES`, `ENABLE_DEFINITION_LIST`, `ENABLE_GFM`, `ENABLE_MATH`, `ENABLE_WIKILINKS`
    - toutes les autres restent désactivées (raisons dans [architecture.md](architecture.md))
- positions : `Parser::into_offset_iter`, pour remplir `span`, `line`, `column` et la `SourceMap`

## 4. Correspondance markdown → Typst

- la forme Typst indiquée est un exemple ; la sémantique est obligatoire

### 4.1 Texte

- échappement de tout texte issu du markdown
    - préfixer par `\` les caractères : `\ # $ * _ @ < > [ ] ` ~ / = - +`
    - préfixer par `\` tout `.` précédé d'un chiffre ASCII
        - pourquoi : `2026. fin` en début de ligne serait une liste numérotée en Typst
    - ne pas échapper `"` ni `'` : Typst les transforme en guillemets et apostrophes français
- retour à la ligne simple (soft break) → une espace
- retour à la ligne forcé (hard break) → `#linebreak()`
- emphase, gras, barré → `#emph[…]`, `#strong[…]`, `#strike[…]`
- code en ligne → `#raw("…")`

### 4.2 Titres et labels

- `#` à `######` → titre de niveau 1 à 6, suivi de son label `<…>`
- chaque titre reçoit un label
    - `{#id}` explicite s'il est valide : uniquement lettres, chiffres, `-`, `_`, `.`, `:`
    - sinon un slug automatique, calculé comme GitHub :
        1) texte du titre sans mise en forme
        2) mise en minuscules
        3) suppression de tout caractère autre que lettre, chiffre, espace, `-` ou `_`
        4) chaque espace devient `-`
        5) slug vide → `section`
        6) si le slug est déjà pris, ajout de `-1`, `-2`… dans l'ordre d'apparition
    - pourquoi : `[voir](#ma-section)` fonctionne alors à la fois dans l'aperçu GitHub et dans le PDF
- les classes et attributs `{.classe cle=valeur}` sont ignorés

### 4.3 Liens

- externe (`https://…`, `mailto:…`, autolien `<…>`) → `#link("url")[…]`
- interne `[texte](#cible)` → `#link(label("cible"))[texte]`, la cible étant décodée (`%20` → espace)
- wikilink `[[cible]]` ou `[[cible|texte]]`
    - label visé : `cible` si c'est un `{#id}` explicite, sinon le slug de `cible`
    - texte affiché : `texte`, sinon `cible`
- label inexistant → erreur `broken-link`
- le titre d'un lien (`[a](url "titre")`) est ignoré

### 4.4 Images

- chemin
    - destination décodée (`%20` → espace), normalisée (`./`, `a/../`), puis émise avec un `/` de tête
    - `http://` ou `https://` → erreur `remote-image`
    - chemin absolu, ou qui remonte au-dessus du dossier du `.md` → erreur `invalid-image-path`
- seule dans son paragraphe (aux espaces près) → figure
    - `#figure(image("/img.png", alt: "…"), caption: […])`
    - légende : le titre de l'image (`![alt](img.png "Légende")`), échappé comme du texte ; pas de `caption` sans titre
- dans une phrase → `#box(image("/img.png", alt: "…", height: 1em))`
- `alt` : le texte alternatif sans mise en forme ; absent s'il est vide

### 4.5 Blocs

- paragraphe → paragraphe
- citation en bloc → `#quote(block: true)[…]`
- callout GFM `> [!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, `[!CAUTION]` → `#callout("note")[…]`, `"tip"`, `"important"`, `"warning"`, `"caution"`
- liste à puces → `#list(tight: …, […], …)`
- liste numérotée → `#enum(start: n, tight: …, […], …)`, `n` étant le premier numéro
    - `tight: true` si aucun élément ne contient de paragraphe séparé par une ligne vide
- case à cocher → `☐ ` ou `☒ ` au début de l'élément
- liste de définitions → `#terms(terms.item[terme][définition], …)` ; plusieurs définitions d'un même terme sont séparées par un paragraphe
- tableau → `#table(columns: n, align: (…), table.header(…), …)`
    - alignement `:--` → `left`, `:-:` → `center`, `--:` → `right`, sans indication → `left`
- ligne horizontale (`---`, `***`, `___`, `- - -`…) → `#pagebreak()`
    - dans un conteneur (citation, callout, liste, note) → `#line(length: 100%)` : Typst y interdit les sauts de page

### 4.6 Blocs de code

- info string : `<langage>` puis éventuellement des attributs `{cle="valeur" …}`
    - seul attribut connu : `caption` ; valeur entre `"`, avec `\"` pour un guillemet
    - tout autre attribut, ou une syntaxe invalide → avertissement `invalid-code-attributes`, attributs ignorés
- sans légende → `#raw(block: true, lang: "rust", "…")` (`lang: none` sans langage)
- avec légende → `#figure(raw(block: true, …), caption: […])`
    - seuls ces blocs sont des figures, et donc les seuls listés dans la table des codes
- bloc indenté → comme un bloc clôturé sans langage
- ` ```{=typst} ` → contenu recopié tel quel, sans échappement
    - chaque ligne recopiée correspond, dans la `SourceMap`, à sa ligne exacte du markdown

### 4.7 Notes de bas de page

- premier appel de `[^x]` → `#footnote[contenu de la définition] <combava-fn-x>`
- appels suivants → `#footnote(<combava-fn-x>)` (même numéro)
- la définition peut se trouver avant ou après l'appel
- dans un titre → `#footnote[…]` sans label
    - pourquoi : Typst recopie les titres dans la table des matières, le label y serait en double
    - un appel ultérieur hors titre émet une nouvelle note, avec label
- une note qui s'appelle elle-même : l'appel interne reste du texte
- le préfixe `combava-fn-` est réservé

### 4.8 Citations

- syntaxe : `[@cle]` ou `[@a; @b]`, la clé ne contenant que lettres, chiffres, `_`, `-`, `.`, `:`
    - repérée dans le texte après fusion des événements `Text` consécutifs (pulldown-cmark peut découper `[`, `@cle` et `]`)
- → `#cite(label("cle"))`, une fois par clé ; Typst regroupe les citations adjacentes
- un `@` hors de cette syntaxe (une adresse mail par exemple) est du texte échappé

### 4.9 Maths

- le contenu est du LaTeX, transmis à mitex sous forme de chaîne échappée
- `$…$` → `#mi("…")`
- `$$…$$` → `#mitex("…")`

### 4.10 HTML

- HTML brut → ignoré, avertissement `raw-html`
- commentaire `<!-- … -->` → ignoré sans avertissement

## 5. Ligne de commande

- binaire `combava`, construit avec `clap`
- les chemins donnés en argument sont relatifs au dossier courant

### 5.1 `combava build [OPTIONS] FILE`

- options
    - `-o`, `--out <FILE>` : surcharge `output`
    - `-t`, `--template <TEMPLATE>` : surcharge `template` (nom ou chemin, section 7)
    - `--transpile-only` : écrit le code Typst au lieu du PDF
- déroulement
    1) lire `FILE` ; s'il n'est pas en UTF-8 → erreur `invalid-utf8`
    2) découper le frontmatter (section 6.1) et le remplacer par des espaces (section 2.2)
    3) trouver la racine du projet, lire et fusionner les couches de configuration (section 6)
    4) résoudre le template (section 7), la bibliographie et le fichier de sortie
    5) appeler `combava_core::transpile`
    6) afficher les diagnostics du core ; arrêter en cas d'erreur
    7) avec `--transpile-only` : écrire `Output::typst` dans le fichier de sortie, extension remplacée par `.typ`, puis s'arrêter
    8) compiler (section 8), afficher les diagnostics Typst
    9) écrire le PDF
- le fichier de sortie n'est écrit (ou remplacé) **que si tout a réussi** : un échec ne détruit jamais le PDF précédent
- le dossier parent du fichier de sortie doit exister

### 5.2 `combava init [OPTIONS] [FILE]`

- `FILE` vaut `rapport.md` par défaut
- options
    - `-t`, `--template <TEMPLATE>` : ajoute `template = "<TEMPLATE>"` au frontmatter, valeur recopiée telle quelle
    - `--force` : écrase un fichier existant
- fichier existant sans `--force` → erreur `file-exists`
- contenu écrit (la date est celle du jour, en français, `1er` pour le premier du mois) :

```markdown
+++
title = "Titre du rapport"
subtitle = "Sous-titre"
authors = ["Prénom Nom"]
teachers = ["Prénom Nom"]
date = "10 octobre 2026"
toc = true
list_of_figures = false
list_of_listings = false
# header_text = "Texte en haut de page (par défaut : le titre)"
+++

# Introduction

```

### 5.3 Aide et version

- `combava help`, `combava --help`, `combava <commande> --help`, `combava --version` : générés par `clap`

### 5.4 Affichage des diagnostics

- sur la sortie d'erreur, une ligne par diagnostic, au format des compilateurs (cliquable dans les éditeurs) :

```
rapport.md:12:5: avertissement[raw-html] : HTML brut ignoré
rapport.md:3:1: erreur[unknown-key] : clé inconnue « autors »
```

- chemin affiché relatif au dossier courant quand c'est possible
- les indications de Typst (`hints`) suivent, une par ligne : `  aide : …`
- diagnostic sans position : `combava: erreur[<code>] : <message>`

### 5.5 Codes de sortie

- `0` : succès, avertissements compris
- `1` : erreur du document, de la configuration, du template ou de la compilation
- `2` : mauvaise utilisation de la ligne de commande (`clap`)

### 5.6 Codes de diagnostic du CLI

| Code | Déclencheur |
|---|---|
| `invalid-utf8` | fichier markdown qui n'est pas en UTF-8 |
| `unclosed-frontmatter` | `+++` d'ouverture sans `+++` de fermeture |
| `invalid-toml` | syntaxe TOML invalide |
| `unknown-key` | clé inconnue |
| `invalid-type` | valeur du mauvais type (par exemple `date = 2026-10-10`, qui est une date TOML et non une chaîne) |
| `output-outside-frontmatter` | clé `output` dans un fichier `config.toml` |
| `template-not-found` | template introuvable ; le message liste les emplacements essayés |
| `bibliography-not-found` | fichier de bibliographie introuvable ou sans extension `.bib` |
| `file-exists` | `init` sur un fichier existant sans `--force` |
| `package-download` | échec du téléchargement d'un package Typst ; indication : une connexion est nécessaire au premier usage des maths |
| `typst` | erreur ou avertissement de compilation Typst |
| `io` | erreur de lecture ou d'écriture |

## 6. Configuration

### 6.1 Frontmatter

- présent si le fichier commence (après un éventuel BOM) par une ligne `+++`
    - espaces et tabulations en fin de ligne tolérés, fins de ligne `\n` ou `\r\n`
- se termine à la ligne `+++` suivante ; absente → erreur `unclosed-frontmatter`
- son contenu est du TOML
- les positions des erreurs TOML sont ramenées à des positions dans le fichier `.md`

### 6.2 Clés

| Clé | Type TOML | Champ de `Config` | Défaut |
|---|---|---|---|
| `title`, `subtitle` | chaîne | `title`, `subtitle` | absent |
| `authors`, `teachers` | tableau de chaînes | `authors`, `teachers` | `[]` |
| `date` | chaîne | `date` | absent |
| `school`, `university`, `academic_year`, `cohort`, `specialization`, `subject` | chaîne | même nom | absent |
| `toc` | booléen | `toc` | `true` |
| `list_of_figures`, `list_of_listings` | booléen | même nom | `false` |
| `header_text` | chaîne | `header_text` | absent (le core met `title`) |
| `bibliography` | chaîne (chemin) | `bibliography = true` si définie | absent |
| `template` | chaîne (nom ou chemin) | — (CLI) | `"default"` |
| `output` | chaîne (chemin) | — (CLI), **frontmatter uniquement** | `<dossier du .md>/<nom du .md>.pdf` |

- toute autre clé → erreur `unknown-key`

### 6.3 Couches

- de la plus prioritaire à la moins prioritaire :
    1) arguments `-o` et `-t`
    2) frontmatter
    3) `<racine du projet>/.combava/config.toml`
    4) `<config globale>/config.toml`
    5) défauts de la section 6.2
- racine du projet : premier dossier contenant un dossier `.combava/`, en remontant depuis le dossier du `.md` ; aucun → pas de couche 3
- config globale : `ProjectDirs::from("", "", "combava").config_dir()` (crate `directories`) ; fichier absent → pas de couche 4
- fusion champ par champ : la première couche qui définit une clé l'emporte
    - une liste n'est jamais concaténée : `authors = []` dans le frontmatter vide la liste

### 6.4 Chemins dans la configuration

- `~` en tête → dossier personnel, dans toutes les couches
- un chemin relatif est résolu par rapport à :
    - frontmatter → dossier du `.md`
    - `.combava/config.toml` → racine du projet
    - configuration globale → dossier de configuration globale
    - arguments de ligne de commande → dossier courant

## 7. Résolution du template

- template = dossier contenant `template.typ`
- valeur contenant `/` ou `\`, ou commençant par `.` ou `~` → chemin vers ce dossier (section 6.4)
- sinon, nom cherché dans l'ordre :
    1) `<racine du projet>/.combava/templates/<nom>/`
    2) `<config globale>/templates/<nom>/`
    3) templates embarqués : `default` (contenu de `templates/default/`, inclus avec `include_dir`)
- un dossier utilisateur nommé `default` remplace donc le template embarqué
- introuvable, ou sans `template.typ` → erreur `template-not-found`

## 8. Compilation

- crates `typst`, `typst-pdf`, `typst-kit`, à une version épinglée dans `Cargo.toml`
- `World`
    - fichier principal : `combava_core::paths::MAIN`, contenu `Output::typst`, en mémoire
    - `combava_core::paths::TEMPLATE_DIR` et ses sous-chemins → dossier du template (sur disque ou embarqué)
    - `combava_core::paths::BIBLIOGRAPHY` → le fichier `.bib` résolu
    - tout autre chemin `/x` → `<dossier du .md>/x`, en lecture seule
    - `today()` → date locale
- polices, dans l'ordre :
    1) fichiers `.ttf`, `.otf`, `.ttc`, `.otc` du dossier `fonts/` du template, s'il existe
    2) polices du système
    3) polices embarquées de `typst-kit`
- packages : téléchargés par `typst-kit` puis mis en cache (même cache que le CLI `typst`)
- diagnostics Typst (gravité conservée, code `typst`)
    - dans `MAIN` : ligne du code généré → `SourceMap::markdown_line`
        - trouvée → position dans le `.md`, colonne 1
        - `None` → diagnostic sur le `.md` sans ligne, message suivi de `(ligne N du code généré, voir --transpile-only)`
    - dans le template ou la bibliographie → chemin réel du fichier, ligne et colonne de Typst
    - attention : `typst::syntax::Source` compte les lignes à partir de 0
- export PDF : `typst-pdf` avec les options par défaut

## 9. Contrat du template

### 9.1 `template`

```typst
#let template(
  title: none,             // str | none
  subtitle: none,          // str | none
  authors: (),             // array de str
  teachers: (),            // array de str
  date: none,              // str | none
  school: none,            // str | none
  university: none,        // str | none
  academic-year: none,     // str | none
  cohort: none,            // str | none
  specialization: none,    // str | none
  subject: none,           // str | none
  toc: true,               // bool
  list-of-figures: false,  // bool
  list-of-listings: false, // bool
  header-text: none,       // str | none, déjà remplacé par title si absent
  body,
) = { … }
```

- les arguments sont en kebab-case (convention Typst) : le core convertit les noms de `Config`
- le core passe toujours **tous** les arguments ; le template doit pourtant garder ces défauts
- le template doit :
    - fonctionner quand n'importe quel argument vaut `none` ou `()`
    - régler `set text(lang: "fr")` et `set document(title: …, author: …)`
    - placer la page de garde, puis les tables demandées :
        - `toc` → `outline()`
        - `list-of-figures` → `outline(target: figure.where(kind: image))`
        - `list-of-listings` → `outline(target: figure.where(kind: raw))`
    - afficher `header-text` en haut de page
- le template peut : numéroter les titres, choisir le style de bibliographie (`set bibliography(style: …)`)
- le template ne doit pas lire `sys.inputs`

### 9.2 `callout`

- `#let callout(kind, body)`, `kind` ∈ `"note"`, `"tip"`, `"important"`, `"warning"`, `"caution"`

### 9.3 Ressources

- chemins relatifs à `template.typ` pour ses propres fichiers
- polices dans `fonts/` (section 8)

## 10. Tests

- core
    - unitaires : échappement (chaque caractère de la section 4.1), sérialisation des valeurs (section 2.3), slugs, attributs de code
    - snapshots `insta` : un cas par sous-section de la section 4, dans `tests/fixtures/<cas>/input.md`
    - chaque code de diagnostic de la section 2.4 : un test qui le déclenche et vérifie `line` et `column`
    - robustesse : `transpile` sur des entrées arbitraires ne panique pas
- CLI
    - découpage du frontmatter : octets et numéros de ligne conservés
    - fusion des couches et résolution des chemins, dans des dossiers temporaires
    - résolution des templates, dont le remplacement de `default`
    - chaque code de diagnostic de la section 5.6 qui ne dépend pas du réseau
    - bout en bout : `build examples/report.md` produit un PDF ; `init` produit un fichier qui se compile
- intégration des lots
    - un test du CLI compile avec le template par défaut le Typst de chaque fixture du core
    - il vérifie à la fois le code émis par le core et le respect du contrat de template

## 11. Ordre de travail

- étape 0 (avant tout le reste)
    - le lot core publie l'API de la section 2 compilable, avec un `transpile` minimal : préambule complet et paragraphes en texte échappé
    - le lot CLI peut alors brancher `build` de bout en bout pendant que le core avance
    - un template provisoire qui respecte la section 9 suffit pour compiler
- étape 1, en parallèle
    - core : section 4, sous-section par sous-section, avec ses tests
    - CLI : sections 5 à 8
- étape 2 : template par défaut, test d'intégration de la section 10, `examples/report.md` compilé sans avertissement

## 12. Hors périmètre de la v1

- `combava watch`
- variables libres transmises au template
- indices et exposants (`~x~`, `^x^`)
- typographie française automatique (espaces insécables avant `: ; ! ?`)
- images distantes, HTML
- plusieurs fichiers markdown dans un même document
- PDF/A
