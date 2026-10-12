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

Chaque titre reçoit un identifiant calculé comme sur GitHub : en minuscules, sans ponctuation, espaces remplacées par `-`. `## Mise en œuvre` devient `#mise-en-œuvre`. Pour choisir l'identifiant vous-même :

```markdown
## Résultats détaillés {#resultats}

Voir [les résultats](#resultats).
```

Un lien vers une section qui n'existe pas est une erreur : vous ne livrerez pas un PDF avec un lien cassé.

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
- Un paragraphe qui commence par `: ` juste après le tableau devient sa **légende** : le tableau est alors numéroté (« Tableau 1 ») et sa légende placée au-dessus. Laissez une ligne vide entre les deux : sans elle, la ligne ferait partie du tableau.
- La légende peut contenir de la mise en forme, du code ou une formule.
- Pour commencer un paragraphe par `:` sans en faire une légende, écrivez `\:`.


## Citations et encadrés

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
