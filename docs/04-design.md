# Design

Direction issue des skills `unslop` et `ui-ux-pro-max`. Valeurs de départ, à juger sur device.

## Direction

- **Produit.** Dire à quelqu'un qui va sortir dans combien de minutes il va pleuvoir.
- **Caractère.** Laconique, exact, un peu pince-sans-rire.
- **Monde.** Le pas de la porte. Le ciel gris qu'on regarde avant de sortir, et le ciré jaune accroché à côté.
- **Moment.** La bande de pluie : 3 h de pluie, seul élément coloré de l'écran.
- **Signature.** Le jaune ciré, réservé à la pluie. S'il n'y a pas de jaune à l'écran, il ne pleut pas.

Test de template : l'écran doit se lire de loin, comme un panneau d'affichage de gare, et ne pourrait pas appartenir à une autre app.

## Écran principal

```text
┌──────────────────────────────┐
│ Paris 11e                 ⚙  │
│                              │
│                              │
│ Pluie dans                   │
│ 20 min                       │  ← le plus gros texte de l'app
│ Jusque vers 15 h 50          │
│                              │
│                              │
│         ▂▅▇▅▃▂               │  ← bande de pluie, jaune
│ ▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁ │
│ 14 h 45     15 h 45    16 h 45│
│                              │
│ Météo-France, il y a 12 min  │
└──────────────────────────────┘
```

Aligné à gauche, pas de carte, pas d'onglets, pas d'illustration météo. Le geste de tirer vers le bas rafraîchit.
La ligne de source en bas est une obligation de licence : elle ne disparaît jamais.

## Textes

Ton : plat, précis, sans point d'exclamation. Heures au format français `17 h 40`.

| Situation | Texte |
| --- | --- |
| Sec | Pas de pluie avant 17 h 40 |
| Pluie proche | Pluie dans 20 min |
| Pluie à plus d'une heure | Pluie dans 1 h 40 |
| Pluie en cours | Il pleut. Fin vers 16 h 15 |
| Pluie longue | Il pleut pour au moins 3 h |
| Précision 15 min | Pluie dans ~30 min |
| Donnée périmée | Prévision indisponible |
| Notification | Pluie dans 15 min. Jusque vers 16 h 30 |
| Permission localisation | Pour savoir quand il pleuvra là où tu es |

Interdit : météo rigolote à chaque écran, emoji, « Bonjour », mots comme « intelligent » ou « hyperlocal ».

## Couleurs

Rampe graphite neutre d'`unslop`. Contrastes mesurés.

| Token | Sombre | Clair | Contraste sombre / clair |
| --- | --- | --- | --- |
| `bg` | `#141415` | `#FAFAFA` | |
| `surface` | `#19191B` | `#FFFFFF` | |
| `raised` | `#232325` | `#F4F4F5` | |
| `hairline` | `text` à 8 % | `text` à 8 % | |
| `text` | `#EDEDED` | `#171717` | |
| `text-2` | `#A1A1A6` | `#525252` | 7,16 / 7,49 |
| `text-3` | `#85858B` | `#737373` | 5,02 / 4,54 |
| `rain` (fond, barres) | `#F5C518` | `#B38900` | 11,29 / 3,10 |
| `rain-text` (texte) | `#F5C518` | `#7A5F00` | 11,29 / 5,81 |
| `on-rain` | `#141415` | `#141415` | 11,29 sur `#F5C518` |

Ces tokens sont déclarés une fois dans le thème HeroUI, qui attend du HSL selon son skill. Les composants n'utilisent jamais de valeur brute.

Le jaune `#F5C518` sur fond clair ne fait que 1,56:1 : **jamais de `#F5C518` sur fond clair.**
Rouge d'erreur inutile : `unknown` s'écrit en `text-2`.

## Typographie

Geist, poids 400 et 500. Chiffres tabulaires partout (`fontVariant: ['tabular-nums']`). Police système dans les widgets.

| Rôle | Taille / interligne | Poids |
| --- | --- | --- |
| Réponse (`20 min`) | 88 / 92, réduit si le texte déborde | 500 |
| Amorce (`Pluie dans`) | 28 / 34 | 400 |
| Détail | 17 / 24 | 400 |
| Légende | 13 / 20 | 400 |

Pas de capitales espacées, pas d'italique. Tailles respectant le réglage système.

## Espacement et formes

- Grille de 4. Bord d'écran 24. Blocs 32 à 64.
- Rayons : bouton et ligne 10, feuille 20.
- Lignes de réglages : 48 de haut minimum.
- Pas d'ombre en sombre. Séparation par `hairline`.
- Icônes : lucide, 20 px, trait 1,5, couleur `text-2`.

## Mouvement

- Une courbe : `Easing.bezier(0.22, 1, 0.36, 1)`.
- Pression 120 ms, feuille 300 ms.
- Une seule animation à l'ouverture : les barres montent, 300 ms.
- Nombre qui change : fondu court, pas de compteur qui défile.
- Aucune animation de pluie, aucune boucle. `Réduire les animations` : tout est statique.

## Widgets

Fond `bg`, un seul chiffre, et la bande de pluie si la taille le permet. Une ligne de légende affiche `Météo-France 14 h 35`, pour la licence. Sur le format rectangulaire, trop petit, le tap ouvre l'écran principal qui la porte : interprétation à confirmer si le doute persiste.

```text
petit             rectangulaire (écran verrouillé)
┌──────────┐      ┌──────────────────┐
│ Pluie    │      │ Pluie dans 20 min│
│ 20 min   │      │ ▁▁▂▅▇▅▂▁▁▁▁▁▁▁▁▁ │
│ ▁▂▅▇▅▂▁▁ │      └──────────────────┘
└──────────┘
```

## Accessibilité

- Le nombre et sa phrase forment un seul élément lu : « Pluie dans 20 minutes, jusque vers 15 heures 50 ».
- La bande de pluie est décrite par une phrase, pas barre par barre.
- La couleur ne porte jamais seule l'information : hauteur des barres et texte disent la même chose.
- Cibles : 44 pt sur iOS, 48 dp sur Android. `accessibilityRole` et `accessibilityLabel` sur chaque contrôle.

## Checklist avant de dire « fini »

- [ ] Clair et sombre vérifiés séparément, sur device ou simulateur
- [ ] Texte système au maximum : rien de coupé
- [ ] Petit écran (375 pt) et zones sûres respectés
- [ ] Lecteur d'écran : ordre et libellés corrects
- [ ] Animations réduites : écran complet et statique
- [ ] Les quatre états affichés, plus hors ligne
- [ ] Aucun emoji, aucun dégradé, aucune lueur, aucune capitale espacée
- [ ] Le jaune n'apparaît que pour la pluie
- [ ] Source et heure de mise à jour visibles
- [ ] Tous les espacements sur la grille de 4, toutes les valeurs via les tokens
- [ ] Une seule famille d'icônes, une seule taille
- [ ] Pas de point d'exclamation, pas de point final sur les libellés
- [ ] Test de template : avec un autre nom, l'écran conviendrait-il encore à une autre app ?

## Ce qu'on a gardé des skills

`ui-ux-pro-max` a proposé : minimalisme suisse, Plus Jakarta Sans, bleu ciel et ambre, structure de type page d'accueil marketing.
Gardés : le minimalisme suisse et ses exigences d'accessibilité. Écartés : la palette et la police, trop génériques pour une app météo, et la structure marketing, sans objet ici. Aucun résultat pour la recherche widget et notification sur la stack React Native : ces règles viennent des docs plateforme et de `references/pro-rules.md`.

Rampe, échelles et méthode de direction adaptées de **unslop** par Damir Sokolovsky ([@heydamir](https://x.com/heydamir)), CC BY 4.0.
