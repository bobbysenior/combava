# Templates de combava

Le template décide de toute la mise en page : couverture, polices, couleurs, en-têtes. Le markdown ne change pas : pour une autre apparence, on change de template, pas le document.

## Sommaire

- [Le template par défaut](#le-template-par-défaut)
- [Où combava cherche un template](#où-combava-cherche-un-template)
- [Personnaliser le template par défaut](#personnaliser-le-template-par-défaut)
- [Créer un template](#créer-un-template)
- [Contrat d'un template](#contrat-dun-template)

## Le template par défaut

### Ordre des pages

1. **Couverture** (sans numéro de page).
2. **Tables** demandées, pages numérotées en chiffres romains (i, ii…) : « Table des matières » (`toc`, niveaux 1 à 3), « Liste des figures » (`list_of_figures`, images seulement), « Liste des codes » (`list_of_listings`, blocs de code légendés).
3. **Corps**, numéroté à partir de 1. Chaque `#` ouvre un chapitre sur une nouvelle page.
4. **Bibliographie**, si déclarée, en dernier chapitre non numéroté.

### Couverture

Chaque élément n'apparaît que si sa clé est définie :

| Zone | Clés |
|---|---|
| logos en haut | logo de l'université si `university`, logo de l'école si `school` |
| surtitre (étiquette corail) | `subject` |
| titre (34 pt), sous-titre (italique) | `title`, `subtitle` |
| grille en bas de page, deux colonnes | « Auteur » / « Auteurs » : `authors` ; « Enseignant » / « Enseignants » : `teachers` ; « Formation » : `specialization`, `cohort`, puis « Année universitaire `academic_year` » ; « Établissement » : `school`, `university` ; « Date » : `date` |

- Les étiquettes (« Auteurs », « Formation »…) sont fixes : ne pas les répéter dans les valeurs.
- `academic_year` est préfixé de « Année universitaire » : écrire `academic_year = "2026-2027"`, pas `"Année universitaire 2026-2027"`.
- Valeurs affichées telles quelles, une par ligne.

### Mise en page du corps

| Élément | Rendu |
|---|---|
| page | A4, marges 2,5 cm × 2,6 cm, texte 11 pt Libertinus Serif, justifié, `lang: "fr"` |
| en-tête de page | `header_text` (ou le titre) à gauche, chapitre en cours à droite ; absent sur la page qui ouvre un chapitre |
| pied de page | numéro de page centré |
| chapitre (`#`) | nouvelle page, étiquette « Chapitre N », grand titre bleu, filet corail |
| sections | `1.1` (`##`), `1.1.1` (`###`) numérotées ; `####` sans numéro, en italique |
| légendes | sous l'élément : « Fig. N — … », « Tableau N — … », « Liste N — … » |
| tableaux | style « livre » : filets horizontaux, en-tête en gras sur fond bleu pâle ; coupés entre deux pages si trop hauts |
| code | DejaVu Sans Mono, fond gris clair |
| citation en bloc | italique, filet corail à gauche |
| encadrés | Note (bleu), Astuce (vert), Important (violet), Attention (orange), Prudence (rouge) ; jamais coupés entre deux pages |
| bibliographie | style ISO 690 auteur-date, « (Knuth 1984) » |
| couleurs | bleu outremer `#203363`, accent corail `#e4572e` |

## Où combava cherche un template

La clé `template` (ou l'option `-t`) accepte un **nom** ou un **chemin**.

- **Chemin** : la valeur contient `/` ou `\`, ou commence par `.` ou `~`. Elle désigne directement le dossier du template : `template = "./mon-template"` (relatif à l'endroit où la clé est écrite, voir `configuration.md`).
- **Nom** : cherché dans cet ordre, le premier dossier existant l'emporte :
    1. `<racine du projet>/.combava/templates/<nom>/`
    2. `<config globale>/templates/<nom>/` (Linux : `~/.config/combava/templates/<nom>/`)
    3. les templates inclus dans le binaire : seulement `default`.
- Un dossier nommé `default` dans le projet ou la configuration globale **remplace** le template inclus. `make install` en installe un dans `~/.config/combava/templates/default/`.
- Le premier dossier trouvé l'emporte **même s'il est incomplet** (pas de repli silencieux sur le suivant).
- Introuvable, ou dossier sans `template.typ` → erreur `template-not-found` (le message liste les emplacements essayés).

## Personnaliser le template par défaut

Fichiers du template installé (`~/.config/combava/templates/default/`) :

| Fichier | Contenu |
|---|---|
| `template.typ` | point d'entrée : fonctions `template` et `callout`, règles de mise en page |
| `couverture.typ` | page de couverture |
| `composants.typ` | encadrés des callouts |
| `utils.typ` | étiquettes, filets, logos |
| `theme.typ` | **toutes** les couleurs, polices et mesures (marges, taille du texte), titres des encadrés |
| `images/univ_logo.png`, `images/school_logo.png` | logos à remplacer, en gardant ces noms |

- Logos : remplacer les deux PNG du template installé.
- Couleurs, polices, marges : modifier `theme.typ`.
- `make install FORCE=1` écrase ces modifications : les sauvegarder avant.

## Créer un template

1. Partir d'une copie :

    ```bash
    cp -r ~/.config/combava/templates/default ~/.config/combava/templates/mon-template
    ```

    (ou dans `<projet>/.combava/templates/mon-template/` pour un template propre au projet)

2. Écrire `template = "mon-template"` dans l'en-tête, ou `combava build rapport.md -t mon-template`.
3. Modifier les fichiers `.typ`.
4. Polices propres : dossier `fonts/` dans le template (`.ttf`, `.otf`, `.ttc`, `.otc`) ; elles passent avant les polices du système, puis celles incluses dans Typst.
5. Les fichiers du template (images, sous-fichiers) se désignent par des chemins **relatifs à `template.typ`**.

Une erreur dans un fichier du template est affichée avec le chemin de ce fichier ; pour le template inclus : `<default>/template.typ`.

## Contrat d'un template

Un template est un dossier qui contient `template.typ`, lequel définit **deux** fonctions.

### `template`

```typst
#let template(
  title: none,             // str | none
  subtitle: none,          // str | none
  authors: (),             // array de str
  teachers: (),            // array de str
  date: none,              // str | none
  school: none,            // str | none
  university: none,        // str | none
  academic-year: none,     // str | none
  cohort: none,            // str | none
  specialization: none,    // str | none
  subject: none,           // str | none
  toc: true,               // bool
  list-of-figures: false,  // bool
  list-of-listings: false, // bool
  header-text: none,       // str | none, déjà remplacé par title si absent
  body,
) = { … }
```

- Arguments en kebab-case (les clés `academic_year` → `academic-year`, etc.). combava les passe **tous**, mais le template doit garder ces défauts.
- Le template **doit** :
    - fonctionner quand n'importe quel argument vaut `none` ou `()` ;
    - régler `set text(lang: "fr")` et `set document(title: …, author: …)` ;
    - placer la page de garde, puis les tables demandées : `toc` → `outline()`, `list-of-figures` → `outline(target: figure.where(kind: image))`, `list-of-listings` → `outline(target: figure.where(kind: raw))` ;
    - afficher `header-text` en haut de page.
- Il **peut** numéroter les titres et choisir le style de bibliographie (`set bibliography(style: …)`).
- Il ne doit pas lire `sys.inputs`.
- Il n'existe pas de variable libre : impossible de passer une clé d'en-tête personnalisée au template.

### `callout`

```typst
#let callout(kind, body) = { … }   // kind ∈ "note", "tip", "important", "warning", "caution"
```

### Erreurs liées au template

- `unresolved import` avec une aide sur le template : `template.typ` ne définit pas `template` ou `callout`, ou il est vide. Vérifier qu'un dossier `templates/default/` incomplet ne traîne pas dans `~/.config/combava/` ou dans le `.combava/` du projet.
- Erreur dans le préambule du code généré : le template n'accepte pas tous les arguments ci-dessus.
