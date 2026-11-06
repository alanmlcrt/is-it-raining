# Recherche

Faite le 5 novembre 2026. Les niveaux de confiance sont indiqués : **vérifié** veut dire lu dans une doc officielle, une spec ou mesuré. Le reste est marqué.

## Ce qu'il faut retenir

Répondre « dans 20 min » demande de la prévision immédiate fondée sur le radar, pas un modèle météo classique.
La meilleure source pour la France est **PIAF de Météo-France** : pas de 5 min, nouveau run toutes les 5 min, grille de 0,01°, licence ouverte.
Open-Meteo est le plan B le plus simple, mais avec un pas de 15 min et une mise à jour horaire seulement.

## Sources de données

| Source | Zone | Pas | Horizon | Mise à jour | Coût et licence | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| **Météo-France PIAF** | France métropole, 0,01° | 5 min | 195 min (mesuré) | 5 min, dispo 10 min après | Licence ouverte 2.0, gratuit, usage commercial autorisé, 100 req/min | **Source principale** |
| Météo-France AROME-PI | France | 15 min | 6 h | Horaire, dispo à H+30 | Licence ouverte 2.0, compte, 100 req/min | Plan B officiel, GRIB à traiter |
| Météo-France Données Radar | France et outre-mer | Observation 5 min | Passé seulement | 5 min | Licence ouverte 2.0 | Utile si on fait notre propre extrapolation. Pas en v1 |
| **Open-Meteo** `minutely_15` | Natif en Europe centrale et de l'Ouest et Amérique du Nord. Ailleurs : horaire interpolé | 15 min | 6 h (AROME 15 min) | Horaire pour AROME | CC BY 4.0. Gratuit si non commercial et sous 10 000 appels/jour | **Repli**, et prototype le plus rapide |
| MET Norway Nowcast 2.0 | Norvège, Suède, Finlande, Danemark | 5 min (tiers) | 2 h | 5 min | Gratuit, User-Agent identifié obligatoire | Hors zone. Candidat pour une extension nordique |
| DWD RADVOR | Allemagne | 5 min | 2 h | 5 min | Open data avec attribution | Fichiers RADOLAN binaires. Candidat pour l'Allemagne |
| Apple WeatherKit | Australie, Irlande, Japon, Royaume-Uni, États-Unis (page Apple du 5 juin 2026) | Minute | 1 h | Non publié | 500 000 appels/mois inclus dans le compte Apple Developer | Écarté : France absente de la liste |
| Rainbow Weather | Annoncée mondiale, France non testée | 1 min | 4 h | 10 min | 5 000 requêtes/mois gratuites, puis 0,10 $ pour 1 000. Clé dans une app : conditions non trouvées | Candidat à évaluer comme source serveur hors France |
| AccuWeather MinuteCast | Annoncée mondiale, France non testée | Minute | 2 h | Non vérifié | 50 appels/jour gratuits, puis 25 $/mois pour 10 000 | Écarté : quota et coût |
| OpenWeather One Call | Non précisée | Minute | 1 h | 10 min | 1 000 appels/jour gratuits | Écarté : modèle propriétaire, radar non mentionné |
| Google Weather | Mondiale | Heure | 240 h | Non vérifié | Non vérifié | Écarté : pas de prévision à la minute |
| RainViewer API | Monde | – | – | – | **Nowcast retiré le 1er janvier 2026** | Écarté |
| API privée de l'app Météo-France | France | Non vérifié | 1 h | Interrogée toutes les 5 min par la communauté | Non documentée, jeton extrait de l'app | **Exclu** : aucun droit, peut casser |
| Buienradar | Pays-Bas | Non vérifié | 2 h | Non vérifié | Commercial sur autorisation | Écarté |
| Pirate Weather | Hors Amérique du Nord : GFS horaire d'après sa doc, Europe non confirmée | Minute | 1 h | Non vérifié | Un rapport d'utilisateur signale un minutely pauvre, possiblement ancien | Écarté sans test |

Revu le 6 novembre 2026 : se passer du serveur a été écarté. Une clé commerciale dans l'app reste extractible, un widget seul consomme environ 2 900 appels par mois et par téléphone, et les notifications iOS exigent un serveur de toute façon.

### PIAF : licence et accès

**Réglé le 5 novembre 2026** avec la licence d'utilisation téléchargée depuis le portail.

- **Licence.** Droit « non-exclusif et gratuit », sous Licence ouverte 2.0. L'exploitation commerciale est autorisée.
- **Obligations.** Ne pas altérer ni dénaturer les données, mentionner Météo-France comme source et la date de dernière mise à jour, quel que soit le support, et tenir les données à jour.
- **Clé.** Clé d'authentification ou OAuth2. Elle ne doit jamais être publique ni partagée, ce qui impose le serveur. En cas de fuite : réinitialiser et prévenir Météo-France sans délai.
- **Disponibilité.** Aucune garantie. Les données peuvent changer sans préavis.
- **Quotas.** Ils existent, par seconde, minute ou jour, en requêtes ou en bande passante, et peuvent changer. La fiche data.gouv indique 100 req/min. Le Swagger dit `Unlimited`, à ne pas croire : gérer le `429`.
- **Responsabilité.** Le réutilisateur répond seul du manque de fiabilité ou de fraîcheur des données. La licence peut changer à tout moment : la relire avant chaque release.

Technique, d'après le Swagger et le [guide d'utilisation](https://confluence-meteofrance.atlassian.net/wiki/spaces/OpenDataMeteoFrance/pages/854032416) :

- WCS et WMS, base `https://api.meteofrance.fr/pro/piaf/1.0`, en-tête `Authorization: Bearer <jeton>`.
- Paramètre `TOTAL_PRECIPITATION_RATE__GROUND_OR_WATER_SURFACE`. Formats GeoTIFF et GRIB.
- **Modèle déterministe.** La mention « probabilistes » de la fiche data.gouv est contredite par le guide.
- **Une échéance par requête**, champ 2D uniquement, pas d'agrégation. Un run complet demande donc 39 appels, soit environ 8 par minute.
- `CoverageId` : `[paramètre]__[niveau]___[réseau ISO 8601 avec des points]`, listés par `GetCapabilities`. `DescribeCoverage` donne les axes `long lat height time`, temps en secondes.
- Rétention de 5 jours, pas d'archive.

### PIAF : mesures réelles

Faites le 5 novembre 2026 sur le run de 16 h 55 UTC, en ligne à 17 h 05.

| Mesure | Résultat |
| --- | --- |
| Authentification | `Authorization: Bearer <jeton>`. L'en-tête `apikey` répond `401` avec ce jeton |
| Durée du jeton | **1 heure** |
| Grille | 1651 × 1051, longitude -6 à 10,5, latitude 41 à 51,5, nord en haut, pas 0,01° |
| Échéances | 39, de 300 à 11 700 s |
| Durées de cumul | `PT5M`, `PT15M`, `PT30M`, `PT1H`, `PT3H`, à la fin du `CoverageId` |
| Unité | **`kg m-2`, soit un cumul en mm, malgré le nom du paramètre.** Taux en mm/h : `PT5M` × 12 |
| Vérification du cumul | Trois `PT5M` additionnés égalent le `PT15M` : ratio 1,0, écart max 0,0002 mm sur 34 838 pixels |
| Plus petite valeur non nulle | 0,01 mm en 5 min, soit 0,12 mm/h |
| Syntaxe | `subset=time(300)&subset=long(2.0,2.7)&subset=lat(48.6,49.1)` |
| Découpage | Identique au fichier France entière sur une zone de pluie |
| France, GRIB | 3,47 Mo, 0,7 s |
| France, GeoTIFF | 13,9 Mo, 1,9 s, flottants 64 bits non compressés |
| Zone de Paris, GeoTIFF | 35 Ko, 0,2 s |
| Runs disponibles | 1 355, depuis le 1er novembre |
| Lecture GRIB en Rust | Crate `grib` sans dépendance C : 71 ms par champ. Maximum et part de pluie identiques au GeoTIFF, nord en haut |
| `GetCapabilities` | 58 Ko en gzip, 2,1 Mo décompressé, 0,5 s, 6 955 `CoverageId` |
| GRIB découpé | 1 Ko pour 0,2° × 0,2°, 20 Ko pour 1° × 1°. Bornes recalées sur la grille, vers l'intérieur |
| Valeurs manquantes | Aucune : pas de bitmap, 0 NaN sur la France. Une zone sans radar ne se distingue donc pas d'une zone sèche. Non vérifié : ce que vaut la donnée loin des radars |
| Période du cumul | L'échéance 300 couvre `[run, run + 5 min]` d'après l'en-tête GRIB |
| Run complet, 1° × 1° | 39 fichiers, 0,80 Mo, environ 30 s |
| Âge à l'arrivée en boucle | 588 s sur un seul run, téléchargement compris. Échantillon trop court pour conclure |

**Volume :** 39 fichiers GRIB par run font environ 135 Mo, soit environ 39 Go par jour. Les quotas de bande passante existent mais ne sont pas publiés.

Non vérifié : les valeurs n'ont pas été comparées à une pluie observée au sol.

### Fraîcheur réelle

Un run PIAF arrive 10 min après son horaire. La donnée a donc au moins 10 min à sa réception, et l'horizon utile est d'environ 185 min.
L'extrapolation radar perd de sa qualité après environ 2 h, et plus vite pour les orages. L'interface ne doit pas sur-promettre au-delà.

### Test réel d'Open-Meteo

Appelé le 5 novembre 2026 sur Paris, avec et sans `models=meteofrance_seamless` : réponse correcte, pas de 15 min, sans clé, 0,00 mm partout. Il ne pleuvait pas, donc la qualité n'est pas évaluée.
La valeur est un cumul sur 15 min en mm, pas un taux.

## Apps similaires

| App | Promesse | Données | Confiance |
| --- | --- | --- | --- |
| Météo-France | Pluie dans l'heure | Propres données, via une API non publique | Vérifié par les clients communautaires |
| Apple Météo | Heure suivante à la minute, certaines régions | Service Apple. Le rachat de Dark Sky en est l'origine (de mémoire) | Partiel |
| Dark Sky | Le modèle du genre | API coupée le 31 mars 2023 | Rapporté par des tiers |
| Buienradar | 2 h aux Pays-Bas | Propre flux. Lien avec le radar KNMI non confirmé | Partiel |
| Yr | Nowcast nordique | MET Norway | Forte présomption |
| RainViewer (app) | Nowcast 2 h, radar | Modèle maison sur données radar | Déclaré par l'éditeur |
| Rain Today | 60 min à la minute | Fournisseur non nommé | Inconnu |
| Breezy Weather (Android, open source) | Agrège des sources | A refusé PIAF à cause des données personnelles demandées | Vérifié (issue 847) |

Leçons pour nous :

- Le créneau reste ouvert sur la simplicité. Les apps à radar empilent carte, abonnement et publicité.
- Aucune source n'expose une réponse prête. Toutes les apps sérieuses font du traitement côté serveur.
- Breezy a écarté PIAF pour une raison qui ne nous touche pas : une app grand public qui agrège demanderait une clé à chaque utilisateur. Nous, c'est **une clé côté serveur**.

## Contraintes plateforme

| Sujet | Constat | Confiance |
| --- | --- | --- |
| Tâches de fond iOS | Pas d'horaire garanti : insuffisant pour alerter à temps | Plusieurs sources, cohérent |
| Widget iOS | Budget de rafraîchissement dynamique, de l'ordre de 40 à 70 par jour | **Non vérifié**, doc Apple illisible par l'outil |
| Push de widget iOS | `WidgetPushHandler`, iOS 26+. Budgété et opportuniste, ne remplace pas la timeline. Échecs signalés sur le forum Apple | Vérifié |
| Widget Android | `updatePeriodMillis` : pas moins de 30 min. WorkManager : pas moins de 15 min, et plus en Doze | Vérifié |
| Mise à jour du widget par FCM | Pratique courante | Non vérifié |

## Sources

- Open-Meteo : [docs](https://open-meteo.com/en/docs), [Météo-France](https://open-meteo.com/en/docs/meteofrance-api), [conditions](https://open-meteo.com/en/terms), [tarifs](https://open-meteo.com/en/pricing)
- Météo-France : [API PIAF](https://www.data.gouv.fr/dataservices/6aba76fe9e189b1f12253326), [Swagger PIAF](https://portail-api.meteofrance.fr/api/am/devportal/v3/apis/aa174208-b051-4c95-af8f-dd52241a4d06/swagger?environmentName=APIS%20commerciales), [API AROME-PI](https://www.data.gouv.fr/dataservices/6661e72ec980db6ce7ac1c6b), [API Radar](https://www.data.gouv.fr/dataservices/6661e3b3ac0706063080aa40), [doc open data](https://confluence-meteofrance.atlassian.net/wiki/spaces/OpenDataMeteoFrance/pages/621019138/Mod+les+et+donn+es+de+pr+vision)
- API privée : [meteofrance-api](https://meteofrance-api.readthedocs.io/), [guide openHAB](https://github.com/sebSmarthome/openHAB3-widget/wiki/Pluie-dans-l'heure-M%C3%A9t%C3%A9o-France)
- Autres sources : [MET Norway Nowcast](https://api.met.no/weatherapi/nowcast/2.0/), [package radolan](https://pkg.go.dev/github.com/shyrmapp/radolan), [Apple WeatherKit](https://developer.apple.com/weatherkit/), [RainViewer, transition](https://rainviewer.com/api/transition-faq.html), [Buienradar pour Home Assistant](https://www.home-assistant.io/components/buienradar/), [rain_checker](https://github.com/mvdbr4nd/rain_checker)
- Apps : [Breezy Weather, issue 847](https://github.com/breezy-weather/breezy-weather/issues/847), [Rain Today](https://mwm.ai/apps/raintoday/326461840), [RainViewer](https://www.rainviewer.com/index.md), [Pirate Weather, issue 48](https://github.com/Pirate-Weather/pirateweather/issues/48), [Meteosource sur Dark Sky](https://www.meteosource.com/blog/how-to-migrate-from-dark-sky-api)
- Plateformes : [push WidgetKit](https://developer.apple.com/tutorials/data/documentation/widgetkit/updating-widgets-with-widgetkit-push-notifications.md), [forum Apple sur les push widget](https://developer.apple.com/forums/thread/839182), [widgets Android avancés](https://developer.android.com/develop/ui/views/appwidgets/advanced?hl=fr), [WorkManager et widgets](https://dev.to/tkuenneth/updating-widgets-with-jetpack-workmanager-g0b), [forum Apple sur WeatherKit en arrière-plan](https://developer.apple.com/forums/thread/730946)
- Limites du nowcast radar : [NHESS 2024](https://nhess.copernicus.org/articles/24/567/2024/nhess-24-567-2024.html)
