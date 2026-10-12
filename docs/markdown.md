# Référence markdown

Cette page décrit tout le markdown que combava comprend, avec un exemple pour chaque élément. Sur le site de documentation, chaque exemple est suivi d'un aperçu de son rendu avec le template par défaut. Pour installer combava, régler l'en-tête ou lire un message d'erreur, voir le [guide d'utilisation](guide.md). Pour la correspondance exacte avec le code Typst, voir la [spécification](specification.md#4-correspondance-markdown--typst).

Combava suit le markdown de GitHub. Les caractères spéciaux de Typst (`#`, `@`, `<`, `=`…) sont affichés tels quels : le texte ne produit jamais de code Typst par accident.

## Texte

| Markdown | Rendu |
|---|---|
| `*emphase*` ou `_emphase_` | *emphase* |
| `**gras**` | **gras** |
| `~~barré~~` | ~~barré~~ |
| `` `code` `` | `code` en ligne |
| deux espaces en fin de ligne | retour à la ligne forcé |
| une ligne vide | nouveau paragraphe |

Un retour à la ligne simple ne coupe pas le paragraphe. Les guillemets droits `"…"` deviennent des guillemets français.

```markdown {preview=texte}
Du texte en *emphase*, en **gras**, ~~barré~~ et avec du `code`.
Cette ligne continue le même paragraphe.

Un nouveau paragraphe, avec "des guillemets droits".
```

## Titres

```markdown {preview=titres}
# Chapitre
## Section
## Sous-section
```

Avec le template par défaut, chaque titre `#` ouvre un chapitre sur une nouvelle page. Les titres sont numérotés et repris dans la table des matières.

## Liens

```markdown {preview=liens}
Un lien externe : [Typst](https://typst.app), <https://typst.app> ou <contact@exemple.fr>.
Un lien vers une section : [voir l'introduction](#introduction).
Un wikilink : [[Introduction]] ou [[Introduction|le début du rapport]].
```

Un lien `#…` vise une section ou un tableau par son label : voir [Labels et renvois](#labels-et-renvois).

## Images

```markdown {preview=images}
![Schéma du pipeline](images/pipeline.svg "Le pipeline de compilation")
```

- **Seule dans son paragraphe**, l'image devient une figure numérotée. Le texte entre guillemets est la légende ; sans lui, la figure n'a pas de légende.
- **Dans une phrase**, l'image est réduite à la hauteur du texte, comme une icône.
- Le chemin est relatif au fichier markdown et ne peut pas en sortir (`../` interdit au-dessus de son dossier).
- Les images distantes (`https://…`) sont refusées : téléchargez-les à côté du document.
- Formats courants : PNG, JPEG, GIF, SVG.

## Listes

```markdown {preview=listes}
- une liste à puces
  - imbriquée
- [x] une tâche faite
- [ ] une tâche à faire

1. une liste numérotée
2. qui continue

Terme
: Sa définition.
```

## Tableaux

```markdown {preview=tableaux}
| Étape       | Entrée   | Sortie |
|:------------|:--------:|-------:|
| Parsing     | markdown | arbre  |
| Compilation | Typst    | PDF    |

: Les étapes de la compilation
```

- `:--` aligne à gauche, `:-:` centre, `--:` aligne à droite.
- Un paragraphe qui commence par `: ` juste après le tableau devient sa **légende** : le tableau est alors numéroté (« Tableau 1 ») et sa légende placée en dessous, comme celle des images et des blocs de code. Laissez une ligne vide entre les deux : sans elle, la ligne ferait partie du tableau.
- La légende peut contenir de la mise en forme, du code ou une formule.
- Pour commencer un paragraphe par `:` sans en faire une légende, écrivez `\:`.

### Faire référence à un tableau

Un `{#id}` à la fin de la légende donne un label au tableau. `[@id]` affiche alors son numéro, et un lien `[texte](#id)` y mène :

```markdown
Le [@tab-etapes] résume la compilation.

| Étape       | Sortie |
|-------------|--------|
| Compilation | PDF    |

: Les étapes de la compilation {#tab-etapes}
```

Le texte devient « Le Tableau 1 résume la compilation », et le numéro suit si des tableaux sont ajoutés avant.

- Un tableau sans légende peut avoir un label : écrivez seulement `: {#tab-etapes}` sous le tableau. Il est alors numéroté.
- Les règles communes à tous les labels (caractères autorisés, doublons) sont dans [Labels et renvois](#labels-et-renvois).

## Citations en bloc et encadrés

Pour citer un ouvrage de la bibliographie, voir [Citations bibliographiques](#citations-bibliographiques).

```markdown {preview=citations}
> Une citation.

> [!NOTE]
> Une information utile.
```

Les cinq encadrés de GitHub sont reconnus :

| Markdown | Encadré (template par défaut) |
|---|---|
| `> [!NOTE]` | Note |
| `> [!TIP]` | Astuce |
| `> [!IMPORTANT]` | Important |
| `> [!WARNING]` | Attention |
| `> [!CAUTION]` | Prudence |

```markdown {preview=encadres}
> [!NOTE]
> Une information utile.

> [!TIP]
> Un conseil pour aller plus vite.

> [!IMPORTANT]
> Une règle à ne pas oublier.

> [!WARNING]
> Un risque de mauvaise surprise.

> [!CAUTION]
> Une action qu'on ne peut pas annuler.
```

## Blocs de code

````markdown {preview=code}
```python
print("Bonjour")
```

```rust {caption="La fonction principale"}
fn main() {}
```
````

Un bloc avec `{caption="…"}` devient une figure numérotée et apparaît dans la liste des codes (`list_of_listings = true`). Les blocs sans légende n'y figurent pas.

## Notes de bas de page

```markdown {preview=notes}
Une affirmation[^source].

[^source]: La source de cette affirmation.
```

La définition peut se trouver n'importe où dans le document. Appeler deux fois la même note réutilise son numéro.

## Citations bibliographiques

Une fois la bibliographie déclarée dans l'en-tête (`bibliography = "references.bib"`, voir le [guide](guide.md#bibliographie)), on cite une référence par sa clé BibTeX :

```markdown {preview=bibliographie}
Le livre de référence [@knuth1984].
Deux sources à la fois [@commonmark; @typst2023].
```

- `[@cle]` cite une référence, `[@cle1; @cle2]` en cite plusieurs.
- La bibliographie est ajoutée à la fin du document. Elle ne contient que les références citées.
- Le template par défaut utilise le style ISO 690 auteur-date : « (Knuth 1984) ».
- Une clé absente du fichier `.bib` est une erreur, signalée à la ligne de la citation.
- Une citation sans bibliographie déclarée est aussi une erreur.
- Seule la forme entre crochets est une citation : `ada@exemple.fr` ou `@knuth1984` seul restent du texte.
- Si la clé est le label d'un tableau du document, `[@cle]` n'est pas une citation mais un renvoi : voir [Labels et renvois](#labels-et-renvois).

## Labels et renvois

Un label identifie un élément du document, pour y renvoyer depuis le texte. Deux éléments en ont un :

| Élément | Label | Exemple |
|---|---|---|
| titre | automatique, ou `{#id}` à la fin du titre | `## Résultats détaillés {#resultats}` |
| tableau | `{#id}` à la fin de la légende | `: Les étapes de la compilation {#tab-etapes}` |

Le label automatique d'un titre est calculé comme sur GitHub : en minuscules, sans ponctuation, espaces remplacées par `-`. `## Mise en œuvre` reçoit `mise-en-œuvre`. Si deux titres ont le même texte, le second reçoit `-1`, le troisième `-2`, etc. Un tableau n'a pas de label automatique.

Trois façons d'y renvoyer :

| Markdown | Rendu | Cible |
|---|---|---|
| `[les résultats](#resultats)` | un lien sur « les résultats » | titre ou tableau |
| `[[Mise en œuvre]]`, `[[resultats\|les résultats]]` | un lien sur le texte du titre, ou sur le texte donné | titre ou tableau |
| `[@tab-etapes]` | le numéro : « Tableau 1 » | tableau |

```markdown
## Résultats détaillés {#resultats}

Le [@tab-etapes] reprend [les résultats](#resultats).

| Étape       | Sortie |
|-------------|--------|
| Compilation | PDF    |

: Les étapes de la compilation {#tab-etapes}
```

Dans un wikilink, `[[cible]]` vise le label `cible` s'il a été posé avec `{#cible}`, sinon le label automatique calculé à partir de `cible` : `[[Mise en œuvre]]` mène au titre `## Mise en œuvre`. Un titre qui a un `{#id}` n'a plus de label automatique : visez-le par son id, `[[resultats]]`.

- **Caractères** : un `{#id}` ne contient que des lettres, des chiffres, `-`, `_`, `.` et `:`. Sinon, il est ignoré avec un avertissement : le titre garde son label automatique, le tableau reste sans label.
- **Unicité** : un même `{#id}` sur deux titres ou tableaux est une erreur.
- **Lien cassé** : un lien vers un label qui n'existe pas est une erreur. Vous ne livrerez pas un PDF avec un lien cassé.

## Mathématiques

Les formules s'écrivent en LaTeX :

```markdown {preview=maths}
En ligne : $E = mc^2$.

Centrée :

$$
\sum_{i=1}^{n} i = \frac{n(n+1)}{2}
$$
```

> [!NOTE]
> Le premier document qui contient des maths télécharge un package Typst (mitex). Il faut une connexion internet cette fois-là ; le package est ensuite gardé en cache.

## Saut de page

Une ligne horizontale (`---`, `***` ou `___`), précédée d'une ligne vide, produit un saut de page. Sans ligne vide avant, `---` transforme le paragraphe qui précède en titre : c'est la règle du markdown. Dans une citation, une liste ou un encadré, la ligne horizontale trace un simple filet.

```markdown {preview=saut-de-page}
Fin de la première page.

---

Début de la deuxième page.
```

## Écrire directement en Typst

Si le markdown ne suffit pas, un bloc ` ```{=typst} ` est recopié tel quel dans le code Typst :

````markdown {preview=typst}
```{=typst}
#align(center)[Ce paragraphe est centré.]
```
````

C'est la seule façon d'écrire du Typst : aucune autre syntaxe du document n'en produit.

## Ce qui n'est pas pris en charge

- **Le HTML** est ignoré, avec un avertissement. Les commentaires `<!-- … -->` sont ignorés sans avertissement : utilisez-les pour vos notes personnelles.
- **Exposant et indice** : utilisez les maths, `$x^2$` ou `$x_i$`.
- **Un document en plusieurs fichiers** : un document est un seul fichier markdown.
- **Les annexes** : il n'y a pas de syntaxe pour ouvrir une partie « Annexes » numérotée A, B, C… Un titre `#` après la bibliographie n'est pas possible non plus : la bibliographie est toujours ajoutée à la fin.
- **Un label sur une image ou un bloc de code** : seuls les titres et les tableaux en ont un.
