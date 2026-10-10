# Architecture de Combava

Combava transforme un fichier markdown avec un frontmatter TOML en PDF, en passant par Typst. Combava est opinioné : un template convient tel quel à l'utilisateur, il n'expose pas de réglages libres.

Le détail du comportement et de l'interface entre le core et le CLI est dans [specification.md](specification.md), qui fait foi en cas de désaccord.

```
 rapport.md ──► combava-cli ──► combava-core ──► combava-cli ──► rapport.pdf
               (config, IO)    (md → Typst)     (Typst → PDF)
```

## Organisation du dépôt

- workspace cargo
    - `crates/combava-core` : bibliothèque, transpilation markdown → Typst
        - aucune lecture ni écriture de fichier, aucune dépendance à la crate `typst`
        - utilisable plus tard depuis WASM ou un autre client
        - `src/`
            - `lib.rs` : fonction `transpile`, constantes `paths` et `MITEX_PACKAGE`
            - `config.rs` : la `struct Config`
            - `output.rs` : `Output`, `Diagnostic`, `Severity` et `Code`
            - `error.rs` : le type d'erreur
            - `source_map.rs` : correspondance ligne Typst → ligne markdown, et tampon d'écriture qui la construit
            - `parser.rs` : options de pulldown-cmark, construction de l'arbre du document
            - `ast.rs` : l'arbre du document (blocs et éléments en ligne, chacun avec sa plage d'octets)
            - `diag.rs` : collecte des diagnostics, conversion des octets en lignes et colonnes
            - `label.rs` : labels des titres (`{#id}` et slugs)
            - `path.rs` : chemins des images
            - `escape.rs` : échappement du markup, littéraux de chaîne, décodage `%XX`
            - `emit/`
                - `mod.rs` : assemblage du fichier, état des notes de bas de page
                - `preamble.rs` : imports et `#show: template.with(…)`
                - `block.rs` : titres, paragraphes, listes, tableaux, code, figures, callouts…
                - `inline.rs` : emphase, liens, images en ligne, citations, notes, maths…
                - `code.rs` : info string des blocs de code (langage, `{caption="…"}`, `{=typst}`)
        - `tests/`
            - `transpile_snapshots.rs` : un cas par sous-section de la section 4 de la spécification (`fixtures/`, `snapshots/`)
            - `diagnostics.rs` : chaque code de diagnostic, avec sa ligne et sa colonne
            - `contract.rs` : garanties de l'API envers le CLI
            - `robustness.rs` : entrées aléatoires et imbrications profondes
    - `crates/combava-cli` : binaire `combava`
        - configuration, accès aux fichiers, compilation PDF
        - `src/`
            - `main.rs` : appelle `combava_cli::run`
            - `lib.rs` : point d'entrée, sous-commande `help`
                - bibliothèque interne, exposée seulement pour les tests d'intégration
            - `args.rs` : définition des commandes (`clap`), textes de l'aide en français
            - `commands/` : `build.rs`, `init.rs`, écriture atomique des fichiers de sortie
            - `frontmatter.rs` : découpage du bloc `+++`
            - `config/`
                - `mod.rs` : chargement des couches, résolution du template, de la bibliographie et de la sortie
                - `layers.rs` : lecture et fusion des couches
                - `paths.rs` : racine du projet, dossier de config globale, résolution des chemins relatifs
            - `template.rs` : résolution du template, templates embarqués
            - `compile/`
                - `mod.rs` : compilation, conversion des diagnostics Typst
                - `world.rs` : le `World` Typst et son système de fichiers virtuel
                - `pdf.rs` : export PDF
            - `diagnostics.rs` : diagnostics, positions, affichage
            - `error.rs` : codes de diagnostic du CLI
        - `build.rs` : recompile le binaire quand un template embarqué change
        - `tests/`
            - `cli_build.rs`, `cli_init.rs` : bout en bout, un test par code de diagnostic
            - `config_layers.rs` : fusion des couches et chemins relatifs
            - `template_resolution.rs` : ordre de recherche, remplacement de `default`
            - `core_fixtures.rs` : le Typst de chaque fixture du core se compile avec le template de test et avec le template par défaut
            - `fixtures/templates/minimal/` : template de test conforme au contrat
    - `templates/default/` : le template par défaut, embarqué dans le binaire (`include_dir`)
        - `template.typ` : point d'entrée, `template` et `callout`
        - `couverture.typ` : page de couverture
        - `composants.typ` : encadré des callouts
        - `utils.typ`, `theme.typ` : étiquettes et logos ; couleurs, polices et mesures
        - `images/` : `univ_logo.png` et `school_logo.png`, de fausses images à remplacer par les vrais logos
    - `examples/` : documents d'exemple

## combava-core

- point d'entrée : `transpile(markdown: &str, config: &Config) -> Result<Output, TranspileError>`
    - `markdown` : le contenu du fichier, frontmatter remplacé par des espaces par le CLI
    - `config` : la configuration fusionnée par le CLI
    - `Output`
        - `typst` : le code Typst complet
        - `diagnostics` : avertissements (élément ignoré, attribut inconnu…), chacun avec sa position dans le markdown
        - `source_map` : correspondance ligne Typst → ligne markdown, pour traduire les erreurs de compilation

- `Config` : une `struct`, pas un enum
    - tous les champs en `Option<T>`, sauf les booléens
    - construite par le CLI à partir des fichiers TOML (voir « combava-cli »)
    - champs (en anglais, mêmes noms que les clés TOML) :
        - `title`, `subtitle`
        - `authors`, `teachers` : listes de chaînes
        - `date` : chaîne affichée telle quelle (`date = "10 octobre 2026"`)
            - pourquoi : `datetime.display()` de Typst ne traduit pas les noms de mois
        - `school`, `university`, `academic_year`, `cohort` (promotion), `specialization`, `subject`
        - `toc` (table des matières), `list_of_figures`, `list_of_listings` (table des codes) : booléens
        - `header_text` : texte en haut de page
            - si absent, le core le remplace par `title` avant d'émettre le code
        - `bibliography` : booléen, vrai si le CLI fournit un fichier `.bib`

- parsing du markdown (pulldown-cmark)
    - options listées une par une, pas `Options::all()` (une nouvelle version de la crate pourrait activer une option non gérée)
        - activées :
            - `ENABLE_TABLES`, `ENABLE_FOOTNOTES`, `ENABLE_STRIKETHROUGH`, `ENABLE_TASKLISTS`
            - `ENABLE_HEADING_ATTRIBUTES`, `ENABLE_DEFINITION_LIST`
            - `ENABLE_GFM` (callouts `> [!NOTE]`)
            - `ENABLE_MATH`
            - `ENABLE_WIKILINKS`
        - désactivées :
            - `ENABLE_YAML_STYLE_METADATA_BLOCKS`, `ENABLE_PLUSES_DELIMITED_METADATA_BLOCKS` : le frontmatter est retiré par le CLI
            - `ENABLE_OLD_FOOTNOTES` : remplace la syntaxe GFM des notes de bas de page
            - `ENABLE_SMART_PUNCTUATION` : produirait des guillemets anglais, alors que Typst produit « » avec `lang: "fr"`
            - `ENABLE_SUBSCRIPT`, `ENABLE_SUPERSCRIPT` : `~x~` resterait du barré en GFM

- génération du code Typst
    - structure du fichier généré :
        - `#import "@preview/mitex:0.2.7": mi, mitex` (seulement si le document contient des maths)
        - `#import "/__combava__/template/template.typ": template, callout`
        - `#show: template.with(title: "…", authors: ("…", "…"), toc: true, …)`
            - chaque valeur est sérialisée en littéral Typst échappé
            - une clé absente est passée à `none`
        - le corps transpilé
        - `#bibliography("/__combava__/bibliography.bib")` si `bibliography` est défini
    - correspondance markdown → Typst :
        - titres → `=`, `==`… ; `{#id}` → label `<id>`
        - emphase, gras, barré → `#emph[…]`, `#strong[…]`, `#strike[…]`
        - code en ligne → `#raw("…")`
        - bloc de code → `#raw(block: true, lang: "…", "…")`
            - avec une légende (` ```rust {caption="Tri rapide"} `) → `#figure(raw(…), caption: […])`
            - seuls les blocs légendés apparaissent dans la table des codes
        - ` ```{=typst} ` → contenu recopié tel quel (échappatoire pour ce que le markdown n'exprime pas)
        - listes, listes de tâches (☐ / ☒), listes de définitions → `#list`, `#enum`, `#terms`
        - liens
            - externes → `#link("url")[…]`
            - internes `[texte](#id)` → `#link(<id>)[…]`
            - wikilinks `[[cible]]` → `#link(label("cible"))[cible]`
        - images
            - seule dans son paragraphe → `#figure(image("…", alt: "…"), caption: […])`, la légende étant le titre de l'image (`![alt](img.png "Légende")`)
            - dans une phrase → `#box(image("…"))`
            - chemins relatifs au `.md`, émis avec un `/` de tête (relatifs à la racine Typst)
            - image distante (`http…`) → erreur : Typst ne télécharge pas
        - tableaux → `#table(columns: …, align: …, table.header(…), …)`
        - notes de bas de page → `#footnote[…]`
            - deux passes : une définition peut apparaître après son appel
        - citations `[@cle]`, `[@a; @b]` → `#cite(<cle>)`
        - citations en bloc → `#quote(block: true)[…]`
        - callouts GFM `> [!NOTE]` → `#callout("note")[…]`
        - maths (LaTeX) : inline `$…$` → `#mi("…")`, display `$$…$$` → `#mitex("…")`
        - lignes horizontales (`---`, `***`, `___`, `- - -`…) → `#pagebreak()` (`#line(length: 100%)` dans un conteneur)
            - attention : une ligne `---` placée juste sous un paragraphe en fait un titre setext (H2) ; il faut une ligne vide avant
        - HTML brut → ignoré avec un avertissement ; les commentaires `<!-- … -->` sont ignorés sans avertissement
    - échappement du texte (module critique, très testé)
        - caractères préfixés par `\` : `\ # $ * _ @ < > [ ] ` ~ / = - +`, et tout `.` précédé d'un chiffre
        - `"` et `'` restent tels quels : Typst les transforme en guillemets français

## combava-cli

- configuration
    - frontmatter
        - bloc TOML délimité par `+++`, en tout début de fichier
        - le CLI remplace chacun de ses octets par une espace, sauf les `\n`, avant d'appeler le core
            - pourquoi : les octets, lignes et colonnes des diagnostics restent ceux du fichier
    - couches, de la plus prioritaire à la moins prioritaire :
        1) arguments de la ligne de commande (`-o`, `-t`)
        2) frontmatter
        3) `.combava/config.toml` du projet
            - cherché en remontant les dossiers à partir du dossier du `.md`, comme git
            - le dossier qui contient `.combava/` est la racine du projet
        4) `<config globale>/config.toml`
            - dossier donné par la crate `directories` : `~/.config/combava` sous Linux, `%APPDATA%\combava` sous Windows, `~/Library/Application Support/combava` sous macOS
        5) valeurs par défaut
    - chaque couche est lue avec `toml::de::DeTable`, qui garde la position de chaque clé et de chaque valeur, puis validée à la main
        - pourquoi pas `#[serde(deny_unknown_fields)]` : les erreurs de serde sont en anglais et ne distinguent pas `unknown-key` de `invalid-type`
        - une clé inconnue est une erreur, avec la clé fautive et sa position
        - toutes les erreurs de toutes les couches sont rapportées en une fois
    - fusion champ par champ (`a.or(b)`) ; une liste (`authors`…) est remplacée, jamais concaténée
    - le résultat fusionné donne la `Config` du core et les clés propres au CLI
    - clés propres au CLI, en plus de celles de `Config` :
        - `output` : fichier PDF, dans le frontmatter uniquement (par défaut, même nom que le `.md` en `.pdf`, à côté du `.md`)
        - `template` : nom ou chemin du template (par défaut `default`)
        - `bibliography` : chemin du fichier `.bib`
    - un chemin relatif est résolu par rapport au fichier qui le déclare :
        - frontmatter → dossier du `.md`
        - `.combava/config.toml` → racine du projet
        - configuration globale → dossier de configuration globale
        - `~` en tête → dossier personnel, dans toutes les couches

- résolution du template
    - un template est un dossier contenant `template.typ` (et ses ressources : images, polices…)
    - valeur contenant un `/`, ou commençant par `.` ou `~` → chemin vers ce dossier
    - sinon, nom de dossier, cherché dans l'ordre :
        1) `<racine du projet>/.combava/templates/<nom>`
        2) `<config globale>/templates/<nom>`
        3) templates embarqués dans le binaire (`default`)
    - dossier introuvable, ou sans `template.typ` → erreur listant les emplacements essayés

- compilation (crates `typst`, `typst-pdf`, `typst-kit`)
    - `World` maison :
        - `/` → dossier du `.md`, en lecture seule
            - un chemin qui en sort (`../`) est refusé : Typst normalise les chemins virtuels
        - `/__combava__/main.typ` → code généré, en mémoire
        - `/__combava__/template/` → dossier du template résolu
        - `/__combava__/bibliography.bib` → fichier `.bib` résolu
    - polices : dossier `fonts/` du template, puis polices système, puis polices embarquées de `typst-kit`
    - packages (`mitex`) : téléchargés au premier usage puis mis en cache par `typst-kit`
    - erreurs Typst
        - dans `main.typ` → ligne traduite vers le markdown grâce à `source_map`
        - dans le template → affichées telles quelles, avec le chemin réel du fichier

- commandes
    - `combava build [OPTIONS] FILE.md`
        - `-o`, `--out FILE` : surcharge `output`
        - `-t`, `--template TEMPLATE` : surcharge `template`
        - `--transpile-only` : écrit seulement le `.typ` (même destination que `output`, en `.typ`)
            - sert au débogage : le fichier importe le template sans le copier, il n'est pas garanti compilable seul
        - code de sortie non nul en cas d'erreur
    - `combava init [OPTIONS] [FILENAME]`
        - crée un fichier markdown avec un frontmatter minimal ; `FILENAME` vaut `./rapport.md` par défaut
        - refuse d'écraser un fichier existant, sauf avec `--force`
        - `-t`, `--template TEMPLATE` : ajoute la clé `template` au frontmatter
        - frontmatter généré :
            - `title`, `subtitle`, `authors`, `teachers` : valeurs à remplir
            - `date` : date du jour en français (`10 octobre 2026`)
            - `toc = true`, `list_of_figures = false`, `list_of_listings = false`
            - `header_text` : en commentaire, pour qu'il suive `title` tant qu'il n'est pas défini
    - `combava help`, `--help`, `--version` : générés par `clap`

## Contrat du template

- `template.typ` exporte :
    - `template(title: none, subtitle: none, authors: (), teachers: (), date: none, school: none, university: none, academic-year: none, cohort: none, specialization: none, subject: none, toc: true, list-of-figures: false, list-of-listings: false, header-text: none, body)`
        - arguments en kebab-case (convention Typst), convertis par le core depuis les noms de `Config`
        - règle la langue (`set text(lang: "fr")`) et les métadonnées du PDF (`set document(…)`)
        - place la page de garde et les tables demandées
    - `callout(kind, body)` : `kind` vaut `"note"`, `"tip"`, `"important"`, `"warning"` ou `"caution"`
- le template importe ses propres ressources par des chemins relatifs à `template.typ`
- toute modification de ces signatures casse les templates des utilisateurs : elle doit être signalée dans le CHANGELOG

## Tests

- core
    - tests unitaires : échappement, sérialisation des valeurs, chaque élément markdown
    - snapshots (`insta`) : `tests/fixtures/<cas>/input.md` → Typst attendu, rangé dans `tests/snapshots/`
- cli
    - fusion des couches de configuration, résolution des templates : par la bibliothèque, avec un environnement (`Env`) injecté
    - bout en bout (`assert_cmd`, `tempfile`) : `build` produit un PDF, `init` refuse d'écraser, chaque code de diagnostic
        - sous Linux, `XDG_CONFIG_HOME` pointe vers un dossier temporaire : la configuration globale de la machine n'influe pas sur les tests
        - `package-download` est simulé sans réseau : caches de packages vides et proxy injoignable
    - template par défaut : `examples/report.md`, un document sans aucun champ, des titres dans des conteneurs (citation, liste, callout, note)

## Dépendances

- core : `pulldown-cmark`, `thiserror`
- cli : `clap`, `toml`, `directories`, `include_dir`, `chrono` (date du jour), `typst`, `typst-layout`, `typst-pdf`, `typst-kit`
- tests : `insta`, `assert_cmd`, `tempfile`
