# Guide d'utilisation

Combava transforme un fichier markdown en rapport PDF. On écrit le contenu en markdown, on règle la couverture dans un en-tête, et combava s'occupe de la mise en page : couverture, table des matières, numérotation, bibliographie.

Ce guide s'adresse aux personnes qui écrivent des documents avec combava. Le comportement exact de chaque fonction est décrit dans la [spécification](specification.md).

- [Installation](#installation)
- [Premier document](#premier-document)
- [L'en-tête du document](#len-tête-du-document)
- [Configuration partagée](#configuration-partagée)
- [Écrire en markdown](#écrire-en-markdown)
- [Bibliographie](#bibliographie)
- [Templates](#templates)
- [Erreurs et avertissements](#erreurs-et-avertissements)
- [Questions fréquentes](#questions-fréquentes)

## Installation

### Avec `make` (Linux, macOS)

Il faut [Rust](https://www.rust-lang.org/tools/install) (version stable) et `make`. Depuis la racine du dépôt :

```bash
make install
```

Cette commande installe :

| Quoi | Où |
|---|---|
| le binaire `combava` | `~/.local/bin/combava` |
| la configuration globale | `~/.config/combava/config.toml` |
| le template par défaut | `~/.config/combava/templates/default/` |

Sous macOS, la configuration et le template vont dans `~/Library/Application Support/combava/`.

Si `~/.local/bin` n'est pas dans votre `PATH`, `make install` vous prévient. Ajoutez alors cette ligne à `~/.bashrc` ou `~/.zshrc` :

```bash
export PATH="$HOME/.local/bin:$PATH"
```

Vérifiez l'installation :

```bash
combava --version
```

### Mettre à jour

Récupérez la dernière version du dépôt, puis relancez `make install`. Le binaire est remplacé, mais la configuration et le template déjà installés sont conservés : vous avez pu les modifier.

Pour remplacer aussi la configuration et le template :

```bash
make install FORCE=1
```

> [!WARNING]
> `FORCE=1` écrase vos modifications, y compris les logos que vous avez mis dans le template. Faites-en une copie avant.

### Désinstaller

```bash
make uninstall
```

Le binaire est retiré. La configuration reste en place : supprimez `~/.config/combava/` à la main si vous n'en voulez plus.

### Sans `make` (Windows)

```bash
cargo install --path crates/combava-cli
```

Le binaire est installé dans `~/.cargo/bin`. Il contient le template par défaut, mais aucune configuration globale n'est créée. La configuration globale se trouve dans `%APPDATA%\combava\config\config.toml`.

## Premier document

```bash
combava init rapport.md
```

`init` crée un fichier prêt à remplir, avec la date du jour :

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

Remplissez l'en-tête, écrivez votre texte sous `# Introduction`, puis compilez :

```bash
combava build rapport.md
```

Le PDF est écrit à côté du fichier markdown : `rapport.pdf`.

Options de `build` :

| Option | Effet |
|---|---|
| `-o`, `--out <FICHIER>` | écrit le PDF ailleurs ; le dossier doit exister |
| `-t`, `--template <TEMPLATE>` | utilise un autre template (voir [Templates](#templates)) |
| `--transpile-only` | écrit le code Typst généré (`.typ`) au lieu du PDF, pour comprendre une erreur |

Options de `init` :

| Option | Effet |
|---|---|
| `[FICHIER]` | nom du fichier à créer ; `rapport.md` par défaut |
| `-t`, `--template <TEMPLATE>` | ajoute la clé `template` à l'en-tête |
| `--force` | écrase le fichier s'il existe déjà |

`combava help`, `combava help build` et `combava --help` affichent l'aide.

> [!TIP]
> Si la compilation échoue, l'ancien PDF reste intact : combava ne remplace le fichier que lorsque tout a réussi.

## L'en-tête du document

L'en-tête (ou *frontmatter*) est le bloc entre les deux lignes `+++` au tout début du fichier. Il est écrit en [TOML](https://toml.io/fr/) : `clé = valeur`, une clé par ligne, `#` pour un commentaire.

### Clés

| Clé | Type | Rôle | Défaut |
|---|---|---|---|
| `title` | texte | titre (couverture, métadonnées du PDF) | aucun |
| `subtitle` | texte | sous-titre | aucun |
| `authors` | liste de textes | auteurs | `[]` |
| `teachers` | liste de textes | enseignants ou encadrants | `[]` |
| `date` | texte | date affichée, telle quelle | aucune |
| `school` | texte | école | aucune |
| `university` | texte | université | aucune |
| `academic_year` | texte | année universitaire, par exemple `"2026-2027"` | aucune |
| `cohort` | texte | promotion | aucune |
| `specialization` | texte | spécialité ou filière | aucune |
| `subject` | texte | matière ou type de document | aucun |
| `toc` | booléen | table des matières | `true` |
| `list_of_figures` | booléen | liste des figures | `false` |
| `list_of_listings` | booléen | liste des codes | `false` |
| `header_text` | texte | texte en haut de chaque page | le titre |
| `bibliography` | chemin | fichier `.bib` (voir [Bibliographie](#bibliographie)) | aucun |
| `template` | nom ou chemin | template à utiliser | `"default"` |
| `output` | chemin | fichier PDF à écrire ; **uniquement dans l'en-tête** | `<nom du .md>.pdf` |

Une clé absente n'est simplement pas affichée.

### Pièges courants

- **Une clé mal orthographiée est une erreur.** `autors = [...]` arrête la compilation avec `clé inconnue « autors »`, au lieu de produire un PDF sans auteur.
- **La date est un texte, entre guillemets.** `date = 2026-10-10` est une date TOML, refusée. Écrivez `date = "10 octobre 2026"` : elle sera affichée telle quelle.
- **Les listes s'écrivent entre crochets**, même avec un seul élément : `authors = ["Ada Lovelace"]`.
- **Les valeurs sont du texte brut.** `title = "*Mon* titre"` affiche les astérisques.

Toutes les erreurs de l'en-tête sont affichées en une seule fois, avec leur ligne.

## Configuration partagée

Certaines valeurs reviennent dans tous vos documents : école, université, promotion… Inutile de les répéter dans chaque en-tête.

### Les quatre couches

Combava lit la configuration à quatre endroits. Pour chaque clé, **le premier qui la définit l'emporte** :

1. les options `-o` et `-t` de la ligne de commande ;
2. l'en-tête du document ;
3. le fichier `.combava/config.toml` du projet ;
4. la configuration globale, `~/.config/combava/config.toml`.

Une clé définie nulle part prend sa valeur par défaut.

Les listes ne se cumulent pas : `authors = []` dans l'en-tête vide la liste, même si la configuration globale en définit une.

### Configuration globale

`make install` crée `~/.config/combava/config.toml` avec toutes les clés. Les clés qui ont un défaut y sont actives, à leur valeur par défaut ; les autres sont commentées. Retirez le `#` d'une ligne pour l'activer :

```toml
school = "École d'ingénieurs"
university = "Université d'exemple"
```

Ne mettez dans la configuration globale que ce qui vaut pour **tous** vos documents : un `title` actif s'appliquerait à chacun d'eux.

### Configuration d'un projet

Pour partager des réglages entre les documents d'un même dossier (un cours, un stage), créez un dossier `.combava/` à la racine du projet, avec un fichier `config.toml` :

```
mon-cours/
├── .combava/
│   └── config.toml      ← subject = "Génie logiciel", teachers = [...]
├── tp1/
│   └── rapport.md
└── tp2/
    └── rapport.md
```

Combava cherche `.combava/` en remontant depuis le dossier du fichier markdown : le premier trouvé définit la racine du projet.

La clé `output` n'est acceptée que dans l'en-tête d'un document.

### Chemins

Un chemin relatif est résolu par rapport à l'endroit où il est écrit :

| Écrit dans | Relatif à |
|---|---|
| l'en-tête du document | le dossier du fichier markdown |
| `.combava/config.toml` | la racine du projet |
| la configuration globale | le dossier de la configuration globale |
| une option de la ligne de commande | le dossier courant |

`~` en tête de chemin désigne votre dossier personnel : `bibliography = "~/references.bib"`.

## Écrire en markdown

Combava suit le markdown de GitHub : titres, listes, tableaux, images, encadrés, blocs de code, notes de bas de page, formules. Chaque élément, avec un exemple et son rendu, est décrit dans la [référence markdown](markdown.md).

## Bibliographie

1. Rassemblez vos références dans un fichier BibTeX, par exemple `references.bib` :

    ```bibtex
    @book{knuth1984,
      author    = {Knuth, Donald E.},
      title     = {The TeXbook},
      publisher = {Addison-Wesley},
      year      = {1984},
    }
    ```

2. Déclarez-le dans l'en-tête :

    ```toml
    bibliography = "references.bib"
    ```

3. Citez avec `[@cle]`, ou `[@cle1; @cle2]` pour plusieurs références :

    ```markdown
    Le livre de référence [@knuth1984].
    ```

La bibliographie est ajoutée à la fin du document. Le template par défaut utilise le style ISO 690 auteur-date.

Une citation sans bibliographie déclarée est une erreur. Une adresse comme `ada@exemple.fr` n'est pas une citation : seule la forme `[@cle]` en est une.

## Templates

Le template décide de toute la mise en page : couverture, polices, couleurs, en-têtes. Le markdown, lui, ne change pas : pour une autre apparence, on change de template.

### Le template par défaut

Sa couverture affiche, quand ils sont définis :

- le logo de l'université (si `university` est défini) et celui de l'école (si `school` est défini) ;
- `subject` en surtitre, puis le titre et le sous-titre ;
- les auteurs, les enseignants, la formation (`specialization`, `cohort`, `academic_year`), l'établissement (`school`, `university`) et la date.

Viennent ensuite les tables demandées, numérotées en chiffres romains, puis le corps du document, numéroté à partir de 1. L'en-tête de page affiche `header_text` à gauche et le chapitre en cours à droite.

### Mettre vos logos

Les logos fournis sont des images à remplacer. Remplacez les fichiers du template installé, en gardant leur nom :

```
~/.config/combava/templates/default/images/univ_logo.png
~/.config/combava/templates/default/images/school_logo.png
```

Ce template installé remplace celui du binaire. Vous pouvez aussi y retoucher les couleurs et les polices : elles sont toutes dans `theme.typ`.

### Où combava cherche un template

La clé `template` (ou l'option `-t`) accepte un nom ou un chemin.

- **Un chemin** (contient `/` ou `\`, ou commence par `.` ou `~`) désigne directement le dossier du template : `template = "./mon-template"`.
- **Un nom** est cherché dans cet ordre :
    1. `.combava/templates/<nom>/` dans le projet ;
    2. `templates/<nom>/` dans la configuration globale ;
    3. les templates contenus dans le binaire : seulement `default`.

Le premier dossier trouvé l'emporte. Un dossier `default` dans le projet ou dans la configuration globale remplace donc le template par défaut.

### Créer un template

Un template est un dossier qui contient un fichier `template.typ`. Ce fichier définit deux fonctions Typst :

- `template`, qui reçoit tous les champs de l'en-tête et le corps du document, et produit la mise en page ;
- `callout`, qui dessine un encadré `> [!NOTE]`.

Le plus simple est de partir d'une copie du template par défaut :

```bash
cp -r ~/.config/combava/templates/default ~/.config/combava/templates/mon-template
```

puis d'écrire `template = "mon-template"` dans l'en-tête. Le contrat exact (arguments, valeurs possibles, obligations) est dans la [section 9 de la spécification](specification.md#9-contrat-du-template).

Un template peut fournir ses propres polices dans un dossier `fonts/` (fichiers `.ttf`, `.otf`, `.ttc`, `.otc`) ; elles passent avant les polices du système.

## Erreurs et avertissements

### Lire un message

Les messages s'affichent au format des compilateurs, ce qui les rend cliquables dans la plupart des éditeurs :

```
rapport.md:3:1: erreur[unknown-key] : clé inconnue « autors »
rapport.md:12:5: avertissement[raw-html] : HTML brut ignoré
```

Dans l'ordre : fichier, ligne, colonne, gravité, code, puis le message. Une ligne `  aide : …` donne parfois une piste.

- Une **erreur** arrête la compilation : aucun PDF n'est écrit, l'ancien reste intact.
- Un **avertissement** n'empêche pas le PDF.

Code de retour de `combava` : `0` en cas de succès (avertissements compris), `1` en cas d'erreur, `2` pour une commande mal écrite.

### Codes

| Code | Cause | Que faire |
|---|---|---|
| `unknown-key` | clé inconnue dans l'en-tête ou un `config.toml` | corriger l'orthographe (voir [Clés](#clés)) |
| `invalid-type` | valeur du mauvais type | mettre le texte entre guillemets, les listes entre crochets |
| `invalid-toml` | syntaxe TOML invalide | vérifier guillemets et crochets à la position indiquée |
| `unclosed-frontmatter` | `+++` d'ouverture sans `+++` de fermeture | fermer l'en-tête |
| `output-outside-frontmatter` | `output` dans un `config.toml` | déplacer `output` dans l'en-tête du document |
| `invalid-utf8` | fichier qui n'est pas en UTF-8 | réenregistrer le fichier en UTF-8 |
| `template-not-found` | template introuvable ; le message liste les emplacements essayés | vérifier le nom, ou que le dossier contient `template.typ` |
| `bibliography-not-found` | fichier `.bib` absent ou sans l'extension `.bib` | vérifier le chemin (voir [Chemins](#chemins)) |
| `citation-without-bibliography` | `[@cle]` sans clé `bibliography` | déclarer la bibliographie |
| `broken-link` | lien vers une section qui n'existe pas | corriger l'identifiant de la section |
| `duplicate-label` | deux titres avec le même `{#id}` | changer l'un des deux |
| `remote-image` | image en `https://…` | télécharger l'image à côté du document |
| `invalid-image-path` | image en chemin absolu ou hors du dossier du document | déplacer l'image dans le dossier du document |
| `package-download` | téléchargement d'un package Typst impossible | se connecter à internet et relancer |
| `file-exists` | `init` sur un fichier existant | choisir un autre nom, ou `--force` |
| `typst` | erreur ou avertissement de Typst | voir ci-dessous |
| `io` | lecture ou écriture impossible | vérifier le chemin, les droits, que le dossier de sortie existe |
| `raw-html` | HTML ignoré (avertissement) | le retirer, ou passer par un bloc ` ```{=typst} ` |
| `invalid-label` | `{#id}` avec un caractère interdit (avertissement) | n'utiliser que lettres, chiffres, `-`, `_`, `.`, `:` |
| `invalid-code-attributes` | attributs `{…}` d'un bloc de code invalides (avertissement) | seul `{caption="…"}` est reconnu |
| `undefined-footnote` | appel `[^x]` sans définition (avertissement) | ajouter `[^x]: …` |
| `unused-footnote` | définition `[^x]: …` jamais appelée (avertissement) | l'appeler, ou la supprimer |

### Erreurs Typst

Une erreur Typst provoquée par votre document est ramenée à la ligne du fichier markdown, le plus souvent dans un bloc ` ```{=typst} `.

Si le message indique `(ligne N du code généré, voir --transpile-only)`, l'erreur ne correspond à aucune ligne du markdown. Lancez :

```bash
combava build rapport.md --transpile-only
```

et ouvrez le fichier `rapport.typ` produit, à la ligne indiquée.

Une erreur dans un fichier du template est affichée avec le chemin de ce fichier. Pour le template contenu dans le binaire, le chemin s'écrit `<default>/template.typ`.

## Questions fréquentes

**`unresolved import`, avec une aide sur le template.**
Le template utilisé ne respecte pas le contrat : il ne définit pas `template` ou `callout`, ou il est vide. Vérifiez qu'un dossier `templates/default/` incomplet ne traîne pas dans `~/.config/combava/` ou dans le `.combava/` du projet : il remplace le template par défaut.

**Mes changements dans `config.toml` ne sont pas pris en compte.**
Une couche plus prioritaire définit sans doute la même clé : l'en-tête du document l'emporte sur `.combava/config.toml`, qui l'emporte sur la configuration globale.

**`make install` n'a pas mis à jour le template.**
C'est voulu : votre template installé peut contenir vos logos. Utilisez `make install FORCE=1` après avoir mis vos logos de côté.

**Comment changer la taille d'une image ?**
Le markdown ne le permet pas. Passez par un bloc Typst :

````markdown
```{=typst}
#figure(image("/images/schema.png", width: 60%), caption: [Mon schéma])
```
````

Dans un bloc Typst, un chemin qui commence par `/` est relatif au dossier du fichier markdown.
