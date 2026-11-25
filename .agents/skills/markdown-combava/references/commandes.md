# Ligne de commande de combava

## Sommaire

- [Commandes](#commandes)
- [`combava build`](#combava-build)
- [`combava init`](#combava-init)
- [Codes de sortie](#codes-de-sortie)
- [Installation et mise à jour](#installation-et-mise-à-jour)
- [Déroulement d'une compilation](#déroulement-dune-compilation)

## Commandes

```bash
combava build rapport.md          # compile en rapport.pdf, à côté du .md
combava init rapport.md           # crée un document prêt à remplir
combava --version
combava help                      # aide générale ; aussi : combava help build, combava build --help
```

Il n'existe que `build` et `init` (pas de `watch`, pas de `new`, pas de `check`).

## `combava build`

```bash
combava build [OPTIONS] <FILE>
```

| Option | Effet |
|---|---|
| `-o`, `--out <FILE>` | écrit le PDF ailleurs ; le **dossier doit exister** ; l'emporte sur la clé `output` |
| `-t`, `--template <TEMPLATE>` | nom ou chemin d'un template ; l'emporte sur la clé `template` |
| `--transpile-only` | écrit le code Typst généré (`<nom>.typ`) au lieu du PDF, pour comprendre une erreur |

- Chemins des options relatifs au **dossier courant**.
- Sortie par défaut : `<dossier du .md>/<nom du .md>.pdf`.
- Le PDF n'est écrit **que si tout a réussi** (fichier temporaire puis renommage) : un échec laisse l'ancien PDF intact. Ne pas conclure qu'une compilation a réussi parce qu'un PDF existe : regarder le code de sortie et les messages.
- Une sortie identique au fichier markdown (`-o rapport.md`) → erreur `io`.
- Les avertissements n'empêchent pas le PDF.
- Le premier document qui contient des maths télécharge le package Typst mitex : connexion internet requise une fois.

Exemples :

```bash
combava build rapport.md -o build/rapport.pdf     # build/ doit exister
combava build rapport.md -t ./mon-template
combava build rapport.md --transpile-only         # écrit rapport.typ
```

Sans binaire installé, depuis le dépôt de combava :

```bash
cargo run -p combava-cli -- build chemin/vers/rapport.md
```

## `combava init`

```bash
combava init [OPTIONS] [FILE]
```

| Option | Effet |
|---|---|
| `[FILE]` | fichier à créer ; `rapport.md` par défaut |
| `-t`, `--template <TEMPLATE>` | ajoute `template = "<TEMPLATE>"` à l'en-tête |
| `--force` | écrase le fichier s'il existe déjà |

- Fichier existant sans `--force` → erreur `file-exists`.
- Contenu créé : l'en-tête de `configuration.md` (titre, sous-titre, matière, auteurs, enseignants, date du jour, tables), puis `# Introduction`.
- Le fichier créé compile tel quel.

## Codes de sortie

| Code | Sens |
|---|---|
| `0` | succès, avertissements compris |
| `1` | erreur du document, de la configuration, du template ou de la compilation (aucun PDF écrit) |
| `2` | commande mal écrite (option inconnue, argument manquant) |

## Installation et mise à jour

Depuis la racine du dépôt combava (Rust stable et `make` requis) :

```bash
make install            # binaire ~/.local/bin/combava, config et template dans ~/.config/combava/
make install FORCE=1    # remplace aussi la config et le template installés (écrase les logos !)
make uninstall          # retire le binaire, garde ~/.config/combava/
```

| Installé | Où (Linux) |
|---|---|
| binaire | `~/.local/bin/combava` |
| configuration globale | `~/.config/combava/config.toml` |
| template par défaut modifiable | `~/.config/combava/templates/default/` |

- macOS : `~/Library/Application Support/combava/` au lieu de `~/.config/combava/`.
- Windows, sans `make` : `cargo install --path crates/combava-cli` (binaire dans `~/.cargo/bin`, template inclus, pas de configuration globale créée).
- `make install` sans `FORCE` remplace le binaire mais **garde** la configuration et le template installés.
- `~/.local/bin` doit être dans le `PATH` (`export PATH="$HOME/.local/bin:$PATH"`).

## Déroulement d'une compilation

Utile pour savoir à quelle étape une erreur se produit.

1. Lecture du fichier (doit être en UTF-8, sinon `invalid-utf8`).
2. Découpage de l'en-tête `+++`.
3. Recherche de la racine du projet, lecture et fusion des couches de configuration.
4. Résolution du template, de la bibliographie et du fichier de sortie.
5. Transpilation markdown → Typst ; affichage des diagnostics du markdown ; arrêt en cas d'erreur.
6. Avec `--transpile-only` : écriture du `.typ`, arrêt.
7. Compilation Typst (diagnostics de code `typst`).
8. Écriture du PDF.

Toutes les erreurs d'une étape sont rapportées en une fois : corriger toute la liste avant de relancer.
