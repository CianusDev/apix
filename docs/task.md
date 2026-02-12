# Taches — Projet Apix

## Phase 1 — CLI simple

### Initialisation
- [x] Creer le projet Rust binaire
- [x] Configurer les dependances (tokio, reqwest, serde, anyhow, thiserror)
- [x] Creer la structure de dossiers (`models/`)
- [x] Implementer la requete GET avec affichage status, headers, body

### Requetes HTTP
- [ ] Implementer POST avec body JSON
- [ ] Implementer PUT avec body JSON
- [ ] Implementer DELETE
- [ ] Implementer PATCH
- [ ] Gestion des headers personnalises en argument
- [ ] Gestion du body en argument (pour POST/PUT/PATCH)

### Architecture
- [ ] Extraire la logique HTTP dans le module `http/` (client, request_builder, response)
- [ ] Completer le module `models/` (response, collection, environment)
- [ ] Creer `errors.rs` a la racine de `src/` (types d'erreurs centralises avec thiserror)
- [ ] Creer le module `config/` (settings, chemins ~/.apix/, preferences)
- [ ] Migrer de `trpl::block_on` vers `#[tokio::main]`
- [ ] Gestion d'erreurs propre avec anyhow/thiserror (remplacer les `.expect()`)

### Tests
- [ ] Tests unitaires pour le module `models/`
- [ ] Tests unitaires pour le module `http/`

---

## Phase 2 — Integration TUI

- [ ] Ajouter les dependances Ratatui + Crossterm
- [ ] Creer le module `tui/` (mod, ui, events, state)
- [ ] Layout deux panneaux (requete / reponse)
- [ ] Navigation clavier
- [ ] Scroll du body reponse
- [ ] Indicateur de chargement
- [ ] Gestion de l'etat applicatif dans `app.rs`

---

## Phase 3 — JSON & headers dynamiques

- [ ] Pretty print JSON dans la reponse
- [ ] Coloration syntaxique JSON
- [ ] Indication d'erreur si JSON invalide
- [ ] Saisie dynamique des headers dans la TUI

---

## Phase 4 — Historique

- [ ] Creer le module `storage/` (history, collections)
- [ ] Sauvegarde automatique des requetes dans `~/.apix/history.json`
- [ ] Consultation de l'historique dans la TUI
- [ ] Re-execution rapide depuis l'historique

---

## Phase 5 — Collections

- [ ] Creation de collections
- [ ] Ajout / suppression de requetes dans une collection
- [ ] Sauvegarde dans `~/.apix/collections.json`

---

## Phase 6 — Environnements

- [ ] Fichier d'environnement (`~/.apix/environments.json`)
- [ ] Support des variables (`{{BASE_URL}}`, `{{TOKEN}}`)
- [ ] Substitution automatique avant envoi

---

## Phase 7 — Authentification

- [ ] Bearer Token
- [ ] Basic Auth
- [ ] API Key
