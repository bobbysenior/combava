# Écrire du Typst dans un document combava

Le markdown ne sait pas tout faire. Un bloc ` ```{=typst} ` est recopié **tel quel** dans le code Typst généré. C'est un **dernier recours** : un document qui en abuse n'est plus lisible en markdown.

## Sommaire

- [Règles des blocs Typst](#règles-des-blocs-typst)
- [Recettes vérifiées](#recettes-vérifiées)
- [Correspondance markdown → Typst](#correspondance-markdown--typst)
- [Lire le code généré](#lire-le-code-généré)

## Règles des blocs Typst

````markdown
```{=typst}
#align(center)[Ce paragraphe est centré.]
```
````

- Le bloc est en **mode balisage** Typst : les fonctions s'appellent avec `#` (`#figure(...)`), le texte s'écrit directement, `*gras*` et `_italique_` sont la syntaxe Typst.
- Il n'existe **pas** de Typst en ligne : un bloc forme toujours son propre paragraphe.
- Une règle `#set` ou `#show` posée dans un bloc s'applique **jusqu'à la fin du document** (elle n'est pas limitée au bloc). Pour la limiter, l'entourer de crochets : `#[ #set text(size: 9pt) … ]`.
- **Chemins** : toujours commencer par `/`, qui désigne le **dossier du fichier markdown** : `image("/images/schema.png")`. Un chemin sans `/` est cherché dans un dossier interne et échoue (`file not found (searched at /__combava__/…)`).
- Fonctions disponibles en plus de celles de Typst :
    - `callout(kind, body)`, `kind` ∈ `"note"`, `"tip"`, `"important"`, `"warning"`, `"caution"` (fournie par le template) ;
    - `mi("…")` et `mitex("…")` (LaTeX en ligne / en bloc) **seulement si** le document contient déjà des maths markdown (`$…$`) : sinon `unknown variable: mi`. Préférer les maths markdown.
- Les labels Typst (`<fig-x>`) ne sont **pas connus de combava** : on ne peut les viser que depuis un bloc Typst (`@fig-x`), pas depuis le markdown (`[@fig-x]` → citation, `[…](#fig-x)` → `broken-link`).
- Inversement, depuis un bloc Typst, on peut viser les labels posés par combava : `@tab-etapes`, `#link(<introduction>)[…]`.
- Ne pas importer de package autre que ceux déjà disponibles sans raison : chaque package demande un téléchargement au premier usage.
- Une erreur dans un bloc est signalée à la bonne ligne du `.md` (code `typst`, message en anglais).

## Recettes vérifiées

### Image à taille fixe

````markdown
```{=typst}
#figure(image("/images/schema.png", width: 60%), caption: [Mon schéma])
```
````

`width:` en `%` de la largeur de texte ou en unité (`8cm`) ; ou `height: 5cm`. Sans `caption`, la figure n'a pas de légende mais reste numérotée et dans la liste des figures ; pour une image seule non numérotée : `#align(center, image("/images/logo.png", width: 30%))`.

### Figure avec renvoi

````markdown
```{=typst}
#figure(image("/images/pipeline.svg", width: 50%), caption: [Le pipeline]) <fig-pipeline>
```

```{=typst}
La @fig-pipeline montre les trois étapes.
```
````

`@fig-pipeline` affiche « Fig. 1 », cliquable. La phrase qui contient le renvoi doit elle-même être dans un bloc Typst.

### Images côte à côte

````markdown
```{=typst}
#figure(
  grid(columns: 2, gutter: 1em,
    image("/images/avant.png"),
    image("/images/apres.png"),
  ),
  caption: [Avant et après],
)
```
````

### Tableau avec cellules fusionnées ou colonnes dimensionnées

````markdown
```{=typst}
#figure(
  table(
    columns: (auto, 1fr, 1fr),
    table.header[Étape][Entrée][Sortie],
    table.cell(colspan: 2)[Parsing et émission], [Typst],
    [Compilation], [Typst], [PDF],
  ),
  caption: [Les étapes],
) <tab-fusion>
```
````

Le style du template (filets, en-tête bleu, coupure entre pages) s'applique. Label `<tab-fusion>` visible seulement depuis un bloc Typst (`@tab-fusion` → « Tableau N »). Les tableaux markdown et Typst partagent la même numérotation.

### Chapitre non numéroté (remerciements, résumé, glossaire)

````markdown
```{=typst}
#heading(numbering: none)[Remerciements]
```

Texte des remerciements, en markdown normal.

# Introduction
````

- Nouvelle page, style de chapitre sans l'étiquette « Chapitre N », présent dans la table des matières sans numéro.
- Placé avant le premier `#`, il apparaît après les tables (table des matières, listes), dans la partie numérotée en chiffres arabes.
- Pour l'exclure de la table des matières : `#heading(numbering: none, outlined: false)[Résumé]`.
- Pas de label combava : `[[Remerciements]]` → `broken-link`.

### Annexes numérotées A, B, C

````markdown
# Conclusion

Texte.

```{=typst}
#heading(numbering: none)[Annexes]
#set heading(numbering: "A.1")
#counter(heading).update(0)
```

# Code source

## Module principal

# Résultats bruts
````

- « Annexes » : page de séparation non numérotée ; puis « Chapitre A — Code source », section « A.1 », « Chapitre B »…
- La **bibliographie reste à la toute fin**, après les annexes : on ne peut pas la placer avant.
- Les titres d'annexe écrits en markdown gardent leur label automatique : `[[Code source]]` fonctionne.

### Mise en forme ponctuelle

````markdown
```{=typst}
#align(center)[*Texte centré en gras*]
#text(fill: rgb("#b42318"))[Texte rouge]
#text(size: 9pt)[Petit texte]
#v(1cm)
#pagebreak()
```
````

- Préférer un saut de page markdown (`---` précédé d'une ligne vide) à `#pagebreak()`.
- Encadré écrit en Typst (par exemple dans une grille) : `#callout("warning")[Contenu avec *gras*.]`.

### Exposant ou indice dans du texte

Pas besoin de Typst : `$x^2$`, `$\text{CO}_2$` en maths markdown. En Typst pur : `#super[e]`, `#sub[2]` (`XIX#super[e] siècle`).

## Correspondance markdown → Typst

Ce que combava émet (la forme exacte peut varier, la sémantique est garantie). Utile pour lire la sortie de `--transpile-only`.

| Markdown | Typst émis |
|---|---|
| texte | texte échappé : `\ # $ * _ @ < > [ ] ` ~ / = - +` précédés de `\`, et `.` après un chiffre |
| retour à la ligne simple / forcé | espace / `#linebreak()` |
| `*a*`, `**a**`, `~~a~~`, `` `a` `` | `#emph[a]`, `#strong[a]`, `#strike[a]`, `#raw("a")` |
| `## Titre {#id}` | `#heading(level: 2)[Titre]#label("id")` |
| `[t](https://u)`, `<https://u>` | `#link("https://u")[t]` |
| `[t](#id)`, `[[id\|t]]` | `#link(label("id"))[t]` |
| image seule | `#figure(image("/chemin", alt: "…"), caption: […])` |
| image dans une phrase | `#box(image("/chemin", alt: "…", height: 1em))` |
| `> citation` | `#quote(block: true)[…]` |
| `> [!NOTE]` | `#callout("note")[…]` |
| liste à puces / numérotée | `#list(tight: …, […])` / `#enum(start: n, tight: …, […])` |
| `- [x]`, `- [ ]` | `☒ `, `☐ ` en début d'élément |
| liste de définitions | `#terms(terms.item[terme][définition])` |
| tableau | `#table(columns: n, align: (…), table.header(…), …)` |
| tableau + `: Légende {#tab}` | `#figure(table(…), caption: […])#label("tab")` |
| bloc de code | `#raw(block: true, lang: "rust", "…")` |
| bloc de code + `{caption="…"}` | `#figure(raw(block: true, …), caption: […])` |
| ` ```{=typst} ` | contenu recopié tel quel |
| `[^x]` (premier appel / suivants) | `#footnote[…] <combava-fn-x>` / `#footnote(<combava-fn-x>)` |
| `[@cle]`, `[@a; @b]` | `#cite(label("cle"))`, une fois par clé |
| `[@tab]` (label de tableau) | `#ref(label("tab"))` |
| `$…$`, `$$…$$` | `#mi("…")`, `#mitex("…")` |
| `---` (après une ligne vide) | `#pagebreak()` ; dans un conteneur : `#line(length: 100%)` |
| HTML | rien (avertissement `raw-html`) |

Structure du fichier généré :

1. `// Généré par combava-core <version> — ne pas modifier`
2. `#import "@preview/mitex:0.2.7": mi, mitex` si le document contient des maths
3. `#import "/__combava__/template/template.typ": template, callout`
4. `#show: template.with(title: …, …)` avec tous les champs de l'en-tête
5. le corps transpilé
6. `#bibliography("/__combava__/bibliography.bib")` si une bibliographie est déclarée

Chemins virtuels : `/__combava__/template/` = dossier du template, `/__combava__/bibliography.bib` = le `.bib` résolu, tout autre `/x` = `<dossier du .md>/x` (lecture seule).

## Lire le code généré

```bash
combava build rapport.md --transpile-only   # écrit rapport.typ à côté de rapport.md
```

- Sert à comprendre une erreur « (ligne N du code généré, voir --transpile-only) ».
- Le `.typ` produit n'est **pas** compilable tel quel avec `typst` (chemins virtuels `/__combava__/…`) : c'est un outil de lecture.
- Supprimer le `.typ` une fois l'erreur comprise ; ne jamais le modifier à la place du markdown.
