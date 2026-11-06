# Architecture

## Vue d'ensemble

```text
Météo-France PIAF ──(toutes les 5 min)──┐
Open-Meteo (repli, à la demande) ───────┤
                                        ▼
                        ┌──────────────────────────────┐
                        │ Serveur Rust, un seul binaire│
                        │ ingest → grille en mémoire   │
                        │ API HTTP   /v1/rain          │
                        │ alertes → APNs / FCM         │
                        │ SQLite : abonnements         │
                        └───────┬───────────────┬──────┘
                       HTTPS    │               │  push
              ┌─────────────────┴───┐      ┌────┴─────────┐
              │ App Expo            │      │ Notification │
              │ Widget Swift/Kotlin │      │ système      │
              └─────────────────────┘      └──────────────┘
```

**Pourquoi un serveur.** Trois raisons : iOS ne garantit pas de réveiller une app à temps pour alerter, PIAF demande une clé qui ne peut pas être dans l'app, et une grille partagée rend le coût indépendant du nombre d'utilisateurs.

**Où est le Rust.** Sur le serveur, pas dans l'app. Voir `decisions.md`, D2.

## Serveur

Une seule crate, modules dans `src/` :

| Module | Rôle |
| --- | --- |
| `ingest` | Télécharge chaque run PIAF, le décode, remplace la grille en mémoire d'un coup |
| `forecast` | Pour un point : série de pluie, début et fin, état |
| `fallback` | Repli Open-Meteo, en cache par zone |
| `api` | Routes HTTP |
| `alerts` | Après chaque run, décide qui notifier |
| `push` | Envoi APNs et FCM |

La grille vit en mémoire et se reconstruit au prochain run après un redémarrage. Échantillonnage : `ligne = (51,5 - lat) / 0,01`, `colonne = (lon + 6) / 0,01`. La base ne stocke que les abonnements.
Une requête PIAF ne renvoie qu'une échéance : 39 appels par run, espacés de 700 ms pour rester sous 100 req/min, soit environ 30 s par run. Sur `429` ou `5xx`, attendre `Retry-After` et réessayer jusqu'à 3 fois. Une erreur restante abandonne le run et garde la grille précédente tant qu'elle est fraîche. Chaque run n'a qu'un essai : le pire cas de volume reste un run toutes les 5 min, et le run suivant prend le relais.
Le dernier run vient de `GetCapabilities` (58 Ko en gzip), interrogé toutes les 30 s à partir de 14 min après le run en place. Le journal donne pour chaque run le volume cumulé, le nombre de requêtes, l'âge à l'arrivée et les runs manqués.
Les valeurs `PT5M` sont des cumuls en mm : les multiplier par 12 pour obtenir des mm/h. L'échéance `300` couvre `[run, run + 5 min]`, et la valeur est étiquetée en fin de pas. En `f32`, une grille de 39 échéances occupe environ 270 Mo, le double pendant le remplacement.
La géométrie se lit dans chaque fichier GRIB, car le WCS recale la zone demandée sur la grille. Tout fichier dont le run, l'échéance ou la grille ne correspondent pas à la demande fait échouer le run.
Le jeton est géré par le serveur : voir « Authentification Météo-France ».

Le repli prend la main quand PIAF répond `unknown` : hors zone, ou run périmé. Il demande à Open-Meteo 2 pas passés et 13 à venir, soit 15 pas de 15 min. Les valeurs sont des cumuls sur les 15 minutes précédentes : les multiplier par 4 pour obtenir des mm/h.
Le cache garde 15 min une réponse par zone de 0,1°, environ 11 km. Open-Meteo reçoit le centre de la zone, jamais la position demandée. Le serveur s'arrête à 9 000 appels par jour, sous la limite de 10 000 de D1. Open-Meteo muet, réponse trouée ou quota atteint : la réponse reste `unknown`. Ne jamais journaliser une erreur `reqwest` : son texte contient l'URL, donc la zone.
Hors d'Europe centrale et d'Amérique du Nord, Open-Meteo interpole des données horaires : l'affichage approché `~` couvre ce cas.
`MIN_WET_STEPS` compte en pas de 5 min : à 15 min, un seul pas suffit à faire un épisode.

`ponytail:` une seule zone fixe (D9). Plusieurs lieux : zones à la demande, limitées à environ 12 par les 39 requêtes de chaque run.
Toutes les réponses portent `source` et `run_at`, car la licence impose de citer la source et la date de mise à jour.

### Paramètres à calibrer

Ce sont de vraies variables d'environnement : le ressenti de la pluie se règle sur le terrain.

| Variable | Départ | Rôle |
| --- | --- | --- |
| `RAIN_THRESHOLD_MM_H` | 0,1 | À partir de quand on dit « il pleut ». PIAF ne descend pas sous 0,12 mm/h : au départ, toute valeur non nulle compte |
| `MIN_WET_STEPS` | 2 | Pas consécutifs requis : ignore une averse d'un seul pas |
| `MAX_DATA_AGE_MIN` | 20 | Au-delà : repli ou `unknown` |
| `DRY_RESET_MIN` | 30 | Temps sec qui clôt un épisode et autorise une nouvelle alerte |

Trois autres réglent l'exécution :

| Variable | Rôle |
| --- | --- |
| `PIAF_BBOX` | Obligatoire : `lon_min,lon_max,lat_min,lat_max`. Voir D9 |
| `PIAF_MAX_RUNS` | Développement : arrête l'ingestion après N runs. L'API continue et passe en `unknown` |
| `BIND_ADDR` | `127.0.0.1:8080` par défaut |

En local, depuis `server/` sous Git Bash : `set -a; . ../.env; set +a; cargo run`.

## Authentification Météo-France

L'app ne contient aucune clé et n'appelle jamais Météo-France. Elle parle à notre serveur, qui seul détient l'identifiant. Mettre une clé dans l'app la rendrait extractible, et la licence l'interdit.

| Environnement | Où vit le secret |
| --- | --- |
| Développement | `.env`, ignoré par git |
| Production | Variable d'environnement chiffrée dans Dokploy |

`METEOFRANCE_APP_ID` est la valeur `Basic` de la commande `curl` fournie par le portail avec le jeton OAuth2. C'est le secret durable, le jeton en découle.

```text
POST https://portail-api.meteofrance.fr/token
Authorization: Basic <METEOFRANCE_APP_ID>
→ { "access_token": "…" }, valable 1 heure
```

Testé le 5 novembre 2026 : `/token` répond `200` avec `expires_in: 3600`, et le jeton obtenu lit PIAF. Deux secondes après un second appel, l'ancien jeton fonctionnait encore. Le client [Meteole](https://github.com/MAIF/meteole) de la MAIF dit au contraire qu'un nouveau jeton invalide le précédent : on garde donc la prudence ci-dessous.

- Garder le jeton en mémoire. Le renouveler 5 min avant expiration, et aussi sur `401`.
- Un seul renouvellement à la fois : une seule tâche possède le client PIAF, sans verrou.
- Une application Météo-France par environnement, pour que le développement ne coupe pas la production.
- Ne jamais journaliser le jeton ni l'identifiant.

Non vérifié : la durée de vie de l'Application ID lui-même. Alternative : le type « API Key » du portail, envoyé dans l'en-tête `apikey`. Elle a une durée de validité choisie à la génération, donc une date d'expiration à surveiller, et elle n'a pas encore été testée.

## Lieu et recherche de ville

| Besoin | Solution |
| --- | --- |
| Position actuelle | `expo-location`, premier plan |
| Nom du lieu actuel | `reverseGeocodeAsync` d'`expo-location` |
| Recherche de ville | Appel direct depuis l'app à `https://geocoding-api.open-meteo.com/v1/search?name=…&language=fr&count=5`, sans clé |

Testé le 5 novembre 2026 : Open-Meteo trouve Saint-Malo dès `saint mal`, sur données GeoNames. La Géoplateforme de l'IGN (`data.geopf.fr/geocodage/search`, `type=municipality`) ne renvoie rien sur ce mot incomplet mais trouve `saint malo` : elle sert de repli pour la France.
Sélection d'une ville : garder ses coordonnées, puis appeler `/v1/rain`. Attendre 300 ms après la frappe et partir de 2 caractères. Une ville hors zone PIAF passe sur le repli Open-Meteo, avec sa précision de 15 min.

## Contrat d'API

### `GET /v1/rain?lat=48.86&lon=2.35`

```json
{
  "state": "rain_soon",
  "source": "piaf",
  "step_minutes": 5,
  "run_at": "2026-11-05T14:30:00Z",
  "rain_starts_at": "2026-11-05T15:05:00Z",
  "rain_ends_at": "2026-11-05T15:50:00Z",
  "horizon_ends_at": "2026-11-05T17:30:00Z",
  "series": [{ "t": "2026-11-05T14:45:00Z", "mm_h": 0.0 }]
}
```

Le client calcule `rain_starts_at - now`, puis arrondit vers le haut selon `step_minutes`. Ainsi le compte à rebours reste juste entre deux appels, sans logique de pluie côté client.
Coordonnées invalides (hors de ±90° et ±180°, absentes ou illisibles) : `400`. Réponse en cache HTTP jusqu'au prochain run attendu, sans dépasser la fraîcheur de la donnée.

| Champ | Règle |
| --- | --- |
| `series` | Tous les pas de la source, passés compris : 39 pour PIAF, 15 pour Open-Meteo. `t` est la fin du pas. Vide en `unknown` |
| `source` | `piaf` ou `open_meteo` |
| `run_at` | Heure du run PIAF. Pour Open-Meteo, qui ne la donne pas : heure de notre appel |
| `rain_starts_at` | Début du premier pas de l'épisode à venir ou en cours, donc déjà passé en `raining` |
| `rain_ends_at` | `null` si la pluie dure jusqu'à `horizon_ends_at` |
| `horizon_ends_at` | Fin du dernier pas : `run_at` + 195 min pour PIAF |

`unknown` : `series` vide, `step_minutes` et dates de pluie à `null`, `Cache-Control: no-store`. Run périmé : `source` et `run_at` restent ceux du dernier run. Aucun run ou point hors zone : ils valent `null`. Ces cas passent d'abord par le repli, `unknown` veut dire qu'il a échoué aussi.
Un épisode compte au moins `MIN_WET_STEPS` pas sur toute la série, passé compris : la fin d'une averse en cours reste `raining`, même à un seul pas de la fin. Un pas sec coupe l'épisode.

### `PUT /v1/subscriptions/{install_id}`

```json
{
  "platform": "ios",
  "push_token": "…",
  "lat": 48.86,
  "lon": 2.35,
  "lead_minutes": 15,
  "quiet_start": "22:00",
  "quiet_end": "07:00",
  "timezone": "Europe/Paris"
}
```

`install_id` est un UUID généré à l'installation, sans compte. `DELETE` sur la même route efface tout.
Valider chaque champ à l'entrée. Limiter le débit au niveau du proxy.

## Alertes

Après chaque run, pour chaque abonnement :

1. Calculer le début de pluie au point suivi.
2. Si `rain_starts_at - now <= lead_minutes`, hors heures calmes, et pas d'alerte pour cet épisode : envoyer et enregistrer.
3. Un épisode se termine après `DRY_RESET_MIN` de temps sec.

Si la donnée est périmée, ne rien envoyer. Un jeton refusé par APNs ou FCM supprime l'abonnement.
Le lieu suivi est le lieu actif : la dernière position vue quand l'app est ouverte, ou la ville choisie. Il n'y a pas de suivi en arrière-plan, donc pas de permission « Toujours ».

## Widgets

Un widget tourne sans le runtime JS de l'app. Il doit donc pouvoir appeler `/v1/rain` seul.

| | iOS | Android |
| --- | --- | --- |
| Rendu | SwiftUI, WidgetKit | Selon S3 : `expo-widgets`, `react-native-android-widget` ou Glance |
| Données | Appel direct dans le provider | Tâche de fond, plus message FCM si vérifié |
| Compte à rebours | Une entrée de timeline par pas de 5 min, à partir de la série déjà reçue : pas de réseau | Recalcul à chaque rafraîchissement |
| Rafraîchissement | Timeline, push widget en bonus sur iOS 26+ | 15 min au mieux, plus pendant Doze |
| Fraîcheur | Afficher `unknown` à l'expiration de la dernière entrée | Afficher l'heure de mise à jour |

La limite d'Android est réelle : le widget peut retarder de 15 min ou plus. La notification reste le canal fiable, le widget un coup d'œil.
Position du widget : dernière position partagée par l'app via App Group sur iOS et stockage partagé sur Android.

## Hébergement

Un conteneur Docker sur l'infrastructure Dokploy existante. Un serveur unique suffit : un redémarrage coûte au plus un run. Le volume pour SQLite arrivera avec N1.

GitHub Actions construit `server/Dockerfile` à chaque push sur `main` et publie `ghcr.io/alanmlcrt/is-it-raining-server`. La machine Dokploy (1 CPU, 4 Go, partagée avec d'autres services) ne compile rien : elle récupère l'image et sert le port 8080 en HTTPS. On y déploie sur demande explicite.

| Variable de production | Valeur |
| --- | --- |
| `METEOFRANCE_APP_ID` | Secret d'une application Météo-France dédiée à la production, saisi dans Dokploy |
| `PIAF_BBOX` | `2.2,2.5,48.8,48.95` |
| `PIAF_MAX_RUNS` | Absente, sinon l'ingestion s'arrête |

`BIND_ADDR` vaut `0.0.0.0:8080` dans l'image.

`ponytail:` SQLite et instance unique. Passer à Postgres et à plusieurs instances seulement si la boucle d'alertes ne tient plus dans les 5 min d'un run.
