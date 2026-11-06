# Roadmap

À tenir à jour à chaque tâche. Statuts : `à faire`, `en cours`, `bloqué`, `fini`.

## M0 : lever les inconnues

Rien d'autre ne démarre avant S1 et S2.

| ID | Tâche | Critère de fin | Statut |
| --- | --- | --- | --- |
| S0 | `git init`, installer `rustup` | Un programme Rust compile, premier commit fait | fini |
| S1 | Créer un compte sur le portail Météo-France et lire les conditions de PIAF | Licence, gratuité et usage commercial écrits dans D1. Si bloqué : AROME-PI, puis Open-Meteo | fini : Licence ouverte 2.0, voir D1 |
| S2 | Télécharger un run PIAF et le lire en Rust | Valeur lue à un point connu, comparée à une référence. Taille, requêtes par run et latence notées | fini : GRIB lu en Rust, identique au GeoTIFF |
| S5 | Obtenir un accès serveur durable à PIAF | Le jeton se renouvelle sans action manuelle | fini : nouveau jeton obtenu avec `METEOFRANCE_APP_ID` et PIAF lu. Renouvellement à coder dans A1 |
| S3 | Widget de test sur vrais téléphones, sans ouvrir l'app | Android tranché. Délai réel de rafraîchissement mesuré sur 24 h | à faire |
| S4 | Choisir la bibliothèque UI | D4 passée de « proposée » à « décidée » | fini |

## M1 : serveur

| ID | Tâche | Critère de fin | Statut |
| --- | --- | --- | --- |
| A1 | Ingestion toutes les 5 min, grille en mémoire | Tourne 24 h sans trou non détecté | en cours : jeton renouvelé seul à 55 min sur l'API réelle. Veille du PC la nuit du 5 novembre : trou de 114 runs détecté, reprise seule. 24 h relancées sur Dokploy le 6 novembre à 13 h 32 UTC, à relire le 7 |
| A2 | `GET /v1/rain`, états et fraîcheur | Tests de `05-process-dev.md` verts | fini : tests verts, réponse réelle `raining` puis `unknown` à 20 min |
| A3 | Repli Open-Meteo | Hors France ou PIAF en panne : `source` et `step_minutes` corrects | en cours : code et tests faits, réponse réelle vue en local. Reste à déployer après les 24 h de A1 |
| A4 | Déploiement Docker | URL HTTPS publique qui répond | fini : `https://rain.alanmlcrt.fr/v1/rain` répond `dry` avec le run de 13 h 20 UTC le 6 novembre |

## M2 : app

| ID | Tâche | Critère de fin | Statut |
| --- | --- | --- | --- |
| B1 | Créer le projet Expo, tokens et polices | Build de développement sur téléphone | en cours : tourne sur émulateur Android, pas encore sur téléphone |
| B2 | Écran principal et bande de pluie | Quatre états, plus hors ligne. Checklist design | en cours : états vus sur émulateur, clair et sombre. Reste la checklist : iOS, lecteur d'écran, animations réduites, `unslop` |
| B3 | Position actuelle et recherche de ville | Refus de localisation géré : recherche proposée. Ville trouvée dès 3 lettres | en cours |
| B4 | Réglages et À propos | Attribution visible | à faire |

## M3 : widgets

| ID | Tâche | Critère de fin | Statut |
| --- | --- | --- | --- |
| C1 | Widget iOS, timeline à partir de la série | Compte à rebours juste sans ouvrir l'app | à faire |
| C2 | Widget Android | Selon S3 | à faire |

## M4 : notifications

| ID | Tâche | Critère de fin | Statut |
| --- | --- | --- | --- |
| N1 | Abonnements : routes et SQLite | Création, modification, effacement | à faire |
| N2 | Boucle d'alertes | Tests d'épisodes verts | à faire |
| N3 | Envoi APNs et FCM, jetons invalides | Notification reçue sur les deux plateformes | à faire |
| N4 | Calibration | Une semaine de carnet annoncé / observé, seuils ajustés | à faire |

## M5 : sortie

| ID | Tâche | Critère de fin | Statut |
| --- | --- | --- | --- |
| E1 | Choisir le vrai nom et l'icône | Nom libre sur les stores | à faire |
| E2 | Modèle économique | Décidé. Licences de données compatibles | fini (gratuite, D6). Licence PIAF liée à S1 |
| E3 | Bêta privée | Dix personnes, deux semaines | à faire |
| E4 | Soumission aux stores | Checklist de `05-process-dev.md` passée | à faire |

## Plus tard, peut-être

Extension Nordiques (MET Norway), Allemagne (DWD RADVOR), extrapolation radar maison, Live Activity pendant l'averse.
