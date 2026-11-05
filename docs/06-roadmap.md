# Roadmap

À tenir à jour à chaque tâche. Statuts : `à faire`, `en cours`, `bloqué`, `fini`.

## M0 : lever les inconnues

Rien d'autre ne démarre avant S1 et S2.

| ID | Tâche | Critère de fin | Statut |
| --- | --- | --- | --- |
| S0 | `git init`, installer `rustup` | Un programme Rust compile, premier commit fait | en cours : git et rustup OK, linker MSVC manquant, premier commit à faire |
| S1 | Créer un compte sur le portail Météo-France et lire les conditions de PIAF | Licence, gratuité et usage commercial écrits dans D1. Si bloqué : AROME-PI, puis Open-Meteo | fini : Licence ouverte 2.0, voir D1 |
| S2 | Télécharger un run PIAF et le lire en Rust | Valeur lue à un point connu, comparée à l'image WMS. Taille, requêtes par run et latence notées | à faire |
| S3 | Widget de test sur vrais téléphones, sans ouvrir l'app | Android tranché. Délai réel de rafraîchissement mesuré sur 24 h | à faire |
| S4 | Choisir la bibliothèque UI | D4 passée de « proposée » à « décidée » | fini |

## M1 : serveur

| ID | Tâche | Critère de fin | Statut |
| --- | --- | --- | --- |
| A1 | Ingestion toutes les 5 min, grille en mémoire | Tourne 24 h sans trou non détecté | à faire |
| A2 | `GET /v1/rain`, états et fraîcheur | Tests de `05-process-dev.md` verts | à faire |
| A3 | Repli Open-Meteo | Hors France ou PIAF en panne : `source` et `step_minutes` corrects | à faire |
| A4 | Déploiement Docker | URL HTTPS publique qui répond | à faire |

## M2 : app

| ID | Tâche | Critère de fin | Statut |
| --- | --- | --- | --- |
| B1 | Créer le projet Expo, tokens et polices | Build de développement sur téléphone | à faire |
| B2 | Écran principal et bande de pluie | Quatre états, plus hors ligne. Checklist design | à faire |
| B3 | Localisation et permission | Refus géré : choix manuel d'un lieu | à faire |
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
