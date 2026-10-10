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

# Introduction

Ce document sert de référence : il utilise chaque fonctionnalité décrite dans la section 4 de la spécification. Il doit se compiler **sans aucun avertissement**.

Combava transforme du markdown en PDF en passant par Typst [@typst2023], comme le résume la figure ci-dessous.

![Les trois étapes de Combava](images/pipeline.svg "Le pipeline markdown → Typst → PDF")

## Texte

Le texte est *mis en emphase*, **mis en gras**, ~~barré~~ ou écrit comme du `code en ligne`. Les caractères spéciaux de Typst s'affichent tels quels : # $ * _ @ < > [ ] ` ~ / = - + et \\. Une adresse comme ada@exemple.fr n'est pas une citation, et l'année 2026\. ne devient pas une liste.

Les "guillemets droits" deviennent des guillemets français, et l'apostrophe d'« aujourd'hui » est typographique.

Un retour à la ligne simple
ne coupe pas le paragraphe, alors qu'un retour forcé  
le coupe.

## Titres et liens {#liens}

Chaque titre reçoit un label : [l'introduction](#introduction) a un label automatique, [cette section](#liens) un label explicite. Les wikilinks fonctionnent aussi : [[Introduction]] ou [[Blocs de code|les blocs de code]].

Les liens externes s'écrivent [comme ceci](https://typst.app), en autolien <https://spec.commonmark.org> ou en courriel <contact@exemple.fr>.

# Contenu

## Listes

- Une liste à puces
- avec un élément
  - imbriqué
- [x] une tâche faite
- [ ] une tâche à faire

1. Une liste numérotée
2. qui commence à 1

7) Une liste qui commence à 7

8) et dont les éléments sont séparés par une ligne vide.

Terme
: Sa définition.
: Une seconde définition.

## Tableau

| Étape | Entrée | Sortie |
|:------|:------:|-------:|
| Parsing | markdown | arbre |
| Émission | arbre | Typst |
| Compilation | Typst | PDF |

## Citations et callouts

> Le markdown est fait pour être lu tel quel, sans rendu.
>
> Une citation peut contenir plusieurs paragraphes.

> [!NOTE]
> Une note.

> [!TIP]
> Une astuce, avec une icône ![icône](images/icone.svg) dans le texte.

> [!IMPORTANT]
> Une information importante.

> [!WARNING]
> Un avertissement.

> [!CAUTION]
> Une mise en garde.

## Blocs de code

Seuls les blocs légendés apparaissent dans la table des codes.

```rust {caption="La fonction principale de Combava"}
pub fn transpile(markdown: &str, config: &Config) -> Result<Output, TranspileError> {
    todo!()
}
```

```python
print("Ce bloc n'a pas de légende")
```

    Un bloc indenté, sans langage.

```{=typst}
#align(center)[Ce paragraphe est écrit directement en Typst.]
```

## Notes, références et maths

Une note de bas de page[^1], une autre en deux paragraphes[^detail], puis de nouveau la première[^1].

[^1]: Le texte de la première note.

[^detail]: Le premier paragraphe de la note.

    Son second paragraphe.

Le livre de Knuth [@knuth1984] et la spécification de CommonMark [@commonmark; @typst2023] sont cités.

Une formule en ligne $E = mc^2$, et une formule centrée :

$$
\sum_{i=1}^{n} i = \frac{n(n+1)}{2}
$$

---

# Conclusion

Ce document s'est compilé sans avertissement : la page précédente s'est terminée par un saut de page.
