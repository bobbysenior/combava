// Composants du corps du document.

#import "theme.typ": signalements
#import "utils.typ": etiquette

/// Encadré de mise en avant des callouts GFM (`> [!NOTE]`…).
/// - kind: "note", "tip", "important", "warning" ou "caution"
#let encadre(kind, body) = {
  assert(
    kind in signalements,
    message: "callout : type « " + str(kind) + " » inconnu, attendu : "
      + signalements.keys().map(k => "« " + k + " »").join(", "),
  )
  let (titre, trait, fond) = signalements.at(kind)
  block(
    width: 100%,
    breakable: false,
    above: 1.2em,
    below: 1.2em,
    fill: fond,
    stroke: (left: 3pt + trait),
    inset: (left: 14pt, right: 12pt, y: 11pt),
    radius: (right: 3pt),
    {
      set par(first-line-indent: 0pt, justify: false)
      etiquette(titre, fill: trait)
      v(5pt)
      body
    },
  )
}
