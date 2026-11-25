# En-tête et configuration de combava

## Sommaire

- [L'en-tête du document](#len-tête-du-document)
- [Clés](#clés)
- [Pièges](#pièges)
- [Configuration partagée : les quatre couches](#configuration-partagée--les-quatre-couches)
- [Chemins](#chemins)
- [Fichiers de configuration types](#fichiers-de-configuration-types)

## L'en-tête du document

L'en-tête (*frontmatter*) est un bloc **TOML** entre deux lignes `+++`, **tout au début** du fichier (un BOM éventuel est toléré avant).

```markdown
+++
title = "Titre du rapport"
subtitle = "Sous-titre"
subject = "Matière"
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

- Syntaxe : `clé = valeur`, une clé par ligne, `#` pour un commentaire.
- Les lignes `+++` peuvent avoir des espaces en fin de ligne ; fins de ligne `\n` ou `\r\n`.
- `+++` d'ouverture sans `+++` de fermeture → erreur `unclosed-frontmatter`.
- L'en-tête est facultatif : sans lui, tout vient de la configuration partagée ou des défauts.
- `---` (YAML) n'est **pas** un en-tête : `---` en première ligne est une ligne horizontale.
- `combava init fichier.md` crée l'en-tête ci-dessus, avec la date du jour en français (`1er` pour le premier du mois).
- Toutes les erreurs de l'en-tête sont rapportées en une seule fois, à leur ligne dans le `.md`.

## Clés

| Clé | Type TOML | Rôle (template par défaut) | Défaut |
|---|---|---|---|
| `title` | chaîne | titre : couverture, métadonnées du PDF | absent |
| `subtitle` | chaîne | sous-titre sur la couverture | absent |
| `authors` | tableau de chaînes | auteurs : couverture, métadonnées du PDF | `[]` |
| `teachers` | tableau de chaînes | enseignants ou encadrants | `[]` |
| `date` | chaîne | date affichée telle quelle | absent |
| `school` | chaîne | école ; affiche aussi le logo de l'école | absent |
| `university` | chaîne | université ; affiche aussi le logo de l'université | absent |
| `academic_year` | chaîne | année universitaire, ex. `"2026-2027"` (le template ajoute « Année universitaire » devant) | absent |
| `cohort` | chaîne | promotion, ex. `"Promotion 2028"` | absent |
| `specialization` | chaîne | spécialité ou filière | absent |
| `subject` | chaîne | matière ou type de document, en surtitre | absent |
| `toc` | booléen | table des matières | `true` |
| `list_of_figures` | booléen | liste des figures (images uniquement) | `false` |
| `list_of_listings` | booléen | liste des codes (blocs avec `caption`) | `false` |
| `header_text` | chaîne | texte en haut de chaque page | le `title` |
| `bibliography` | chaîne (chemin d'un `.bib`) | active la bibliographie | absent |
| `template` | chaîne (nom ou chemin) | template de mise en page (voir `templates.md`) | `"default"` |
| `output` | chaîne (chemin du PDF) | fichier PDF à écrire ; **en-tête uniquement** | `<dossier du .md>/<nom du .md>.pdf` |

- Une clé absente n'est simplement pas affichée.
- Les noms de clés sont en `snake_case` (`academic_year`, `list_of_figures`, `header_text`).

## Pièges

| Erreur fréquente | Correct | Diagnostic |
|---|---|---|
| `autors = ["Ada"]` (clé mal orthographiée, ou clé inventée : `author`, `abstract`, `lang`, `keywords`…) | `authors = ["Ada"]` | `unknown-key` : la compilation **s'arrête** |
| `date = 2026-10-10` (date TOML) | `date = "10 octobre 2026"` | `invalid-type` |
| `authors = "Ada Lovelace"` | `authors = ["Ada Lovelace"]` (crochets même pour un seul) | `invalid-type` |
| `toc = "true"` | `toc = true` | `invalid-type` |
| `title = Mon titre` | `title = "Mon titre"` | `invalid-toml` |
| `title = "*Mon* titre"` | `title = "Mon titre"` | aucun : les astérisques s'affichent (texte brut) |
| `title = "Le "vrai" titre"` | `title = "Le « vrai » titre"` ou `'Le "vrai" titre'` | `invalid-toml` |
| `output = "pdf/rapport.pdf"` dans un `config.toml` | `output` dans l'en-tête du document | `output-outside-frontmatter` |
| dossier de sortie inexistant | créer le dossier avant | `io` |

- **Toute clé hors de la table est une erreur**, il n'y a pas de clé libre.
- Les valeurs sont du **texte brut** : pas de markdown, pas de Typst. Les guillemets droits restent droits.
- Chaîne TOML : `"…"` (échappements `\"`, `\\`, `\n`) ou `'…'` (littérale, sans échappement, pratique pour les chemins Windows). Chaîne multiligne : `"""…"""`.
- Une liste ne se cumule jamais entre couches : `authors = []` dans l'en-tête **vide** la liste définie ailleurs.

## Configuration partagée : les quatre couches

Pour chaque clé, **la première couche qui la définit l'emporte** :

1. options de la ligne de commande `-o` (sortie) et `-t` (template) ;
2. l'en-tête du document ;
3. le fichier `.combava/config.toml` du projet ;
4. la configuration globale ;
5. les défauts de la table des clés.

### Projet

- Racine du projet : le **premier** dossier qui contient un dossier `.combava/`, en remontant depuis le dossier du fichier markdown. Aucun → pas de couche projet.
- On y met ce qui est commun aux documents d'un cours ou d'un stage (`subject`, `teachers`, `bibliography`…).

```
mon-cours/
├── .combava/
│   ├── config.toml      ← subject = "Génie logiciel", teachers = [...]
│   └── templates/       ← templates propres au projet (facultatif)
├── tp1/
│   └── rapport.md
└── tp2/
    └── rapport.md
```

### Globale

| Système | Fichier |
|---|---|
| Linux | `~/.config/combava/config.toml` |
| macOS | `~/Library/Application Support/combava/config.toml` |
| Windows | `%APPDATA%\combava\config\config.toml` |

- Créée par `make install` avec toutes les clés : celles qui ont un défaut sont actives, les autres commentées.
- N'y mettre que ce qui vaut pour **tous** les documents (école, université, promotion, spécialité) : un `title` actif s'appliquerait partout.
- Fichier absent → pas de couche globale.

### Avant d'écrire un en-tête

1. Regarder `.combava/config.toml` en remontant depuis le dossier du document, puis la configuration globale.
2. Ne pas répéter dans l'en-tête ce qu'elles définissent déjà (école, université…).
3. Une valeur de l'en-tête remplace celle des couches inférieures : c'est voulu pour surcharger, piégeux sinon.

## Chemins

Un chemin relatif est résolu par rapport à l'endroit où il est **écrit** :

| Écrit dans | Relatif à |
|---|---|
| l'en-tête du document | le dossier du fichier markdown |
| `.combava/config.toml` | la racine du projet (le dossier qui contient `.combava/`) |
| la configuration globale | le dossier de la configuration globale |
| une option de la ligne de commande | le dossier courant |

- `~` en tête de chemin désigne le dossier personnel, dans toutes les couches : `bibliography = "~/references.bib"`.
- `bibliography` doit désigner un fichier existant, d'extension `.bib` ; sinon `bibliography-not-found`.
- Les **images** et les chemins des blocs Typst ne suivent pas ces règles : ils sont relatifs au fichier markdown, sans en sortir (voir `syntaxe.md` et `typst.md`).

## Fichiers de configuration types

`.combava/config.toml` d'un projet :

```toml
subject = "Génie logiciel"
teachers = ["Grace Hopper"]
bibliography = "references.bib"   # relatif à la racine du projet
list_of_figures = true
```

Configuration globale (extrait) :

```toml
school = "École d'ingénieurs"
university = "Université d'exemple"
academic_year = "2026-2027"
cohort = "Promotion 2028"
specialization = "Informatique"

toc = true
list_of_figures = false
list_of_listings = false
template = "default"
# bibliography = "~/references.bib"
# La clé `output` n'est acceptée que dans l'en-tête d'un document.
```

En-tête complet (tous les champs de la couverture) :

```toml
+++
title = "Rapport d'exemple"
subtitle = "Toutes les fonctionnalités de Combava"
authors = ["Ada Lovelace", "Alan Turing"]
teachers = ["Grace Hopper"]
date = "10 octobre 2026"
school = "École d'ingénieurs"
university = "Université d'exemple"
academic_year = "2026-2027"
cohort = "Promotion 2028"
specialization = "Informatique"
subject = "Génie logiciel"
toc = true
list_of_figures = true
list_of_listings = true
header_text = "Combava : rapport d'exemple"
bibliography = "references.bib"
+++
```
