---
name: markdown-combava
description: Rédaction de documents markdown conformes à la syntaxe de combava (en-tête TOML `+++`, légendes de tableaux, labels `{#id}`, renvois `[@id]`, citations `[@cle]`, encadrés, blocs de code légendés, maths). À utiliser dès qu'il faut écrire, corriger ou relire un fichier markdown destiné à `combava build`, ou diagnostiquer une erreur ou un avertissement de combava.
---

# Écrire du markdown pour combava

Combava transforme **un seul fichier markdown** en PDF (via Typst). Il suit le markdown de GitHub, plus quelques extensions. Ce qu'il ne comprend pas est ignoré ou refusé : mieux vaut s'en tenir à la syntaxe ci-dessous.

Sources de référence, à relire en cas de doute : `docs/markdown.md` (syntaxe), `docs/guide.md` (en-tête, configuration, erreurs), `docs/specification.md` section 4 (règles exactes), `examples/report.md` (document qui utilise tout, sans avertissement).

## Structure d'un document

```markdown
+++
title = "Titre du rapport"
authors = ["Prénom Nom"]
date = "12 octobre 2026"
+++

# Introduction

Texte…
```

1. L'en-tête TOML est entre deux lignes `+++`, **tout au début** du fichier. Sans en-tête utile, `combava init fichier.md` en génère un.
2. Chaque titre `#` ouvre un chapitre sur une nouvelle page (template par défaut) ; les titres sont numérotés et repris dans la table des matières. `##`, `###`… forment les sections.
3. La bibliographie est ajoutée **automatiquement à la fin** : ne pas écrire de titre « Bibliographie ».

## En-tête (TOML)

Une clé par ligne, `clé = valeur`, `#` pour un commentaire.

| Clé | Type | Défaut |
|---|---|---|
| `title`, `subtitle`, `subject`, `date`, `school`, `university`, `academic_year`, `cohort`, `specialization`, `header_text` | texte | absent (`header_text` : le titre) |
| `authors`, `teachers` | liste de textes | `[]` |
| `toc` | booléen | `true` |
| `list_of_figures`, `list_of_listings` | booléen | `false` |
| `bibliography` | chemin d'un `.bib` | absent |
| `template` | nom ou chemin | `"default"` |
| `output` | chemin du PDF (en-tête uniquement) | `<nom>.pdf` |

Pièges :

- **Toute autre clé est une erreur** (`unknown-key`) : `autors` arrête la compilation.
- **La date est un texte entre guillemets** : `date = "12 octobre 2026"`. `date = 2026-10-12` est refusé.
- **Les listes s'écrivent entre crochets**, même avec un seul élément : `authors = ["Ada Lovelace"]`.
- **Les valeurs sont du texte brut** : pas de markdown dans `title = "*Mon* titre"` (les astérisques s'affichent).
- Les valeurs communes à tous les documents (école, université, promotion…) peuvent venir de `.combava/config.toml` (projet) ou de `~/.config/combava/config.toml` (global) : regarder ces fichiers avant de les répéter dans l'en-tête. L'en-tête l'emporte sur le projet, qui l'emporte sur le global. Une liste n'est jamais fusionnée.
- Les chemins relatifs de l'en-tête partent du dossier du fichier markdown.

## Syntaxe markdown

### Texte

| Markdown | Rendu |
|---|---|
| `*emphase*` | emphase |
| `**gras**` | gras |
| `~~barré~~` | barré |
| `` `code` `` | code en ligne |
| deux espaces en fin de ligne | retour à la ligne forcé |
| ligne vide | nouveau paragraphe |

- Un simple retour à la ligne ne coupe pas le paragraphe.
- Les guillemets droits `"…"` deviennent des guillemets français ; ne pas les remplacer à la main.
- Les caractères spéciaux de Typst (`#`, `@`, `$`, `<`, `=`…) s'écrivent tels quels, sans échappement.

### Liens

```markdown
[Typst](https://typst.app), <https://typst.app>, <contact@exemple.fr>
[voir l'introduction](#introduction)
[[Introduction]] ou [[Introduction|le début du rapport]]
```

### Images

```markdown
![Schéma du pipeline](images/pipeline.svg "Le pipeline de compilation")
```

- **Seule dans son paragraphe** : figure numérotée ; le texte entre guillemets est la légende (sans lui, pas de légende).
- **Dans une phrase** : réduite à la hauteur du texte (icône).
- Chemin **relatif au fichier markdown**, sans sortir de son dossier (`../` au-dessus interdit, chemin absolu interdit).
- Les images distantes (`https://…`) sont refusées : les télécharger à côté du document.
- Formats : PNG, JPEG, GIF, SVG. Pas de taille réglable en markdown (voir « Typst direct »).

### Listes

```markdown
- puces
  - imbriquée
- [x] tâche faite
- [ ] tâche à faire

1. numérotée
2. qui continue

Terme
: Sa définition.
```

### Tableaux et légendes

```markdown
Le [@tab-etapes] résume la compilation.

| Étape       | Entrée   | Sortie |
|:------------|:--------:|-------:|
| Parsing     | markdown | arbre  |
| Compilation | Typst    | PDF    |

: Les étapes de la compilation {#tab-etapes}
```

- Alignement : `:--` gauche, `:-:` centre, `--:` droite.
- Légende : paragraphe commençant par `: ` **juste après le tableau, séparé par une ligne vide** (sans ligne vide, elle est lue comme une ligne du tableau). Le tableau devient alors numéroté (« Tableau 1 ») et la légende est placée **sous** lui.
- La légende accepte emphase, code, formule. `{#id}` à la fin lui donne un label.
- Tableau sans légende mais avec label : `: {#tab-etapes}` seul.
- Commencer un paragraphe par `:` sans légende : écrire `\:`.
- Un tableau n'a qu'une légende.

### Labels et renvois

Seuls les **titres** et les **tableaux** ont un label (pas les images ni les blocs de code).

| Élément | Label |
|---|---|
| titre | automatique (comme GitHub : minuscules, sans ponctuation, espaces → `-`), ou `## Résultats {#resultats}` |
| tableau | `{#id}` à la fin de la légende ; pas de label automatique |

| Pour renvoyer vers… | Écrire | Rendu |
|---|---|---|
| un titre ou un tableau | `[texte](#id)` | un lien sur « texte » |
| un titre ou un tableau | `[[cible]]` ou `[[cible\|texte]]` | un lien |
| un tableau | `[@id]` | le numéro : « Tableau 1 » |

- `## Mise en œuvre` a pour label `mise-en-œuvre` ; un second titre identique reçoit `-1`, puis `-2`…
- Un titre qui a un `{#id}` **perd son label automatique** : le viser par son id.
- Un `{#id}` ne contient que lettres, chiffres, `-`, `_`, `.`, `:`. Sinon : avertissement `invalid-label` et l'id est ignoré.
- Même `{#id}` sur deux éléments : erreur `duplicate-label`. Renvoi vers un label inexistant : erreur `broken-link`.
- Dans un tableau markdown, le `|` d'un wikilink s'écrit `\|`.

### Citations en bloc et encadrés

```markdown
> Une citation.

> [!NOTE]
> Une information utile.
```

Cinq encadrés, **en majuscules** : `[!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, `[!CAUTION]`.

### Blocs de code

````markdown
```python
print("Bonjour")
```

```rust {caption="La fonction principale"}
fn main() {}
```
````

- Avec `{caption="…"}` : figure numérotée, reprise dans la liste des codes (`list_of_listings = true`). Sans légende : pas dans la liste.
- `caption` est le **seul** attribut reconnu ; tout autre donne l'avertissement `invalid-code-attributes`.
- Un bloc indenté de quatre espaces est un bloc sans langage.

### Notes de bas de page

```markdown
Une affirmation[^source].

[^source]: La source de cette affirmation.
```

La définition peut être n'importe où. Appeler deux fois la même note réutilise son numéro. Appel sans définition : `undefined-footnote` ; définition jamais appelée : `unused-footnote`.

### Bibliographie

1. Un fichier BibTeX à côté du document (`references.bib`).
2. Dans l'en-tête : `bibliography = "references.bib"` (extension `.bib` obligatoire).
3. Dans le texte : `[@knuth1984]` ou `[@commonmark; @typst2023]`.

- Seule la forme entre crochets est une citation : `ada@exemple.fr` et `@knuth1984` seul restent du texte.
- Une clé absente du `.bib` est une erreur ; une citation sans bibliographie déclarée est une erreur (`citation-without-bibliography`).
- Si la clé est le label d'un tableau, `[@cle]` est un renvoi (« Tableau 1 »), pas une citation.
- La bibliographie ne contient que les références citées.

### Maths

```markdown
En ligne : $E = mc^2$.

$$
\sum_{i=1}^{n} i = \frac{n(n+1)}{2}
$$
```

LaTeX, via le package Typst mitex (téléchargé au premier usage : connexion internet nécessaire une fois). Pour un exposant ou un indice hors formule : `$x^2$`, `$x_i$`.

### Saut de page

Une ligne `---` **précédée d'une ligne vide** produit un saut de page. Sans ligne vide avant, `---` transforme le paragraphe précédent en titre. Dans une citation, une liste ou un encadré, c'est un simple filet.

### Typst direct (dernier recours)

````markdown
```{=typst}
#align(center)[Ce paragraphe est centré.]
```
````

Recopié tel quel dans le code Typst. À réserver à ce que le markdown ne sait pas faire, par exemple une image à taille fixe :

````markdown
```{=typst}
#figure(image("/images/schema.png", width: 60%), caption: [Mon schéma])
```
````

Dans un bloc Typst, un chemin commençant par `/` est relatif au dossier du fichier markdown.

## Ce qui n'existe pas (ne pas l'utiliser)

- **HTML** (`<div>`, `<br>`, `<sup>`…) : ignoré avec l'avertissement `raw-html`. Les commentaires `<!-- … -->` sont ignorés sans avertissement : bons pour des notes personnelles.
- **Exposant / indice** : passer par les maths.
- **Plusieurs fichiers** : pas d'inclusion, un document = un fichier.
- **Annexes** numérotées A, B, C… : aucune syntaxe. Un titre `#` après la bibliographie est impossible, elle est toujours en dernier.
- **Label sur une image ou un bloc de code** : seuls titres et tableaux en ont un.
- **Markdown dans l'en-tête** : les valeurs TOML sont du texte brut.

## Méthode de travail

1. Lire le fichier existant, son en-tête et les fichiers `.combava/config.toml` voisins avant d'écrire.
2. Écrire en respectant la syntaxe ci-dessus ; en cas de doute sur un élément, ouvrir `examples/report.md`.
3. **Vérifier en compilant** : un document doit se compiler sans erreur et, idéalement, sans avertissement.

   ```bash
   combava build rapport.md
   ```

   - Avec `--transpile-only`, combava écrit le code Typst généré (`rapport.typ`) au lieu du PDF : utile pour comprendre une erreur de ligne « du code généré ».
   - Sans binaire installé : `cargo run -p combava-cli -- build rapport.md` depuis le dépôt combava.
   - Les diagnostics ont la forme `rapport.md:12:5: avertissement[raw-html] : HTML brut ignoré` (fichier, ligne, colonne, gravité, code, message). Toutes les erreurs sont rapportées en une fois. Une erreur ne produit aucun PDF et laisse l'ancien intact ; un avertissement n'empêche pas le PDF.
4. Corriger selon le code du diagnostic (liste complète dans `docs/guide.md`, section « Codes »).

| Code | Correction |
|---|---|
| `unknown-key` | orthographe de la clé d'en-tête |
| `invalid-type` | texte entre guillemets, listes entre crochets |
| `invalid-toml` / `unclosed-frontmatter` | guillemets, crochets, `+++` de fermeture |
| `broken-link` | corriger l'identifiant (titre ou tableau existant) |
| `duplicate-label` | changer l'un des deux `{#id}` |
| `remote-image` / `invalid-image-path` | image locale, dans le dossier du document |
| `bibliography-not-found` | chemin et extension `.bib` |
| `citation-without-bibliography` | déclarer `bibliography` dans l'en-tête |
| `raw-html` | retirer le HTML |
| `invalid-label` | lettres, chiffres, `-`, `_`, `.`, `:` seulement |
| `invalid-code-attributes` | seul `{caption="…"}` est permis |
| `undefined-footnote` / `unused-footnote` | ajouter la définition, ou retirer la note |

## Rédaction

- Écrire en français, avec les accents, ponctuation et guillemets français « » pour les citations dans le texte (les guillemets droits `"…"` sont convertis automatiquement).
- Pour un rapport d'ingénieur (structure résumé / introduction / conclusion, sourcing), compléter avec le skill `redaction` s'il est disponible.
- Ne pas livrer un document qui ne compile pas : si `combava` n'est pas utilisable, le dire explicitement.
