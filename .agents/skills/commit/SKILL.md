---
name: commit
description: Convention de message de commit — conventional commits rédigés en français, avec une espace avant les deux-points. À utiliser dès qu'un commit est créé, relu ou reformulé.
---

# Convention de commit

Format : `type(scope) : sujet`, avec **une espace avant les deux-points**.

```
feat(auth) : connexion par jeton à usage unique
fix(cache) : purge de l'entrée expirée avant lecture
docs(api) : documenter la pagination des résultats
```

Avant le premier commit dans un dépôt, jeter un œil à `git log --oneline -30` :
si le dépôt suit déjà une autre convention, c'est la sienne qui prime.

## Sujet

- **En français**, avec les accents (jamais dépouillés), et les guillemets `« »` pour les citations.
- **Minuscule** après le séparateur, **pas de point final**.
- Groupe nominal (« connexion par jeton à usage unique ») ou verbe à l'infinitif (« borner la taille du cache »).
- ~70 caractères, 85 max.
- Le scope est facultatif (`docs : refonte de l'arborescence par thème`) mais quasi toujours présent.

## Types

| Type | Usage |
|---|---|
| `feat` | nouvelle fonctionnalité ou module |
| `fix` | correction de bug |
| `docs` | documentation (README, site, rapports) |
| `refactor` | réorganisation sans changement de comportement |
| `style` | formatage seul (indentation, passage d'un formateur, mise en page) |
| `test` | tests |
| `chore` | intendance (relances CI, renommages, gitignore) |
| `perf` | performance |
| `hardening` | durcissement sécurité (type maison, hors spec) |

: types utilisés

## Scope

Un mot, minuscule, tiré du domaine touché : composant, module, paquet ou
fonctionnalité (`api`, `auth`, `cache`, `ui`, `ci`…). Reprendre les scopes déjà
employés dans le dépôt plutôt que d'en inventer :

```shell
git log --pretty=format:'%s' | sed -nE 's/^[a-z]+\(([^)]+)\).*/\1/p' | sort | uniq -c | sort -rn
```

## Corps du message

Optionnel pour un changement trivial, **attendu dès que le « pourquoi » n'est pas évident**.

- En français, lignes coupées à ~75 caractères.
- Expliquer la **cause** et la **décision**, pas la liste des lignes modifiées (le diff s'en charge).
- Pour un commit qui touche plusieurs fichiers, une liste `- fichier : ce qui change et pourquoi`.

```
fix(cache) : borner la taille du cache en mémoire

Le service était tué par l'OOM killer après quelques heures de charge :
aucune limite sur le cache, qui grossissait à chaque clé distincte.

Cause : l'éviction n'était déclenchée que par expiration des entrées,
jamais par la pression mémoire.

- cache.py : capacité maximale et éviction LRU
- config.yaml : `cache_max_entries`, valeur par défaut 10 000
```

## Règles de travail

- **Ne jamais commiter ni pousser sans demande explicite** de l'utilisateur.
- Terminer par le trailer `Co-Authored-By:` habituel quand le commit est écrit par Claude.
- Merge de PR : le sujet auto (`Merge pull request #N from <org>/<branche>`) est conservé,
  avec en corps une phrase récapitulative du lot.
