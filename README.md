# Is it raining

App météo mobile à une seule question : dans combien de minutes il va pleuvoir ?
Un écran, des widgets, une notification avant la pluie. « Is it raining » est un nom de travail.

**État : serveur en cours (M1), écran principal de l'app sur émulateur (M2).** Suivi dans [docs/06-roadmap.md](docs/06-roadmap.md).

## Documentation

| Fichier | Contenu |
| --- | --- |
| [AGENTS.md](AGENTS.md) | Règles de développement, pour humains et agents |
| [CLAUDE.md](CLAUDE.md) | Importe AGENTS.md, plus les consignes propres à Claude Code |
| [docs/00-produit.md](docs/00-produit.md) | Périmètre, états de l'app, ce qu'on ne fait pas |
| [docs/01-recherche.md](docs/01-recherche.md) | Sources météo, apps similaires, sources citées |
| [docs/02-architecture.md](docs/02-architecture.md) | Flux, contrat d'API, widgets, notifications |
| [docs/03-stack.md](docs/03-stack.md) | Technos et comparatif des bibliothèques UI |
| [docs/04-design.md](docs/04-design.md) | Direction, tokens, règles anti-slop, checklist |
| [docs/05-process-dev.md](docs/05-process-dev.md) | Workflow, vérifications, Orca, release |
| [docs/06-roadmap.md](docs/06-roadmap.md) | Jalons et statut, à tenir à jour |
| [docs/decisions.md](docs/decisions.md) | Journal des décisions |

## Arborescence prévue du code

Ces dossiers seront créés par les tâches qui en ont besoin, pas avant.

```text
server/    Rust : ingestion météo, API, alertes, push
app/       Expo / React Native : écrans et réglages
app/targets/   widgets natifs iOS (Swift)
app/plugins/   widget Android (Kotlin), selon le spike S3
```
