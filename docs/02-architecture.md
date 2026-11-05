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
| `forecast` | Pour un point : série de 36 pas, début et fin de pluie, état |
| `api` | Routes HTTP |
| `alerts` | Après chaque run, décide qui notifier |
| `push` | Envoi APNs et FCM |

La grille vit en mémoire et se reconstruit au prochain run après un redémarrage. La base ne stocke que les abonnements.
Une requête PIAF ne renvoie qu'une échéance : 36 appels par run, étalés pour rester sous 100 req/min. Sur `429`, attendre et garder la grille précédente tant qu'elle est fraîche. La taille de la grille est à mesurer au spike S2.
Toutes les réponses portent `source` et `run_at`, car la licence impose de citer la source et la date de mise à jour.

### Paramètres à calibrer

Ce sont de vraies variables d'environnement : le ressenti de la pluie se règle sur le terrain.

| Variable | Départ | Rôle |
| --- | --- | --- |
| `RAIN_THRESHOLD_MM_H` | 0,1 | À partir de quand on dit « il pleut » |
| `MIN_WET_STEPS` | 2 | Pas consécutifs requis : ignore une averse d'un seul pas |
| `MAX_DATA_AGE_MIN` | 20 | Au-delà : repli ou `unknown` |
| `DRY_RESET_MIN` | 30 | Temps sec qui clôt un épisode et autorise une nouvelle alerte |

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
Coordonnées hors bornes : `400`. Réponse en cache HTTP jusqu'au prochain run attendu.

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
Le lieu suivi est la dernière position vue quand l'app est ouverte, ou un lieu fixé dans les réglages. Il n'y a pas de suivi en arrière-plan, donc pas de permission « Toujours ».

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

Un conteneur Docker, un volume pour SQLite, sur l'infrastructure Dokploy existante. Un serveur unique suffit : un redémarrage coûte au plus un run.

`ponytail:` SQLite et instance unique. Passer à Postgres et à plusieurs instances seulement si la boucle d'alertes ne tient plus dans les 5 min d'un run.
