// Jetons de design : toute valeur de style (couleur, police, mesure) vit ici.
// Les autres fichiers n'écrivent jamais de couleur ni de dimension « en dur ».
//
// Parti pris : une identité (bleu outremer), un accent chaud (corail), un
// encre, un gris. Rien d'autre. Les polices sont celles embarquées par Typst :
// le rendu est identique sur toutes les machines.

#let couleur = (
  bleu: rgb("#203363"),                    // identité : bleu outremer
  bleu-pale: rgb("#203363").lighten(93%),  // aplats discrets
  accent: rgb("#e4572e"),                  // corail : numéros, filets, détails
  encre: rgb("#1b1f2a"),                   // texte courant
  gris: rgb("#5f6675"),                    // texte secondaire
  trait: rgb("#d5d9e2"),                   // filets et contours
  fond-code: rgb("#f4f5f8"),
)

#let police = (
  texte: "Libertinus Serif",
  code: "DejaVu Sans Mono",
)

#let mise-en-page = (
  marge-x: 2.5cm,
  marge-y: 2.6cm,
  corps: 11pt,
)

// Encadrés `callout`, un par type de la spécification (section 9.2) :
// (titre, couleur du filet, couleur de fond).
#let signalements = (
  note: ("Note", couleur.bleu, couleur.bleu-pale),
  tip: ("Astuce", rgb("#1f7a4d"), rgb("#e8f5ee")),
  important: ("Important", rgb("#5b3fa8"), rgb("#f1edfa")),
  warning: ("Attention", rgb("#a15c07"), rgb("#fdf3e0")),
  caution: ("Prudence", rgb("#b42318"), rgb("#fdecea")),
)
