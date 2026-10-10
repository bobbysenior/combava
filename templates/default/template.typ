// Template par défaut de Combava (spécification, section 9).
// Inspiré de silky-report-insa
// https://typst.app/universe/package/silky-report-insa
//
// Ce template dérive de la base « Université de La Réunion », distribuée sous
// licence MIT :
//   Copyright (c) 2024 Université de La Réunion
//   Permission is hereby granted, free of charge, to any person obtaining a copy
//   of this software and associated files, to deal in it without restriction,
//   subject to inclusion of this notice. Provided "as is", without warranty.

// Il dessine la couverture, les tables (chiffres romains), puis le corps
// (chiffres arabes, la page 1 est celle du premier chapitre).
//
//   template.typ    point d'entrée : `template` et `callout`
//   couverture.typ  page de couverture
//   composants.typ  encadré des callouts
//   utils.typ       étiquettes, filets, logos
//   theme.typ       couleurs, polices, mesures
//   images/         logos de l'université et de l'école, à remplacer

#import "theme.typ": couleur, mise-en-page, police
#import "utils.typ": etiquette, filet-accent
#import "couverture.typ": couverture
#import "composants.typ": encadre

#let numero(it) = numbering(it.numbering, ..counter(heading).at(it.location()))

/// Titre de niveau 1 : un chapitre, sur une nouvelle page si `saut`.
#let chapitre(it, saut: true) = {
  if saut { pagebreak(weak: true) }
  v(1.2cm)
  block(width: 100%, below: 1.6cm, {
    if it.numbering != none {
      context etiquette("Chapitre " + numero(it), fill: couleur.accent, taille: 9.5pt)
      v(8pt)
    }
    set par(first-line-indent: 0pt, leading: 0.45em)
    text(size: 28pt, weight: "bold", fill: couleur.bleu, it.body)
    v(12pt)
    filet-accent()
  })
}

/// Typst interdit les sauts de page dans un conteneur : un titre de niveau 1
/// écrit dans une citation, une liste, une note ou un callout n'en fait pas.
#let sans-saut(corps) = {
  show heading.where(level: 1): chapitre.with(saut: false)
  corps
}

#let callout(kind, body) = encadre(kind, sans-saut(body))

#let template(
  title: none,
  subtitle: none,
  authors: (),
  teachers: (),
  date: none,
  school: none,
  university: none,
  academic-year: none,
  cohort: none,
  specialization: none,
  subject: none,
  toc: true,
  list-of-figures: false,
  list-of-listings: false,
  header-text: none,
  body,
) = {
  set document(title: title, author: authors)

  // ---------------------------------------------------------------- Texte ---
  set text(font: police.texte, size: mise-en-page.corps, lang: "fr", fill: couleur.encre)
  set par(justify: true, leading: 0.68em, spacing: 0.68em, first-line-indent: 1.2em)
  set list(marker: text(fill: couleur.accent)[•], indent: 0.6em, body-indent: 0.6em)
  set enum(numbering: n => text(fill: couleur.bleu, weight: "bold")[#n.], indent: 0.6em)
  show list: set block(above: 1.1em, below: 1.1em)
  show enum: set block(above: 1.1em, below: 1.1em)
  set footnote.entry(separator: line(length: 25%, stroke: 0.4pt + couleur.trait))
  set bibliography(style: "iso-690-author-date")

  show link: set text(fill: couleur.bleu)
  show link: underline.with(stroke: 0.5pt + couleur.accent, offset: 2pt)

  show quote.where(block: true): it => block(
    width: 100%,
    inset: (left: 16pt, y: 6pt),
    stroke: (left: 2.5pt + couleur.accent),
    {
      set par(first-line-indent: 0pt)
      text(style: "italic", size: 1.05em, it.body)
      if it.attribution != none {
        v(4pt)
        align(right, text(size: 9.5pt, fill: couleur.gris)[— #it.attribution])
      }
    },
  )

  // ---------------------------------------------------------------- Titres ---
  set heading(numbering: "1.1", supplement: "Section")
  show heading: set text(hyphenate: false)

  show heading.where(level: 1): chapitre
  show selector.or(quote, list, enum, terms, table, figure, footnote.entry): sans-saut

  show heading.where(level: 2): it => block(above: 2em, below: 1em, sticky: true, {
    set par(first-line-indent: 0pt)
    text(size: 15pt, weight: "bold", fill: couleur.bleu, {
      if it.numbering != none {
        context text(fill: couleur.accent, numero(it))
        h(0.7em)
      }
      it.body
    })
  })

  show heading.where(level: 3): it => block(above: 1.6em, below: 0.8em, sticky: true, {
    set par(first-line-indent: 0pt)
    text(size: 12pt, weight: "semibold", fill: couleur.encre, {
      if it.numbering != none {
        context text(fill: couleur.gris, numero(it))
        h(0.6em)
      }
      it.body
    })
  })

  show heading.where(level: 4): it => block(above: 1.3em, below: 0.6em, sticky: true, {
    set par(first-line-indent: 0pt)
    text(size: 11pt, style: "italic", weight: "semibold", fill: couleur.bleu, it.body)
  })

  // ------------------------------------------------------- Tables des matières ---
  set outline(indent: 1.4em, depth: 3)
  show outline.entry.where(level: 1): it => if it.element.func() == heading {
    set text(weight: "bold", fill: couleur.bleu)
    block(above: 1.1em, it)
  } else {
    it
  }
  show outline.entry: set block(above: 0.7em)

  // ------------------------------------------------------------- Figures ---
  set figure(gap: 0.9em)
  show figure: set block(above: 1.6em, below: 1.6em)
  show figure.where(kind: table): set figure.caption(position: top)
  set figure.caption(separator: [ — ])
  show figure.caption: it => context {
    set par(first-line-indent: 0pt)
    set text(size: 9.5pt)
    text(weight: "bold", fill: couleur.bleu)[#it.supplement~#it.counter.display(it.numbering)#it.separator]
    it.body
  }

  // ------------------------------------------------------------ Tableaux ---
  // Style « livre » : un filet au-dessus, un sous l'en-tête, un en dessous ;
  // pas de quadrillage.
  set table(
    inset: (x: 9pt, y: 7pt),
    stroke: (_, y) => if y > 0 { (top: 0.4pt + couleur.trait) },
    fill: (_, y) => if y == 0 { couleur.bleu-pale },
  )
  show table.cell.where(y: 0): set text(weight: "bold", fill: couleur.bleu)
  show table: it => block(
    stroke: (top: 1.2pt + couleur.bleu, bottom: 1.2pt + couleur.bleu),
    it,
  )

  // ---------------------------------------------------------------- Code ---
  show raw: set text(font: police.code)
  // Aligné à gauche même dans une figure, qui centre son contenu.
  show raw.where(block: true): it => block(
    width: 100%,
    fill: couleur.fond-code,
    radius: 3pt,
    inset: (x: 11pt, y: 9pt),
    align(left, it),
  )
  show raw.where(block: false): it => box(
    fill: couleur.fond-code,
    radius: 2pt,
    inset: (x: 3pt),
    outset: (y: 3pt),
    it,
  )

  // ================================================================ PAGES ===

  // Couverture : page 1 physique, sans numéro.
  couverture(
    title: title,
    subtitle: subtitle,
    subject: subject,
    authors: authors,
    teachers: teachers,
    date: date,
    school: school,
    university: university,
    academic-year: academic-year,
    cohort: cohort,
    specialization: specialization,
  )

  // En-tête : `header-text` à gauche, chapitre courant à droite. Pas d'en-tête
  // sur la page où s'ouvre un chapitre.
  let en-tete = context {
    let ici = here().page()
    let chapitres = query(heading.where(level: 1))
    let ouvre-un-chapitre = chapitres.any(h => h.location().page() == ici)
    // Premier chapitre numéroté : avant lui, ce sont les tables.
    let corps-commence = chapitres.any(h => h.numbering != none and h.location().page() <= ici)
    if not ouvre-un-chapitre {
      let precedents = if corps-commence { query(heading.where(level: 1).before(here())) } else { () }
      block(
        width: 100%,
        stroke: (bottom: 0.4pt + couleur.trait),
        inset: (bottom: 7pt),
        grid(
          columns: (1fr, auto),
          column-gutter: 1em,
          if header-text != none { etiquette(header-text) },
          if precedents.len() > 0 {
            text(size: 9pt, style: "italic", fill: couleur.gris, precedents.last().body)
          },
        ),
      )
    }
  }

  let pied = context align(
    center,
    text(size: 9.5pt, weight: "semibold", fill: couleur.bleu, counter(page).display()),
  )

  set page(
    paper: "a4",
    margin: (x: mise-en-page.marge-x, y: mise-en-page.marge-y),
    header: en-tete,
    footer: pied,
    numbering: "i",
  )
  counter(page).update(1)

  // ------------------------------------------------------------- Tables ---
  if toc {
    outline(title: "Table des matières")
  }
  if list-of-figures {
    outline(title: "Liste des figures", target: figure.where(kind: image))
  }
  if list-of-listings {
    outline(title: "Liste des codes", target: figure.where(kind: raw))
  }

  // -------------------------------------------------------------- Corps ---
  set page(numbering: "1")
  counter(page).update(1)

  body
}
