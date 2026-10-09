# Architecture de Combava

Ce document décrit l'architecture du code : les modules, leurs responsabilités et les règles de dépendance entre eux. Les choix de fond (pourquoi un transpileur maison, quelles fonctionnalités viser) sont dans [brainstorming.md](brainstorming.md).

## 1. Vue d'ensemble

Combava transforme un fichier markdown avec frontmatter en PDF, en passant par Typst. Le code est un **crate unique** composé d'une bibliothèque (`lib.rs`) et d'un binaire fin (`main.rs`).

- `lib.rs` contient toute la logique. Les tests d'intégration appellent la bibliothèque directement, sans lancer de processus.
- `main.rs` parse les arguments, appelle la bibliothèque et affiche les erreurs.

Si un aperçu WASM devient nécessaire, le crate pourra être scindé en workspace (`combava-core` et `combava-cli`) sans réécriture : les modules ne dépendent déjà pas du terminal, sauf `cli/`.

## 2. Le pipeline

```
 fichier .md
     │
     ▼
 config ──────────────► valeurs par défaut (auteur, type, date…)
     │
     ▼
 document      sépare le frontmatter du corps, valide le schéma du type de document
     │
     ▼
 markdown      parse le corps (comrak) puis l'abaisse vers l'IR
     │
     ▼
 ir            modèle de document propre à Combava, chaque nœud porte son Span
     │
     ▼
 lint          règles sur l'IR (h1 en double, image sans alt, tableau trop large…)
     │
     ▼
 transpile     typographie française, puis émission de Typst sémantique
     │
     ▼
 theme         fournit les fonctions Typst (section, callout…) et vérifie le contrat
     │
     ▼
 compile       compile le Typst avec le thème et produit le PDF
     │
     ▼
 fichier .pdf
```

`pipeline.rs` orchestre ces étapes dans l'ordre. Il est le seul module qui connaît toute la chaîne ; chaque étape ne connaît que son entrée et sa sortie.

## 3. Principes directeurs

**Une représentation intermédiaire (IR) entre comrak et Typst.**
Le markdown est converti dès l'entrée en types propres à Combava (`Block`, `Inline`), qui portent chacun un `Span` (position dans le fichier source). Tout ce qui suit travaille sur l'IR, jamais sur l'AST de comrak. Conséquences :
- on peut changer de parseur markdown sans toucher au reste ;
- les erreurs peuvent pointer la ligne du markdown source, à n'importe quelle étape ;
- les règles de lint et la typographie se testent sur des structures construites à la main.

**Le seul couplage avec le design est le contrat du thème.**
`transpile/` émet des appels à des fonctions sémantiques (`#callout(...)`, `#section(...)`) dont il ne connaît que le nom et la signature. Le thème fournit ces fonctions. Changer l'esthétique ne demande aucune modification du code Rust (voir la section 6).

**Thème et configuration sont deux choses distinctes.**
Le thème règle l'apparence du PDF (`theme.toml`, fichiers `.typ`). La configuration règle le comportement de l'outil (`config.toml`). La configuration ne fait que *choisir* un thème.

**Chaque étape retourne un résultat, jamais un `panic`.**
Les erreurs sont typées (`error.rs`) et rendues de façon lisible par `diagnostics/`.

## 4. Règles de dépendance

Les flèches signifient « peut utiliser ».

```
cli ──► pipeline ──► config, document, markdown, lint, transpile, theme, compile
                           │
 markdown ──► ir           │
 lint ──────► ir           ├──► diagnostics, error (utilisés par tous)
 transpile ─► ir, theme (contrat uniquement)
 compile ───► theme
```

- `ir` ne dépend de rien d'autre que de la bibliothèque standard.
- `lint`, `transpile` et `markdown` ne se connaissent pas entre eux.
- Seul `cli/` affiche quelque chose dans le terminal. Les autres modules retournent des valeurs et des diagnostics.
- `compile/` est le seul module qui dépend de la crate `typst`.

## 5. Les modules

### `cli/`
Interface en ligne de commande, construite avec `clap`.
- `args.rs` : définition des commandes et options.
- `build.rs`, `init.rs`, `watch.rs` : une commande par fichier. Elles traduisent les arguments en appel de la bibliothèque.
- `config_cmd.rs` : sous-commandes `combava config set|get|list|unset|path|edit`.

### `config/`
Configuration de l'outil, à plusieurs niveaux (voir la section 6 de [brainstorming.md](brainstorming.md)).
- `model.rs` : la liste des clés connues et leurs types.
- `layers.rs` : fusion des couches (CLI > frontmatter > `combava.toml` > global > défauts embarqués), avec mémorisation de l'origine de chaque valeur pour `config list`.
- `store.rs` : lecture et écriture du fichier avec `toml_edit`, pour que `config set` préserve les commentaires.
- `paths.rs` : emplacements des fichiers (XDG).

### `document/`
Tout ce qui concerne les métadonnées du document.
- `frontmatter.rs` : sépare le frontmatter du corps et le désérialise.
- `doc_type.rs` : l'énumération des types (`report`, `memo`…).
- `schema.rs` : champs obligatoires et facultatifs par type. Refuse de continuer si `title` ou `author` manque.

### `markdown/`
- `parser.rs` : appelle comrak avec les extensions voulues (GFM, notes de bas de page).
- `lower.rs` : parcourt l'AST de comrak et construit l'IR en calculant les `Span`.
- `callout.rs` : reconnaît les callouts GFM (`> [!NOTE]`) et les transforme en nœud dédié de l'IR.

### `ir/`
Le modèle de document.
- `block.rs` : titres, paragraphes, listes, tableaux, blocs de code, figures, callouts, saut de page.
- `inline.rs` : texte, emphase, code en ligne, liens, images, notes de bas de page.
- `span.rs` : position dans le fichier source (octets, ligne, colonne).

### `lint/`
- `rule.rs` : le trait qu'implémente toute règle (entrée : l'IR ; sortie : une liste de diagnostics).
- `rules/` : une règle par fichier (`heading_depth`, `duplicate_h1`, `image_alt`, `table_width`). Pour ajouter une règle : créer un fichier, puis l'enregistrer dans `rules/mod.rs`.
- `mod.rs` : exécute les règles activées par la configuration.

### `transpile/`
- `typography.rs` : règles françaises (espaces insécables, guillemets « », tirets cadratins) appliquées aux nœuds texte de l'IR.
- `escape.rs` : échappement des caractères spéciaux de Typst (`#`, `$`, `*`, `_`, `@`…). Module critique, très testé.
- `emitter.rs` : parcourt l'IR et écrit le Typst, en n'émettant que des appels aux fonctions sémantiques du contrat.

### `theme/`
- `manifest.rs` : lecture de `theme.toml` (nom, version de l'API, polices, couleurs, marges).
- `contract.rs` : liste versionnée des fonctions que tout thème doit fournir. Erreur claire si l'une manque ou si `api` ne correspond pas.
- `loader.rs` : résolution du thème dans l'ordre projet (`./combava/theme/`), global (`~/.config/combava/themes/`), embarqué.
- `embedded.rs` : le thème par défaut, inclus dans le binaire à la compilation.

### `compile/`
- `world.rs` : implémentation du trait `World` de Typst (résolution des fichiers, sources, date).
- `fonts.rs` : chargement des polices du thème, puis du système.
- `packages.rs` : résolution des paquets Typst (`cetz`, `fletcher`, `mitex`…).
- `pdf.rs` : export PDF, métadonnées issues du frontmatter, PDF/A.

### `diagnostics/` et `error.rs`
- `error.rs` : le type d'erreur du crate.
- `diagnostics/` : un diagnostic est un message, une gravité et un `Span`. `report.rs` le rend avec l'extrait de source souligné. Les diagnostics sont communs au lint, au frontmatter, au thème et à la compilation Typst (dont les positions sont ramenées au markdown quand c'est possible).

### `pipeline.rs`
Enchaîne les étapes et accumule les diagnostics. Les avertissements de lint deviennent bloquants quand `strict = true`.

## 6. Le thème

`themes/default/` est un thème Typst complet, embarqué dans le binaire et remplaçable à l'exécution.

```
themes/default/
├── theme.toml          manifeste (nom, api = 1, réglages exposés)
├── lib.typ             point d'entrée, réexporte les fonctions du contrat
├── report.typ          template complet du type `report`
├── memo.typ            template complet du type `memo`
├── components/         une fonction sémantique par fichier
│   ├── section.typ
│   ├── callout.typ
│   ├── figure-block.typ
│   ├── cover.typ
│   ├── table.typ
│   └── code.typ
└── fonts/              polices embarquées
```

Le **contrat** est la liste des fonctions de `components/` avec leurs paramètres, plus les templates par type de document. Il est décrit dans `theme/contract.rs` et versionné par le champ `api` du manifeste. Faire évoluer le contrat de façon incompatible impose d'incrémenter `api`.

Les squelettes markdown de `combava init` sont dans `templates/` (`report.md`, `memo.md`), hors du thème : ils décrivent le contenu à écrire, pas l'apparence.

## 7. Stratégie de tests

| Niveau | Où | Ce qui est testé |
|---|---|---|
| Unitaire | `#[cfg(test)]` dans `src/` | échappement Typst, règles typographiques, une règle de lint isolée |
| Par étape | `tests/frontmatter.rs`, `markdown_lowering.rs`, `lint.rs`, `config.rs`, `theme_contract.rs` | chaque module via son API publique |
| Snapshots | `tests/transpile_snapshots.rs` | markdown d'entrée → Typst émis, comparé à `tests/fixtures/*/expected.typ` |
| Bout en bout | `tests/cli_build.rs`, `cli_init.rs`, `cli_config.rs` | le binaire, avec un dossier temporaire |

- `tests/fixtures/report/` et `memo/` contiennent une entrée et la sortie Typst attendue.
- `tests/fixtures/invalid/` contient des documents qui doivent échouer avec une erreur précise (titre manquant, h1 en double, image sans alt).
- `tests/common/mod.rs` regroupe les utilitaires partagés.
- Les tests de bout en bout vérifient que le PDF est produit et que son nombre de pages et ses métadonnées sont corrects, pas son rendu pixel par pixel.

## 8. Dépendances envisagées

| Besoin | Crate |
|---|---|
| Parsing markdown | `comrak` |
| Frontmatter YAML | à choisir : `serde_yaml` est archivé, comparer `serde_yml`, `serde_norway` ou `saphyr` |
| Sérialisation | `serde` |
| Configuration | `toml`, `toml_edit`, `directories` |
| CLI | `clap` |
| Erreurs | `thiserror`, plus `miette` ou `ariadne` pour le rendu des diagnostics |
| Typst | `typst`, `typst-pdf`, ou `typst-as-lib` pour démarrer plus vite |
| Thème embarqué | `include_dir` ou `rust-embed` |
| Mode `watch` | `notify` |
| Tests | `insta`, `assert_cmd`, `tempfile` |

## 9. Décisions ouvertes

- **Crate unique ou workspace dès le départ** : crate unique pour commencer, scission si WASM.
- **Appeler le CLI `typst` ou embarquer la crate** : le premier est plus simple pour une v1 et ne touche que `compile/` ; le second donne le binaire unique.
- **Correspondance des erreurs Typst vers le markdown** : demande de conserver une table de correspondance entre lignes émises et `Span` de l'IR. À prévoir dans `emitter.rs` dès le début, car c'est difficile à ajouter après coup.
- **Choix de la bibliothèque de diagnostics** (`miette` ou `ariadne`).
