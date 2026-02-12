# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Apercu du projet

Apix est un executeur d'API HTTP en ligne de commande ecrit en Rust, visant a etre une alternative legere a Postman en terminal. Actuellement en debut de developpement (Phase 1 — CLI simple). Le projet integrera a terme une TUI basee sur Ratatui.

## Commandes

```bash
cargo build              # Compiler le projet
cargo run -- GET <url>   # Executer une requete GET (methode puis URL)
cargo test               # Lancer tous les tests
cargo test <nom_test>    # Lancer un test specifique
cargo clippy             # Linter
cargo fmt                # Formater le code
```

**Note :** La CLI prend les arguments sous la forme `METHODE URL` (ex: `cargo run -- GET https://example.com/api`).

## Architecture

Le projet utilise l'edition Rust 2024. Seul GET est implemente pour l'instant.

**Etat actuel :**
- `src/main.rs` — Point d'entree. Parse les args CLI, dispatch vers le handler HTTP async via `trpl::block_on`
- `src/models/request.rs` — Structures de donnees : enum `Method` (GET/POST/PUT/DELETE/PATCH) et struct `Request`
- `src/app.rs` — Vide, reserve pour l'etat futur de l'application

**Structure modulaire prevue** (voir `docs/architecture-tree.md`) :

| Module    | Responsabilite                              |
|-----------|---------------------------------------------|
| `tui/`    | Interface Ratatui & gestion clavier         |
| `http/`   | Execution des requetes via reqwest          |
| `models/` | Structures de donnees                       |
| `storage/`| Historique & collections (fichiers JSON dans `~/.apix/`) |
| `utils/`  | Helpers & gestion d'erreurs                 |

## Dependances cles

- **reqwest** (0.13) — Client HTTP
- **tokio** — Runtime async (mais utilise `trpl::block_on` au lieu de `#[tokio::main]` pour l'instant)
- **serde / serde_json** — Serialisation JSON
- **anyhow / thiserror** — Gestion d'erreurs

## Conventions

- Support multiplateforme vise (Linux, macOS, Windows)
- Gestion d'erreurs : `anyhow` pour les erreurs applicatives, `thiserror` pour les erreurs typees style librairie
