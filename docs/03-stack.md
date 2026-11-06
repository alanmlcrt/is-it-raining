# Stack

Versions et chiffres relevés sur npm, crates.io et GitHub le 5 novembre 2026. Vérifier la version au moment de l'installation.

## Retenu

| Rôle | Choix | Version | Remarque |
| --- | --- | --- | --- |
| App | Expo + React Native | 58.0 / 0.88 | Nouvelle architecture obligatoire depuis SDK 55 |
| Navigation | Expo Router | Livré avec Expo | Deux écrans, sans plus |
| UI | **HeroUI Native** + Uniwind | 1.0 / 1.13 | Réglages seulement, écran principal sur mesure (D4) |
| Icônes | `lucide-react-native` | 1.69 | Une seule famille |
| Police | `@expo-google-fonts/geist` + `expo-font` | 0.4 / 58.0 | Geist, voir `04-design.md` |
| Mode sombre Android | `expo-system-ui` | Selon SDK | Requis pour `userInterfaceStyle: automatic` |
| Build de développement | `expo-dev-client` | 58.0 | Expo Go ne convient pas à cause des widgets |
| Tests de l'app | `node --test` | Node 22 | Sur `src/rain.ts`, sans Jest : pas de dépendance à ajouter |
| Données client | `fetch` + état React | | TanStack Query seulement si le cache devient pénible |
| Bande de pluie | `View` natives | | Une barre par pas : pas de bibliothèque de graphiques |
| Notifications | `expo-notifications` | Selon SDK | Jeton natif, envoi par notre serveur |
| Localisation | `expo-location` | Selon SDK | Premier plan seulement |
| Serveur HTTP | `axum` + `tokio` | 0.8 / 1.54 | Très utilisés |
| Client HTTP | `reqwest` | 0.13 | `gzip` : `GetCapabilities` passe d'environ 2 Mo à 58 Ko à chaque cycle |
| Dates | `chrono` | 0.4 | Lire les `CoverageId`, écrire les dates RFC 3339 de l'API |
| JSON | `serde` | 1 | Réponses de l'API et jeton Météo-France |
| Base | `rusqlite` | 0.41 | Plus simple que `sqlx` pour une table ou deux |
| Décodage météo | GRIB via `grib` | 0.20 | `default-features = false` : pas de dépendance C. 4 fois plus léger que le GeoTIFF. Décodage France : 71 ms |
| Push iOS | `a2` | 0.10 | Environ 67 000 téléchargements récents. Sinon appel HTTP/2 direct |
| Push Android | FCM HTTP v1 via `reqwest` | | Pas de crate nécessaire |

Écarté : `uniffi-bindgen-react-native` (0.31 pre-release). Maintenu et sérieux, mais encore pré-1.0, et sans utilité ici puisque la logique reste sur le serveur.

## Bibliothèques UI

| Bibliothèque | Étoiles GitHub | Téléchargements/mois | Licence | Force | Limite pour nous |
| --- | --- | --- | --- | --- | --- |
| **HeroUI Native** 1.0.11 | 3 700 | 1,09 M | Apache-2.0 | Moderne, sobre, thème par tokens, compatible Expo 58 et RN 0.88 | Stable seulement depuis mars 2026 |
| **React Native Paper** 5.15 | 14 500 | 1,88 M | MIT | La plus éprouvée, très stable, Callstack derrière | Material Design : aspect Android sur iPhone, difficile à personnaliser à fond |
| Tamagui 3.2 | 14 300 | 910 000 | MIT | Performant, universel web et natif | Compilateur et configuration trop lourds pour trois écrans |
| gluestack-ui 5 | 5 300 | 454 000 (core) | Non détectée par GitHub | Composants copiés dans le projet | Dernier push du dépôt en septembre, licence à vérifier |
| React Native Reusables | 8 700 | CLI, pas de paquet | MIT | Façon shadcn : le code est chez nous | Plus de code à maintenir |

Styles : **Uniwind** 1.13 (1 700 étoiles, 4,1 M/mois) est imposé par HeroUI. **NativeWind** 4.2 (8,1 M/mois) reste le standard Tailwind.

### Décision

**HeroUI Native + Uniwind.** Le risque de jeunesse est faible ici : la bibliothèque ne sert que pour les réglages (liste, interrupteur, feuille, bouton). L'écran principal et les widgets sont faits sur mesure.

Repli si HeroUI pose problème : **React Native Paper**, avec une couche Material sur les réglages.

### Skill `heroui-native`

Installé dans le projet (`.agents/skills/`, version verrouillée dans `skills-lock.json`), pour Claude Code et Codex.

L'audit de skills.sh affiche Gen en échec, Snyk en alerte et Socket sans alerte, sans raison publiée. Relecture faite : les quatre scripts lisent la doc sur les domaines HeroUI, sans lancer de commande ni écrire de fichier. Le point sensible est le `curl | bash` de son `SKILL.md`, d'où les règles d'`AGENTS.md`.

## Widgets : options

| Option | Plateformes | État | Limite |
| --- | --- | --- | --- |
| `expo-widgets` 58.0 | iOS et Android | Alpha annoncée en mars 2026, statut actuel non indiqué dans la doc | Android : sous-ensemble de composants, pas de timeline. Mises à jour pilotées depuis l'app, à confirmer |
| `@bacons/apple-targets` 5.0 + SwiftUI | iOS | Stable, 2,8 M/mois | Du Swift à écrire, mais peu de lignes |
| `react-native-android-widget` 0.26 | Android | 900 étoiles, 400 000/mois | Comportement sans app ouverte à vérifier |
| Kotlin + Glance | Android | Natif | Plugin Expo à écrire |

Choix par défaut : **SwiftUI via apple-targets sur iOS.** Android tranché au spike S3.

## Prérequis sur ce poste

Sur le PC Windows : Node 22, Rust 1.99 via `rustup`, Visual Studio Build Tools 2022 (C++).

**Limite de build Android sous Windows.** Le `ninja` 1.10 du SDK ne gère pas les chemins longs, même avec `LongPathsEnabled=1`. `react-native-safe-area-context` dépasse 260 caractères, même depuis `C:\iir`. Il est donc retiré pour l'instant, avec des marges approchées dans `App.tsx`. HeroUI l'exige : à régler avant B4, avec un CMake récent dans le SDK ou un build EAS.
Sur le Mac : Xcode, pour le widget SwiftUI et les builds iOS.
Skill `heroui-native` sur une nouvelle machine : `npx skills add heroui-inc/heroui --skill heroui-native --agent claude-code codex`, car `.claude/skills/` contient un lien absolu et reste hors de git.
