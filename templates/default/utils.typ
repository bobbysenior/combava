// Utilitaires partagés : étiquettes, filets, rangée de logos.

#import "theme.typ": couleur

/// Étiquette : petites capitales espacées, pour les surtitres et les légendes
/// de champs. C'est la signature typographique du template.
#let etiquette(corps, fill: couleur.gris, taille: 8.5pt) = text(
  size: taille,
  weight: "semibold",
  tracking: 0.14em,
  fill: fill,
  upper(corps),
)

/// Filet fin avec une pointe d'accent à gauche : le détail récurrent du
/// template (sous les titres de chapitre).
#let filet-accent(largeur-accent: 2cm) = box(width: 100%, {
  line(length: 100%, stroke: 0.5pt + couleur.trait)
  place(top + left, line(length: largeur-accent, stroke: 2.5pt + couleur.accent))
})

// =============================================================================
// Logos
// =============================================================================

/// Logos fournis avec le template : des images à remplacer par les vraies, sous
/// le même nom. Chaque entrée est `(logo: image, echelle: 1.4)` : le facteur
/// compense le poids optique, un logo empilé paraissant plus petit qu'un
/// bandeau à hauteur égale.
#let logo-universite = (logo: image("images/univ_logo.png"), echelle: 1)
#let logo-ecole = (logo: image("images/school_logo.png"), echelle: 1.4)

/// Aligne une liste de logos sur une seule rangée : le premier à gauche, le
/// dernier à droite, les autres répartis, tous centrés verticalement.
#let rangee-logos(logos, hauteur: 34pt) = {
  let n = logos.len()
  if n == 0 { return none }
  let cellules = logos.map(l => {
    set image(height: hauteur * l.echelle)
    l.logo
  })
  let alignements = range(n).map(i => {
    (if i == 0 { left } else if i == n - 1 { right } else { center }) + horizon
  })
  grid(columns: n * (1fr,), align: alignements, ..cellules)
}
