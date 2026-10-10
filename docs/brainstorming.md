# Architecture of Combava

The present document describes the architecture of the tool and the pipeline we use / will use in order to make it work.


# Brainstorming

## 1. Pipeline markdown → Typst → PDF

| Option | Principe | Avantages | Limites |
|---|---|---|---|
| **A. Pandoc + template Typst** | `pandoc -t typst --template=...`, filtres Lua | Prototype rapide, frontmatter YAML géré | Grosse dépendance, parti pris limité par les filtres |
| **B. `cmarker` (dans Typst)** | Le markdown est rendu directement par Typst | Aucune étape intermédiaire | Frontmatter à parser soi-même |
| **C. Transpileur maison** | Parse le markdown en AST, génère du Typst, compile | Contrôle total, binaire unique | Plus de travail initial |

**Décision : option C (transpileur maison).** Pandoc est écarté, même pour un premier prototype : le frontmatter YAML et les filtres Lua borneraient trop vite ce qu'on peut exprimer (schéma strict, lint, callouts, typographie française, erreurs pointant la source). Mieux vaut partir directement sur la bonne architecture que valider un rendu sur un pipeline qu'on jettera.

**Stacks possibles pour l'option C**
- **Rust** : `comrak` / `pulldown-cmark`, `serde_yaml`, crate `typst` ou `typst-as-lib`
- **TypeScript** : `unified` + `remark` (+ `remark-frontmatter`, `remark-gfm`), `typst.ts`
- **Python** : `markdown-it-py` / `mistletoe`, `typst-py`
- **Go** : `goldmark` + appel du CLI `typst`

## 2. Astuce clé : émettre des fonctions sémantiques

Au lieu de convertir `## Titre` en `== Titre`, le transpileur génère :

```typst
#callout(kind: "warning")[...]
#figure-block(src: "img.png", caption: "...")
#section(level: 2)[Titre]
```

Le design vit entièrement dans une bibliothèque Typst : on change l'esthétique sans toucher au parseur, et on peut proposer plusieurs thèmes.

## 3. Idées de « parti pris »

- **Structure et métadonnées**
  - Champ `type:` (`report`, `memo`, `letter`, `spec`, `cv`, `one-pager`) qui choisit un template complet
  - Schéma de frontmatter strict (refus de compiler si `title` ou `author` manque)
  - Page de garde, table des matières, en-têtes et pieds de page automatiques
  - Métadonnées PDF remplies depuis le frontmatter
- **Contraintes assumées**
  - Une seule paire de polices, une grille typographique fixe
  - Profondeur de titres limitée (h1 à h3)
  - Lint : tableau trop large, image sans alt, h1 en double
- **Typographie**
  - Règles françaises (insécables, guillemets « », tirets cadratins)
  - Veuves et orphelines, grille de lignes de base, ligatures
- **Éléments riches**
  - Callouts GFM (`> [!NOTE]`), `---` comme saut de page
  - Coloration syntaxique, diagrammes (mermaid, `cetz`, `fletcher`), maths (`mitex`)
  - Bibliographie, notes de bas de page, QR codes, filigrane `status: draft`
- **Qualité du PDF**
  - PDF/A, PDF accessible, polices embarquées

## 4. Expérience développeur

- Commandes `build`, `watch`, `init`
- Erreurs qui pointent la ligne du markdown source
- Binaire unique ou WASM pour un aperçu web
- Thèmes partageables comme packages Typst
- Intégration CI (GitHub Action)

## 5. Personnalisation après compilation du binaire

Le transpileur est figé dans le binaire, mais **le design est chargé au runtime**.

**Trois niveaux**
1. **`theme.toml`** : paramètres exposés volontairement (polices, couleurs, marges), injectés via `sys.inputs` ou un `config.typ` généré
2. **Surcharge de fichiers `.typ`** : ordre de recherche `./monoutil/theme/`, puis `~/.config/monoutil/themes/`, puis thème embarqué (`include_str!`, `//go:embed`)
3. **Thèmes complets** : `--theme ./mon-theme/` ou `theme:` dans le frontmatter

**Technique**
- Rust : implémenter le trait `World` de Typst (résolution des fichiers, polices, packages)
- Autres langages : appeler le CLI `typst` en pointant vers un dossier de thème
- Prévoir un dossier `fonts/` chargeable au runtime

**Contrat transpileur ↔ thème**
- Documenter et versionner la liste des fonctions requises (`callout`, `figure-block`, `section`, `cover`…)
- Champ `api = 1` dans le manifeste, erreur claire si une fonction manque

**Recommandation** : combiner `theme.toml` (réglages courants) et surcharge `.typ` (usages avancés).

## 6. Configuration de l'outil

Distinction à garder en tête : le **thème** (section 5) règle l'*apparence* du PDF, la **configuration** règle le *comportement* de l'outil (type de document par défaut, auteur, date automatique…).

**Fichiers de configuration**
- Global : `~/.config/combava/config.toml` (convention XDG)
- Projet : `./combava.toml`, qui surcharge le global pour un dépôt donné
- Ordre de priorité : option CLI > frontmatter du document > `combava.toml` > config globale > valeurs par défaut embarquées

**Exemple de `config.toml`**

```toml
default_template = "report"   # type utilisé par `init` et `build` si `type:` est absent
author = "Prénom Nom"         # injecté si `author` manque dans le frontmatter
auto_date = true              # date du jour si `date` manque dans le frontmatter
date_format = "%d %B %Y"
lang = "fr"                   # active les règles typographiques françaises
theme = "default"
output_dir = "build/"
strict = true                 # les warnings du lint deviennent bloquants
```

**Commande `combava config`**

```sh
combava config set default_template memo   # écrit dans la config globale
combava config set --local author "Alice"  # écrit dans ./combava.toml
combava config get default_template
combava config list                         # valeurs effectives + origine de chacune
combava config unset auto_date
combava config path                         # affiche les fichiers lus
combava config edit                         # ouvre dans $EDITOR
```

**Options envisagées**
- **Document** : `default_template`, `author`, `auto_date`, `date_format`, `lang`, `status` par défaut (`draft`)
- **Rendu** : `theme`, `output_dir`, `pdf_standard` (PDF/A), `fonts_dir`
- **Lint** : `strict`, liste de règles activées ou désactivées
- **Templates de départ** : chemin vers un squelette markdown personnalisé pour `init` (par exemple `~/.config/combava/templates/report.md`), même ordre de recherche que les thèmes (projet, puis global, puis embarqué)

**Comportement**
- `init` génère un frontmatter prérempli avec `author` et `date` issus de la config
- `auto_date` ne remplit que les champs absents : le frontmatter l'emporte toujours
- Clés inconnues ou valeurs invalides : erreur claire avec le nom de la clé, jamais d'ignorance silencieuse
- `config set` valide la valeur avant d'écrire (par exemple `default_template` doit désigner un template existant)

**Technique**
- Rust : `serde` + `toml`, crate `directories` pour les chemins XDG, `toml_edit` pour que `config set` préserve les commentaires et la mise en forme du fichier
- Fusion des niveaux : chaque couche est un `Option<T>` fusionné dans l'ordre de priorité, ce qui permet à `config list` d'afficher l'origine de chaque valeur
- Les paramètres de design restent dans `theme.toml` ; la config ne fait que *choisir* le thème

**Questions ouvertes**
- Faut-il autoriser `author` à être une liste (documents à plusieurs auteurs) ?
- Variables d'environnement (`COMBAVA_AUTHOR`…) pour la CI, en plus des fichiers ?
- `combava.toml` doit-il pouvoir déclarer plusieurs profils (`[profile.school]`, `[profile.work]`) ?

## 7. Estimation du transpileur maison

Hypothèse : Rust, avec `pulldown-cmark`, `serde`, `clap` et la crate `typst` (ou `typst-as-lib`). Ce sont des ordres de grandeur, pas des mesures.

| Brique | Lignes de Rust |
|---|---|
| Frontmatter (parsing YAML, schéma strict, erreurs) | 200–300 |
| AST markdown → Typst (titres, listes, tableaux, code, images, liens, notes, callouts, échappement des caractères spéciaux Typst) | 600–1 000 |
| Compilation Typst (`typst-as-lib` : ~100–200 ; trait `World` maison : 400–600) | 100–600 |
| CLI (`build`, `init`, `watch`) | 300–500 |
| Chargement de thèmes runtime + vérification du contrat `api` | 200–300 |
| Configuration (`config.toml`, commande `combava config`) | 300–500 |
| Erreurs qui pointent la ligne du markdown source | 200–400 |
| Lint | 200–400 |
| Typographie française | 150–300 |
| Tests | 500–1 000 |

- **Première version utile** : `build` seul, `report` et `memo`, frontmatter, sans config, lint ni watch. **1 500 à 2 500 lignes de Rust**, plus **400 à 800 lignes de Typst** pour le thème, qui est un vrai travail à part.
- **Version complète** (tout le brainstorming) : **4 000 à 6 000 lignes de Rust**, plus 1 000 à 1 500 lignes de Typst.

**Points qui font varier l'estimation**
- L'échappement et le mapping des nœuds markdown vers Typst est la partie la plus fastidieuse : c'est là que se trouvent les cas limites (`#`, `$` et `*` dans le texte, tableaux à cellules multilignes…).
- Appeler le CLI `typst` au lieu de l'embarquer économise 300 à 500 lignes et reste correct pour une v1. On y perd le binaire unique, qu'on pourra ajouter ensuite.

## 8. Plan de démarrage

1. Écrire à la main un petit thème Typst (`report`) exposant les fonctions sémantiques (`section`, `callout`, `figure-block`, `cover`…) et le valider avec le CLI `typst` sur un document Typst écrit à la main
2. Transpileur minimal : frontmatter + AST markdown → appels de fonctions sémantiques → PDF (`combava build`)
3. Commencer avec 1 ou 2 types de documents (`report`, `memo`)
4. Ajouter le chargement de thèmes runtime et le contrat versionné
5. Ajouter la configuration (`config.toml`, commande `combava config`) une fois `init` et `build` stables
