// Template de test : respecte le contrat de la section 9 de la spécification,
// sans mise en forme soignée. Le template par défaut est dans templates/default/.

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
  set text(lang: "fr")
  set document(title: title, author: authors)
  set page(header: if header-text != none { align(right, header-text) })
  set heading(numbering: "1.1")

  // Page de garde : chaque argument est affiché s'il est défini.
  for value in (school, university, academic-year, cohort, specialization, subject) {
    if value != none { value; linebreak() }
  }
  if title != none { text(2em, strong(title)); linebreak() }
  if subtitle != none { subtitle; linebreak() }
  if authors.len() > 0 { authors.join(", "); linebreak() }
  if teachers.len() > 0 { teachers.join(", "); linebreak() }
  if date != none { date }
  pagebreak()

  if toc { outline() }
  if list-of-figures { outline(target: figure.where(kind: image)) }
  if list-of-listings { outline(target: figure.where(kind: raw)) }
  body
}

#let callout(kind, body) = block(stroke: 1pt, inset: 8pt, width: 100%)[*#kind* : #body]
