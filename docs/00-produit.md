# Produit

## Promesse

Ouvrir l'app, regarder le widget ou recevoir une notification, et savoir en une seconde quand la pluie arrive.

## Périmètre v1

- **Zone :** France métropolitaine en précision 5 min. Ailleurs en Europe centrale et de l'Ouest, précision 15 min si le repli Open-Meteo est gardé.
- **Lieu :** la position actuelle par défaut, ou une ville cherchée. Un seul lieu actif à la fois, repris par le widget et les alertes.
- **Écran principal :** la réponse, puis la bande de pluie sur 3 h.
- **Réglages :** délai d'alerte, heures calmes, À propos.
- **Widgets :** petit format iOS et Android. Écran verrouillé iOS si le spike S3 le permet.
- **Notification :** une alerte par épisode de pluie, sur le lieu actif.

## Les quatre états

| État | Condition | Texte | Widget |
| --- | --- | --- | --- |
| `dry` | Aucune pluie dans l'horizon | Pas de pluie avant 17 h 40 | 17 h 40 |
| `rain_soon` | Pluie prévue dans l'horizon | Pluie dans 20 min | 20 min |
| `raining` | Pluie en cours | Il pleut. Fin vers 16 h 15 | 16 h 15 |
| `unknown` | Donnée absente ou trop vieille | Prévision indisponible | – |

Si la pluie ne s'arrête pas dans l'horizon, on affiche « Il pleut pour au moins 3 h ».

## Ce qu'on ne fait pas en v1

Prévisions à plusieurs jours, température, carte radar, comptes utilisateurs, localisation en arrière-plan, plusieurs lieux suivis, publicité, abonnement.

La température et les prévisions à plusieurs jours sont exclues pour de bon : c'est ce qui rend l'app simple.

## Modèle économique

**Gratuite, sans publicité ni abonnement** (D6). Cela garde l'accès gratuit à Open-Meteo, qui traite pub et abonnement comme un usage commercial. Un pourboire volontaire resterait autorisé.

## Questions ouvertes

Elles sont aussi suivies dans `06-roadmap.md`.

1. Quel seuil de pluie colle au ressenti ? Départ : 0,1 mm/h, à calibrer sur le terrain.
2. Quand prendre le compte Apple Developer ? Il est nécessaire pour le push, TestFlight et l'App Store.
