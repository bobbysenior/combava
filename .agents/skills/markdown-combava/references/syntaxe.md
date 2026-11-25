# Syntaxe markdown de combava (référence complète)

Tout le markdown que combava comprend, avec la règle exacte appliquée à chaque élément. Base : le markdown de GitHub (CommonMark + GFM), plus des extensions (légendes et labels de tableaux, citations `[@cle]`, labels `{#id}`, wikilinks, maths, blocs Typst).

Rendus décrits pour le template par défaut (voir `templates.md`).

## Sommaire

- [Texte](#texte)
- [Titres](#titres)
- [Liens](#liens)
- [Images](#images)
- [Listes](#listes)
- [Tableaux](#tableaux)
- [Labels et renvois](#labels-et-renvois)
- [Citations en bloc et encadrés](#citations-en-bloc-et-encadrés)
- [Blocs de code](#blocs-de-code)
- [Notes de bas de page](#notes-de-bas-de-page)
- [Citations bibliographiques](#citations-bibliographiques)
- [Mathématiques](#mathématiques)
- [Saut de page](#saut-de-page)
- [Typst direct](#typst-direct)
- [HTML](#html)
- [Ce qui n'est pas pris en charge](#ce-qui-nest-pas-pris-en-charge)

## Texte

| Markdown | Rendu |
|---|---|
| `*emphase*` ou `_emphase_` | italique |
| `**gras**` | gras |
| `~~barré~~` | barré |
| `` `code` `` | code en ligne (police à chasse fixe, fond gris) |
| deux espaces en fin de ligne (ou `\` en fin de ligne) | retour à la ligne forcé |
| une ligne vide | nouveau paragraphe |

```markdown
Du texte en *emphase*, en **gras**, ~~barré~~ et avec du `code`.
Cette ligne continue le même paragraphe.

Un nouveau paragraphe, avec "des guillemets droits".
```

- Un retour à la ligne simple devient une espace : il ne coupe pas le paragraphe.
- Les guillemets droits `"…"` deviennent des guillemets français, l'apostrophe `'` devient typographique. Ne pas les remplacer à la main.
- **Tout le texte est échappé** : les caractères spéciaux de Typst (`\ # $ * _ @ < > [ ] ` ~ / = - +`) s'affichent tels quels. Le texte ne produit jamais de code Typst par accident. Un `.` après un chiffre (`2026.`) est aussi protégé : il ne devient pas une liste.
- Les échappements markdown habituels restent valables : `\*` affiche un astérisque, `\\` une barre oblique inverse.
- Pas d'exposant ni d'indice hors maths : écrire `$x^2$`, `$H_2 O$`.
- Pas d'espace insécable automatique avant `: ; ! ?` : la typographie française fine n'est pas gérée.

## Titres

```markdown
# Chapitre
## Section
### Sous-section
#### Paragraphe titré
```

- `#` à `######` : niveaux 1 à 6. Les titres « soulignés » (`===`, `---` sous une ligne) sont aussi des titres de niveau 1 et 2.
- Template par défaut :
    - chaque `#` ouvre un **chapitre sur une nouvelle page**, avec l'étiquette « Chapitre N » ;
    - numérotation `1`, `1.1`, `1.1.1` ; le niveau 4 s'affiche sans numéro ;
    - la table des matières reprend les niveaux 1 à 3.
- Chaque titre reçoit un label (voir [Labels et renvois](#labels-et-renvois)) : automatique, ou explicite avec `{#id}` à la fin : `## Résultats {#resultats}`.
- Les classes et attributs `{.classe cle=valeur}` sont **ignorés** : `{.unnumbered}` ou `{-}` ne retirent pas le numéro. Pour un titre non numéroté, voir `typst.md` (recette « Chapitre non numéroté »).
- Une note de bas de page dans un titre est permise (elle n'a pas de label, pour ne pas être dupliquée dans la table des matières).
- Ne pas écrire de titre « Bibliographie » : elle est ajoutée automatiquement à la fin.

## Liens

```markdown
Un lien externe : [Typst](https://typst.app), <https://typst.app> ou <contact@exemple.fr>.
Un lien vers une section : [voir l'introduction](#introduction).
Un wikilink : [[Introduction]] ou [[Introduction|le début du rapport]].
```

| Forme | Cible |
|---|---|
| `[texte](https://…)`, `[texte](mailto:…)`, `<https://…>`, `<a@b.fr>` | lien externe |
| `[texte](#id)` | titre ou tableau du document, par son label ; `%20` est décodé en espace |
| `[[cible]]`, `[[cible\|texte]]` | titre ou tableau ; affiche `texte`, sinon `cible` |

- Lien interne ou wikilink vers un label qui n'existe pas → **erreur `broken-link`** : un PDF ne sort jamais avec un lien cassé.
- Le titre d'un lien (`[a](url "titre")`) est ignoré.
- Toute autre destination (`[annexe](annexe.pdf)`) est recopiée telle quelle comme lien externe, sans vérification : préférer une URL complète.

## Images

```markdown
![Schéma du pipeline](images/pipeline.svg "Le pipeline de compilation")
```

| Position | Résultat |
|---|---|
| **seule dans son paragraphe** (aux espaces près) | figure numérotée, centrée, légende « Fig. N — … » dessous ; le texte entre guillemets est la **légende** ; sans lui, pas de légende |
| **dans une phrase** | image réduite à la hauteur du texte (`1em`), comme une icône |

- Le texte entre crochets est le texte alternatif (`alt`), pas la légende.
- Chemin **relatif au fichier markdown**, normalisé (`./`, `a/../`), `%20` décodé en espace.
    - chemin absolu, ou qui remonte au-dessus du dossier du `.md` → erreur `invalid-image-path` ;
    - `http://…` ou `https://…` → erreur `remote-image` : télécharger l'image à côté du document.
- Formats sûrs : PNG, JPEG, GIF, SVG.
- Les figures-images sont reprises dans la liste des figures si `list_of_figures = true`.
- **Pas de taille réglable en markdown**, pas de label sur une image : passer par un bloc Typst (`typst.md`, recettes « Image à taille fixe » et « Figure avec renvoi »).

## Listes

```markdown
- une liste à puces
  - imbriquée
- [x] une tâche faite
- [ ] une tâche à faire

1. une liste numérotée
2. qui continue

7) une liste qui commence à 7

Terme
: Sa définition.
: Une seconde définition.
```

- Puces : `-`, `*` ou `+`. Numérotées : `1.` ou `1)` ; le **premier numéro est conservé** (`7)` commence à 7), les suivants sont recalculés.
- Liste « serrée » si aucun élément n'est séparé par une ligne vide ; sinon, espacement entre éléments.
- Cases à cocher : `[ ]` → ☐, `[x]` → ☒ en début d'élément.
- Liste de définitions : `Terme` sur une ligne, puis `: définition` (plusieurs définitions possibles, séparées par un paragraphe).
- Un élément peut contenir plusieurs paragraphes, du code, une citation : indenter le contenu sous le texte de l'élément.

## Tableaux

```markdown
| Étape       | Entrée   | Sortie |
|:------------|:--------:|-------:|
| Parsing     | markdown | arbre  |
| Compilation | Typst    | PDF    |

: Les étapes de la compilation {#tab-etapes}
```

- Alignement : `:--` gauche, `:-:` centre, `--:` droite, rien → gauche.
- La première ligne est l'en-tête (gras, fond bleu pâle) ; style « livre », sans quadrillage.
- Un tableau trop haut est **coupé entre deux pages**, l'en-tête répété.
- Les cellules acceptent le markdown en ligne (emphase, code, lien, formule), pas de bloc (liste, paragraphe multiple). Un `|` dans une cellule s'écrit `\|`.
- Pas de cellule fusionnée, pas de largeur de colonne : passer par un bloc Typst (`#table(...)`) si nécessaire.

### Légende

- Un paragraphe qui commence par `:` suivi d'une espace ou d'une tabulation, **juste après le tableau**, est sa légende (syntaxe de Pandoc).
- **Une ligne vide est obligatoire** entre le tableau et la légende : sans elle, GFM lit la ligne comme une ligne du tableau.
- Effet : le tableau devient une figure numérotée « Tableau N », légende **sous** le tableau. Le paragraphe de légende disparaît du flux.
- La légende est du markdown en ligne : emphase, code, formule, note de bas de page (sans label).
- Un tableau n'a qu'une légende. Pour commencer un paragraphe normal par `:`, écrire `\:`.
- Les tableaux ne sont repris dans **aucune** liste (la liste des figures ne contient que les images).

### Label

- `{#id}` à la fin de la légende : `: Légende {#tab-x}`. Il est retiré du texte affiché.
- Tableau sans légende mais avec label : `: {#tab-x}` seul. Il devient numéroté, sans légende.
- Pas de label automatique pour un tableau.

## Labels et renvois

Deux éléments seulement ont un label : les **titres** et les **tableaux**. Images, blocs de code, formules et notes n'en ont pas.

| Élément | Label |
|---|---|
| titre | automatique, ou `{#id}` à la fin du titre (`## Résultats détaillés {#resultats}`) |
| tableau | `{#id}` à la fin de la légende (`: Les étapes {#tab-etapes}`) ; jamais automatique |

### Label automatique d'un titre (slug, comme GitHub)

1. texte du titre sans mise en forme ;
2. en minuscules ;
3. suppression de tout caractère autre que lettre, chiffre, espace, `-` ou `_` (la ponctuation et les apostrophes disparaissent, **les accents restent**) ;
4. chaque espace devient `-` ;
5. slug vide → `section` ;
6. déjà pris → `-1`, `-2`… dans l'ordre d'apparition.

Exemples : `## Mise en œuvre` → `mise-en-œuvre` ; `## L'en-tête du document` → `len-tête-du-document` ; un second `## Résultats` → `résultats-1`.

Un titre qui a un `{#id}` valide **n'a plus de label automatique** : le viser par son id.

### Renvoyer

| Markdown | Rendu | Cible |
|---|---|---|
| `[les résultats](#resultats)` | lien sur « les résultats » | titre ou tableau |
| `[[Mise en œuvre]]` | lien sur « Mise en œuvre » | titre ou tableau |
| `[[resultats\|les résultats]]` | lien sur « les résultats » | titre ou tableau |
| `[@tab-etapes]` | « Tableau 1 » (numéro à jour, cliquable) | **tableau uniquement** |

- Wikilink `[[cible]]` : vise `cible` si c'est un `{#id}` explicite, sinon le slug calculé à partir de `cible`. `[[Mise en œuvre]]` mène donc au titre `## Mise en œuvre`.
- Dans un tableau markdown, le `|` d'un wikilink s'écrit `\|`.
- Pas de « Section 2.3 » automatique : `[@id]` sur un titre est lu comme une citation bibliographique (erreur). Pour un titre, utiliser un lien.

### Règles

- **Caractères d'un `{#id}`** : lettres, chiffres, `-`, `_`, `.`, `:`. Sinon avertissement `invalid-label`, l'id est ignoré (le titre garde son slug, le tableau reste sans label).
- **Unicité** : le même `{#id}` sur deux titres ou tableaux → erreur `duplicate-label`. Les `{#id}` explicites sont réservés avant le calcul des slugs.
- **Lien cassé** → erreur `broken-link`.
- Les labels posés dans un bloc Typst (`<fig-x>`) sont **invisibles pour combava** : `[texte](#fig-x)` → `broken-link`, `[@fig-x]` → citation. Les viser depuis un bloc Typst (`@fig-x`), voir `typst.md`.
- Le préfixe `combava-fn-` est réservé (labels des notes).

## Citations en bloc et encadrés

```markdown
> Une citation.
>
> Sur plusieurs paragraphes.

> [!NOTE]
> Une information utile.
```

- Citation : bloc en italique avec un filet coloré à gauche.
- Encadrés (callouts GFM), **mot-clé en majuscules**, seul sur la première ligne :

| Markdown | Titre affiché | Couleur |
|---|---|---|
| `> [!NOTE]` | Note | bleu |
| `> [!TIP]` | Astuce | vert |
| `> [!IMPORTANT]` | Important | violet |
| `> [!WARNING]` | Attention | orange |
| `> [!CAUTION]` | Prudence | rouge |

- Le titre de l'encadré est fixé par le template : pas de titre personnalisé (`> [!NOTE] Mon titre` n'est pas prévu).
- Un encadré ne se coupe pas entre deux pages : le garder court.
- Il peut contenir listes, code, images ; un `---` y trace un filet au lieu d'un saut de page.

## Blocs de code

````markdown
```python
print("Bonjour")
```

```rust {caption="La fonction principale"}
fn main() {}
```

    Un bloc indenté de quatre espaces, sans langage.
````

- Info string : `<langage>` puis éventuellement `{caption="…"}`. Coloration syntaxique selon le langage (noms de Typst/Sublime : `python`, `rust`, `bash`, `js`, `json`, `toml`, `yaml`, `sql`, `c`, `java`…).
- **Avec `{caption="…"}`** : figure numérotée « Liste N — légende » sous le code, reprise dans la liste des codes (`list_of_listings = true`). Guillemet dans la légende : `\"`.
- **Sans légende** : simple bloc, absent de la liste des codes.
- `caption` est le **seul** attribut reconnu. Tout autre (`{linenos}`, `{.class}`, `{title="…"}`) ou une syntaxe invalide → avertissement `invalid-code-attributes`, attributs ignorés.
- Pas de label sur un bloc de code, pas de numéros de ligne.
- Le contenu est recopié tel quel (aucun échappement visible).
- ` ```{=typst} ` n'est pas un bloc de code : voir [Typst direct](#typst-direct).
- Pour montrer un bloc de code markdown qui contient lui-même des ` ``` `, l'entourer de quatre accents graves.

## Notes de bas de page

```markdown
Une affirmation[^source], puis de nouveau la même[^source].

[^source]: La source de cette affirmation.

[^long]: Premier paragraphe de la note.

    Second paragraphe, indenté de quatre espaces.
```

- Identifiant libre (`[^1]`, `[^source]`).
- La définition peut se trouver avant ou après l'appel, n'importe où.
- Appeler deux fois la même note réutilise son numéro.
- Appel sans définition → avertissement `undefined-footnote`, l'appel s'affiche comme du texte.
- Définition jamais appelée → avertissement `unused-footnote`.

## Citations bibliographiques

Prérequis : `bibliography = "references.bib"` dans l'en-tête (voir `configuration.md`).

```markdown
Le livre de référence [@knuth1984].
Deux sources à la fois [@commonmark; @typst2023].
```

- `[@cle]` cite une référence, `[@cle1; @cle2]` en cite plusieurs (séparées par `;`).
- Clé : lettres, chiffres, `_`, `-`, `.`, `:` (la clé BibTeX exacte, sensible à la casse).
- Rendu (template par défaut) : style ISO 690 auteur-date, « (Knuth 1984) ».
- La bibliographie est ajoutée **automatiquement à la fin** du document, après le dernier chapitre. Elle ne contient **que les références citées**.
- Seule la forme entre crochets est une citation : `ada@exemple.fr`, `@knuth1984` seul ou `[voir @knuth1984]` restent du texte.
- Pas de numéro de page ni de préfixe : `[@knuth1984, p. 12]` n'est **pas** une citation, il s'affiche tel quel entre crochets. Écrire `[@knuth1984], p. 12` ou mettre la page dans le texte.
- Bibliographie déclarée mais aucune citation : une page « Bibliographie » vide est ajoutée. Ne déclarer `bibliography` que si le document cite.
- La bibliographie apparaît dans la table des matières, sans numéro.
- Si `cle` est le label d'un tableau du document, `[@cle]` est un **renvoi** (« Tableau 1 »), pas une citation.
- Erreurs :
    - citation alors qu'aucune bibliographie n'est déclarée → `citation-without-bibliography` ;
    - clé absente du `.bib` → erreur `typst` « citation key `x` is not present in the bibliography », à la ligne de la citation.

Fichier BibTeX type :

```bibtex
@book{knuth1984,
  author    = {Knuth, Donald E.},
  title     = {The {\TeX}book},
  publisher = {Addison-Wesley},
  year      = {1984},
}

@misc{typst2023,
  author = {Mädje, Laurenz and Haug, Martin},
  title  = {Typst: A Markup-Based Typesetting System},
  year   = {2023},
  url    = {https://typst.app},
}
```

## Mathématiques

Les formules s'écrivent en **LaTeX**, converties par le package Typst mitex (`@preview/mitex:0.2.7`).

```markdown
En ligne : $E = mc^2$.

Centrée :

$$
\sum_{i=1}^{n} i = \frac{n(n+1)}{2}
$$
```

- `$…$` : formule en ligne ; `$$…$$` : formule centrée (en bloc).
- Syntaxe LaTeX courante : `\frac`, `\sqrt`, `\sum`, `\int`, `\alpha`, `\mathbb{R}`, `\begin{pmatrix}…\end{pmatrix}`, `\begin{aligned}…\end{aligned}`, `\text{…}`.
- Pas de numéro d'équation, pas de label sur une formule.
- Le **premier** document qui contient des maths télécharge mitex : connexion internet nécessaire cette fois-là (sinon erreur `package-download`), ensuite le package est en cache.
- Un `$` isolé dans le texte (prix) : l'échapper, `\$`.

## Saut de page

```markdown
Fin de la première page.

---

Début de la deuxième page.
```

- Une ligne horizontale (`---`, `***`, `___`, `- - -`), **précédée d'une ligne vide**, produit un saut de page.
- Sans ligne vide avant, `---` transforme le paragraphe qui précède en titre de niveau 2 (règle du markdown).
- Dans une citation, une liste, une note ou un encadré : simple filet horizontal (Typst y interdit les sauts de page).
- Un saut de page avant un `#` est inutile : chaque chapitre commence déjà sur une nouvelle page.

## Typst direct

````markdown
```{=typst}
#align(center)[Ce paragraphe est centré.]
```
````

- Le contenu est **recopié tel quel** dans le code Typst, sans échappement. C'est la seule façon d'écrire du Typst.
- À réserver à ce que le markdown ne sait pas faire. Recettes, fonctions disponibles et règles de chemin : `typst.md`.
- Une erreur Typst dans ce bloc est signalée à la bonne ligne du `.md`.

## HTML

- Le HTML brut (`<div>`, `<br>`, `<sup>`, `<img>`, `<details>`…), en bloc ou en ligne, est **ignoré** avec l'avertissement `raw-html`.
- Les commentaires `<!-- … -->` sont ignorés **sans** avertissement : bons pour des notes personnelles ou du texte à masquer.

## Ce qui n'est pas pris en charge

| Besoin | Que faire |
|---|---|
| exposant, indice | maths : `$x^2$`, `$x_i$` (`~x~` et `^x^` n'existent pas) |
| taille d'une image | bloc Typst (`typst.md`) |
| label sur une image, un code ou une formule | bloc Typst pour l'image ; sinon, pas de renvoi |
| renvoi numéroté vers un titre (« Section 2.3 ») | lien `[texte](#id)` |
| plusieurs fichiers markdown | impossible : un document = un fichier |
| annexes numérotées A, B, C | bloc Typst (`typst.md`, recette « Annexes ») ; la bibliographie reste à la toute fin |
| titre non numéroté | bloc Typst (`typst.md`) |
| HTML | retirer, ou bloc Typst |
| images distantes | télécharger à côté du document |
| markdown dans l'en-tête | impossible : valeurs en texte brut |
| emoji `:smile:` | non convertis ; le caractère Unicode s'affiche si la police l'a |
| `combava watch`, PDF/A | non disponibles |
