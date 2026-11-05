# Décisions

Journal en ajout seul. Pour changer d'avis, ajouter une décision qui remplace l'ancienne.
Statuts : `proposée`, `décidée`, `remplacée par Dn`.

## D1 : PIAF en source principale, Open-Meteo en repli

**Décidée, 2026-11-05.** Licence lue au spike S1.

- **Contexte.** Il faut un pas fin et une mise à jour fréquente. PIAF offre 5 min et un run toutes les 5 min. Open-Meteo offre 15 min et une mise à jour horaire.
- **Décision.** PIAF pour la France métropolitaine. Open-Meteo hors zone et en cas de panne.
- **Licence.** Licence ouverte 2.0, gratuite, usage commercial autorisé. La mention `commerciales` du portail n'implique pas de paiement.
- **Risque.** Aucune garantie de disponibilité, quotas modifiables. Le repli Open-Meteo reste donc utile en France aussi.
- **Conséquence.** Compatible avec D6 tant que le serveur reste sous 10 000 appels Open-Meteo par jour : mettre en cache par zone.

## D2 : Rust sur le serveur, pas dans l'app

**Décidée, 2026-11-05.** Critère donné : ce qui est le plus performant pour l'app.

- **Contexte.** La demande est une app en Rust et React Native. Un serveur est de toute façon nécessaire pour les notifications et pour protéger la clé.
- **Décision.** Toute la logique météo est en Rust sur le serveur. L'app React Native affiche une réponse déjà calculée.
- **Rejeté.** Rust embarqué via `uniffi-bindgen-react-native` : encore pré-1.0, et il ajouterait une compilation croisée iOS et Android, alors que le client n'a qu'une soustraction de dates à faire. Les widgets natifs ne pourraient pas non plus l'utiliser simplement.
- **Performance.** L'app fait un appel HTTP et affiche une petite réponse. Le temps perçu vient du réseau et du rendu, pas d'un calcul : Rust embarqué ne l'accélérerait pas. Le téléphone évite de télécharger et de décoder les grilles, ce qui épargne batterie et données. Raisonnement non mesuré, à confirmer avec un profil sur device.
- **À revoir si.** Une logique lourde doit tourner hors ligne sur le téléphone.

## D3 : widgets natifs, pas de rendu React Native

**Proposée, 2026-11-05. Android tranché au spike S3.**

- **Contexte.** Un widget tourne sans le JS de l'app et doit se rafraîchir app fermée.
- **Décision.** SwiftUI via `@bacons/apple-targets` sur iOS. Android selon S3.
- **Rejeté pour l'instant.** `expo-widgets` comme solution unique : pas de timeline sur Android, composants limités, et on n'a pas confirmé qu'il se rafraîchit sans l'app.

## D4 : HeroUI Native + Uniwind

**Décidée, 2026-11-05.**

- **Décision.** HeroUI Native pour les réglages. Écran principal sur mesure.
- **Alternative.** React Native Paper, plus éprouvée mais Material.
- **Détail.** Comparatif dans `03-stack.md`.

## D5 : pas de localisation en arrière-plan

**Proposée, 2026-11-05.**

- **Décision.** Alerte sur la dernière position vue dans l'app, ou sur la ville choisie.
- **Pourquoi.** Permission « Toujours » intrusive, vérification plus dure sur les stores, positions à stocker.
- **Coût assumé.** Si tu pars loin sans ouvrir l'app, l'alerte concerne l'ancien lieu. L'écran des réglages le dit clairement.

## D6 : app gratuite

**Décidée, 2026-11-05.**

- **Décision.** Pas de publicité, pas d'abonnement.
- **Conséquence.** Usage non commercial au sens d'Open-Meteo. Le serveur et le compte Apple restent à notre charge.
- **À revoir si.** Un don ou un revenu apparaît : relire les conditions de chaque source.

## D7 : un seul lieu actif

**Décidée, 2026-11-05.**

- **Décision.** Position actuelle par défaut, ou ville cherchée. Le widget et les alertes suivent ce lieu. Pas de liste de favoris.
- **Recherche.** Open-Meteo Geocoding, directement depuis l'app.

## D8 : le secret Météo-France reste sur le serveur

**Décidée, 2026-11-05.**

- **Décision.** `METEOFRANCE_APP_ID` en variable d'environnement. Le serveur crée et renouvelle seul un jeton d'une heure.
- **Pourquoi.** La licence interdit de divulguer la clé, et une app publiée est décompilable.
