// Couverture : arcs concentriques (un sonar, la mer), un point d'accent, le
// bleu outremer en aplat.

#import "theme.typ": couleur
#import "utils.typ": etiquette, logo-ecole, logo-universite, rangee-logos

/// Arcs concentriques centrés sur un coin de la page, avec un point d'accent
/// posé sur l'un d'eux (`rang-point`, à `angle-point` de l'horizontale). À
/// utiliser en `background` de page.
#let arcs(coin: bottom + right, trait: couleur.trait, rayon-pas: 38pt, n: 12, rang-point: 9, angle-point: 50deg) = {
  let (sx, sy) = (
    if coin.x == right { 1 } else { -1 },
    if coin.y == bottom { 1 } else { -1 },
  )
  block(width: 100%, height: 100%, clip: true, {
    for i in range(1, n + 1) {
      let r = i * rayon-pas
      place(coin, dx: sx * r, dy: sy * r, circle(radius: r, stroke: 0.6pt + trait))
    }
    // Point d'accent sur un arc, dans une zone généralement libre de texte.
    let r = rang-point * rayon-pas
    let (dx, dy) = (r * calc.cos(angle-point) - 5pt, r * calc.sin(angle-point) - 5pt)
    place(coin, dx: -sx * dx, dy: -sy * dy, circle(radius: 5pt, fill: couleur.accent))
  })
}

/// Page de couverture. Les arguments sont ceux du template (spécification,
/// section 9.1) ; chacun peut valoir `none` ou `()`, et n'est alors pas affiché.
/// Le logo de l'université et celui de l'école n'apparaissent que si
/// `university` ou `school` est défini.
#let couverture(
  title: none,
  subtitle: none,
  subject: none,
  authors: (),
  teachers: (),
  date: none,
  school: none,
  university: none,
  academic-year: none,
  cohort: none,
  specialization: none,
) = {
  // Un champ : étiquette + valeurs empilées.
  let champ(nom, valeurs) = {
    etiquette(nom)
    v(5pt)
    stack(spacing: 4pt, ..valeurs.map(v => text(size: 12.5pt, fill: couleur.encre, v)))
  }
  // Un champ affiché seulement s'il a au moins une valeur.
  let champs = (
    (("Auteur", "Auteurs"), authors),
    (("Enseignant", "Enseignants"), teachers),
    (("Formation",), (
      specialization,
      cohort,
      if academic-year != none { "Année universitaire " + academic-year },
    )),
    (("Établissement",), (school, university)),
    (("Date",), (date,)),
  ).map(((noms, valeurs)) => {
    let valeurs = valeurs.filter(v => v != none)
    if valeurs.len() > 0 { champ(noms.at(calc.min(valeurs.len(), noms.len()) - 1), valeurs) }
  }).filter(c => c != none)

  let logos = (
    if university != none { logo-universite },
    if school != none { logo-ecole },
  ).filter(l => l != none)

  page(
    margin: (left: 3.2cm, right: 2.5cm, y: 2.5cm),
    header: none,
    footer: none,
    numbering: none,
    background: {
      arcs(trait: couleur.bleu.lighten(85%))
      // Le « dos » : une bande bleue sur toute la hauteur, avec une pointe corail.
      place(top + left, rect(width: 12pt, height: 100%, fill: couleur.bleu))
      place(top + left, dy: 2.5cm, rect(width: 12pt, height: 3.2cm, fill: couleur.accent))
    },
    {
      rangee-logos(logos, hauteur: 34pt)

      v(1fr)

      if subject != none {
        etiquette(subject, fill: couleur.accent, taille: 10pt)
        v(12pt)
      }
      if title != none {
        par(leading: 0.58em, justify: false, first-line-indent: 0pt, text(
          size: 34pt,
          weight: "bold",
          fill: couleur.bleu,
          hyphenate: false,
          title,
        ))
        v(16pt)
        line(length: 2.4cm, stroke: 3.5pt + couleur.accent)
      }
      if subtitle != none {
        v(14pt)
        text(size: 15pt, style: "italic", fill: couleur.gris, subtitle)
      }

      v(1fr)

      if champs.len() > 0 {
        line(length: 100%, stroke: 0.5pt + couleur.trait)
        v(14pt)
        grid(columns: (1fr, 1fr), column-gutter: 24pt, row-gutter: 20pt, ..champs)
      }
    },
  )
}
