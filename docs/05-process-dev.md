# Process de développement

## Cycle d'une tâche

1. **Choisir.** Prendre la prochaine tâche `à faire` de `06-roadmap.md` et la passer `en cours`.
2. **Isoler.** Une branche ou un worktree Orca par tâche.
3. **Lire.** La doc du domaine, selon le tableau d'`AGENTS.md`.
4. **Faire.** Le plus petit changement qui répond à la tâche, avec son test.
5. **Vérifier.** Commandes ci-dessous, puis voir le résultat tourner.
6. **Relire.** `code-review`, puis `simplify` ou `ponytail-review`.
7. **Fermer.** Roadmap et docs à jour dans le même commit, puis PR.

Un **spike** répond à une question et ne part pas en production. Il se termine par une décision dans `decisions.md`, et son code jetable reste sur sa branche.

## Vérifications

Ces commandes s'appliquent dès que le code existe.

```bash
# server/
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test

# app/
npx tsc --noEmit
npx expo lint
npx expo-doctor
```

## Quoi tester

Peu de tests, mais sur ce qui fait mal quand ça casse :

| Sujet | Test |
| --- | --- |
| Détection de pluie | Séries fixes : sec, averse d'un pas, début, fin, pluie sans fin |
| Fraîcheur | Run trop vieux : `unknown` ou repli, jamais `dry` |
| Alertes | Une seule par épisode, heures calmes, nouvel épisode après `DRY_RESET_MIN` |
| Décodage | Un vrai fichier PIAF en fixture, valeur connue à un point connu |
| Arrondi client | `rain_starts_at` et `now` : arrondi vers le haut |

Sur le terrain : tenir un carnet « annoncé / observé » pendant les premiers jours de pluie, pour calibrer `RAIN_THRESHOLD_MM_H`.

Les widgets et le push ne se testent bien que sur de vrais téléphones. Les simulateurs ne reçoivent pas les mêmes contraintes de fond.

## Orca

Orca a besoin d'un dépôt git : faire `git init` et un premier commit avant d'utiliser les worktrees.

```text
# Tâche indépendante, confiée à un agent dans son propre worktree
orca worktree create --name <tache> --no-parent --agent claude --prompt "<brief>" --json

# Nouvel agent dans un worktree existant
orca agent launch --agent codex --worktree id:<repoId>::<worktreePath> --prompt "<brief>" --json
```

- **Passage de relais sans retour attendu :** `orca-cli`.
- **Résultat attendu en retour, par exemple une revue par Codex :** `orchestration`.
- Le brief cite la ligne de roadmap, la doc à lire et le critère de fin.
- Bons candidats au parallèle : serveur et app, une fois le contrat d'API figé. Deux agents ne touchent pas le même fichier de doc en même temps.

## Build et release

| Étape | Outil |
| --- | --- |
| Dev Android | Build de développement, téléphone ou émulateur. Sous Windows, le natif se construit hors du dépôt, voir ci-dessous |
| Dev iOS | Mac et Xcode. Expo Go ne convient pas à cause des widgets |
| Bêta | TestFlight, test interne Play |
| Serveur | Image Docker, déployée sur demande explicite |

Avant la première soumission :

- [ ] Politique de confidentialité : quelles données, combien de temps, comment effacer
- [ ] Attribution Météo-France et Open-Meteo : écran principal, widgets, À propos
- [ ] Licence PIAF relue : elle peut changer sans préavis
- [ ] Clés APNs et FCM en variables d'environnement
- [ ] Justification de la localisation, premier plan seulement
- [ ] `security-review` passé

### Build Android sous Windows

Le chemin du dépôt est trop long pour CMake, et un lecteur `subst` échoue car Node résout le vrai chemin. Copier `app/` vers `C:\iir\app`, puis y lancer `expo prebuild -p android` et `gradlew assembleDebug -PreactNativeArchitectures=x86_64`. À refaire seulement si une dépendance native change : le JS vient de Metro, lancé depuis le dépôt.
Arrêter ensuite le daemon avec `gradlew --stop`, il garde plusieurs Go de RAM.

Pour un téléphone, sans Metro : mettre `EXPO_PUBLIC_API_URL=https://rain.alanmlcrt.fr` dans le `.env` de la copie, lancer `gradlew assembleRelease "-PreactNativeArchitectures=arm64-v8a,x86_64" --no-daemon`, puis `adb install -r` sur `app-release.apk`. Compter 11 min à froid. L'URL est figée dans le paquet. La signature reste celle de debug : à remplacer avant les stores.

## Quand une règle gêne

La changer dans `AGENTS.md` par une PR, avec sa raison. Ne pas la contourner en silence.
