# Apix

Apix (API eXecutor) est un executeur d'API HTTP en ligne de commande ecrit en Rust. L'objectif est de proposer une alternative legere a Postman, entierement en terminal, performante et extensible.

## Installation

### Prerequis

- [Rust](https://www.rust-lang.org/tools/install) (edition 2024)

### Compilation

```bash
git clone <url-du-repo>
cd apix
cargo build --release
```

Le binaire sera disponible dans `target/release/apix`.

## Utilisation

```bash
# Requete GET
cargo run -- GET https://jsonplaceholder.typicode.com/posts/1
```

La reponse affiche le status code, les headers et le body JSON.

## Fonctionnalites

### Implementees

- Requetes GET avec affichage du status, headers et body JSON

### A venir

- Support complet des methodes HTTP (POST, PUT, DELETE, PATCH)
- Headers et body personnalises
- Interface terminal interactive (TUI) avec Ratatui
- Pretty print et coloration syntaxique JSON
- Historique des requetes
- Collections de requetes
- Environnements et variables (`{{BASE_URL}}`, `{{TOKEN}}`)
- Authentification (Bearer Token, Basic Auth, API Key)

## Stack technique

| Composant       | Technologie        |
|-----------------|--------------------|
| Langage         | Rust               |
| Runtime async   | Tokio              |
| Client HTTP     | Reqwest            |
| TUI (a venir)   | Ratatui + Crossterm |
| Serialisation   | Serde              |
| Gestion erreurs | Anyhow / Thiserror |
| Stockage        | Fichiers JSON      |

## Structure du projet

```
src/
├── main.rs          # Point d'entree et logique CLI
├── app.rs           # Etat de l'application (a venir)
└── models/
    └── request.rs   # Structures Request et Method
```

Voir `docs/architecture-tree.md` pour la structure modulaire complete prevue.

## Licence

A definir.
