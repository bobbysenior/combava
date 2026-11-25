# Erreurs et avertissements de combava

## Sommaire

- [Lire un message](#lire-un-message)
- [Tous les codes](#tous-les-codes)
- [Erreurs Typst](#erreurs-typst)
- [Problèmes sans message](#problèmes-sans-message)

## Lire un message

Les messages sont écrits sur la sortie d'erreur, au format des compilateurs :

```
rapport.md:3:1: erreur[unknown-key] : clé inconnue « autors »
rapport.md:12:5: avertissement[raw-html] : HTML brut ignoré
rapport.md:7:1: erreur[typst] : citation key `absent` is not present in the bibliography
  aide : …
combava: erreur[template-not-found] : …
```

- Ordre : fichier, ligne, colonne (comptées à partir de 1, en caractères), gravité, code, message.
- Une ligne `  aide : …` qui suit donne une piste.
- `combava: …` sans fichier : diagnostic sans position (configuration, template, réseau).
- **Erreur** : aucun PDF écrit, l'ancien reste intact. **Avertissement** : le PDF est produit.
- Toutes les erreurs sont rapportées en une seule fois.

## Tous les codes

### Erreurs (la compilation s'arrête)

| Code | Cause | Correction |
|---|---|---|
| `invalid-utf8` | fichier markdown qui n'est pas en UTF-8 | réenregistrer en UTF-8 (`iconv -f latin1 -t utf-8`) |
| `unclosed-frontmatter` | `+++` d'ouverture sans `+++` de fermeture | ajouter la ligne `+++` de fin |
| `invalid-toml` | syntaxe TOML invalide (guillemet manquant, valeur sans guillemets, crochet non fermé) | corriger à la position indiquée |
| `unknown-key` | clé inconnue dans l'en-tête ou un `config.toml` | corriger l'orthographe ; liste des clés dans `configuration.md` ; aucune clé libre |
| `invalid-type` | valeur du mauvais type : `date = 2026-10-10`, `authors = "Ada"`, `toc = "true"` | texte entre guillemets, listes entre crochets, booléens sans guillemets |
| `output-outside-frontmatter` | clé `output` dans un `config.toml` | la déplacer dans l'en-tête du document |
| `template-not-found` | template introuvable, ou dossier sans `template.typ` ; le message liste les emplacements essayés | corriger le nom ou le chemin (voir `templates.md`) |
| `bibliography-not-found` | fichier `.bib` absent, ou sans l'extension `.bib` | corriger le chemin (relatif au fichier où il est écrit) |
| `citation-without-bibliography` | `[@cle]` alors qu'aucune clé `bibliography` n'est définie, et `cle` n'est pas un label de tableau | déclarer `bibliography = "references.bib"`, ou corriger le label du tableau visé |
| `broken-link` | `[texte](#id)` ou `[[cible]]` vers un label qui n'existe pas | corriger l'identifiant : slug du titre (voir `syntaxe.md`, « Label automatique »), ou `{#id}` explicite |
| `duplicate-label` | même `{#id}` sur deux titres ou tableaux | renommer l'un des deux |
| `remote-image` | image en `http://` ou `https://` | télécharger l'image à côté du document, chemin relatif |
| `invalid-image-path` | image en chemin absolu, ou qui sort du dossier du `.md` (`../`) | déplacer l'image dans le dossier du document (ou un sous-dossier) |
| `package-download` | téléchargement d'un package Typst impossible (mitex, premier usage des maths) | se connecter à internet et relancer ; les erreurs Typst qui en découlent sont masquées |
| `file-exists` | `combava init` sur un fichier existant | autre nom, ou `--force` |
| `io` | lecture ou écriture impossible : fichier absent, droits, dossier de sortie inexistant, sortie = fichier `.md` | vérifier le chemin, créer le dossier de sortie |
| `typst` | erreur de compilation Typst | voir [Erreurs Typst](#erreurs-typst) |

### Avertissements (le PDF est produit)

| Code | Cause | Correction |
|---|---|---|
| `raw-html` | HTML brut, en bloc ou en ligne, ignoré | le retirer ; markdown équivalent, ou bloc ` ```{=typst} ` ; un commentaire `<!-- -->` ne déclenche rien |
| `invalid-label` | `{#id}` avec un caractère interdit : espace, `/`, `#`, `'`… (les lettres accentuées sont permises) | n'utiliser que lettres, chiffres, `-`, `_`, `.`, `:` ; sinon l'id est ignoré |
| `invalid-code-attributes` | attributs `{…}` d'un bloc de code invalides ou inconnus | seul `{caption="…"}` est reconnu |
| `undefined-footnote` | appel `[^x]` sans définition ; l'appel s'affiche en texte | ajouter `[^x]: …` |
| `unused-footnote` | définition `[^x]: …` jamais appelée | l'appeler, ou la supprimer |
| `typst` | avertissement de Typst | lire le message ; souvent sans conséquence |

Objectif : **zéro erreur et zéro avertissement**.

## Erreurs Typst

Code `typst`. Le message vient de Typst (en anglais).

| Où | Affichage |
|---|---|
| dans le code issu du markdown, le plus souvent un bloc ` ```{=typst} ` | ramené à la ligne du `.md`, colonne 1 |
| dans une ligne générée sans équivalent markdown | `rapport.md: … (ligne N du code généré, voir --transpile-only)` |
| dans le template | chemin réel du fichier ; template inclus dans le binaire : `<default>/template.typ` |
| dans un package | `@preview/mitex:0.2.7/…` |

Pour une erreur « du code généré » :

```bash
combava build rapport.md --transpile-only   # écrit rapport.typ
```

puis lire `rapport.typ` à la ligne N (correspondance markdown → Typst dans `typst.md`). Supprimer ce `.typ` ensuite.

Messages Typst fréquents :

| Message | Cause probable | Correction |
|---|---|---|
| `citation key `x` is not present in the bibliography` | clé `[@x]` absente du `.bib` (ou faute de casse) | corriger la clé ou compléter le `.bib` |
| `file not found (searched at /__combava__/images/…)` | chemin relatif sans `/` dans un bloc Typst | écrire `"/images/…"` (voir `typst.md`) |
| `file not found (searched at …)` sur une autre ressource | fichier absent du dossier du document | vérifier le chemin, relatif au `.md` |
| `unknown variable: …`, `expected …` | erreur de syntaxe dans un bloc Typst | corriger le Typst |
| `label <x> does not exist in the document` | `@x` dans un bloc Typst vers un label inexistant | poser `<x>` sur l'élément visé |
| `unresolved import`, avec une aide sur le template | template incomplet : il ne définit pas `template` ou `callout` | voir [Problèmes sans message](#problèmes-sans-message) |
| erreur dans une formule (mitex) | LaTeX non pris en charge par mitex | simplifier la formule |

## Problèmes sans message

| Symptôme | Cause | Correction |
|---|---|---|
| un paragraphe devient un titre | `---` juste sous le paragraphe, sans ligne vide | ajouter une ligne vide avant `---` |
| la légende du tableau apparaît comme une ligne du tableau | pas de ligne vide entre le tableau et `: Légende` | ajouter la ligne vide |
| `[@cle, p. 12]` affiché tel quel | forme non reconnue | `[@cle], p. 12` |
| page « Bibliographie » vide | `bibliography` déclarée sans aucune citation | citer, ou retirer la clé |
| astérisques visibles dans le titre de couverture | markdown dans l'en-tête | texte brut seulement |
| école ou université inattendue sur la couverture | valeur venue de `.combava/config.toml` ou de la configuration globale | la surcharger dans l'en-tête, ou corriger la couche concernée |
| une modification de `config.toml` est sans effet | une couche plus prioritaire définit la même clé | vérifier l'en-tête puis `.combava/config.toml` |
| logos génériques sur la couverture | logos d'exemple du template | remplacer les images du template installé (`templates.md`) |
| `make install` n'a pas mis à jour le template | voulu, pour garder les logos | `make install FORCE=1` après sauvegarde des logos |
| image trop grande | pas de taille en markdown | bloc Typst avec `width:` (`typst.md`) |
| encadré qui saute à la page suivante en laissant un blanc | un encadré ne se coupe pas | le raccourcir ou le scinder |
