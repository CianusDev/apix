# Taches — Projet Apix

## Phase 1 — CLI simple

### Initialisation
- [x] Creer le projet Rust binaire
- [x] Configurer les dependances (tokio, reqwest, serde, anyhow, thiserror)
- [x] Creer la structure de dossiers (`models/`)
- [x] Implementer la requete GET avec affichage status, headers, body

### Requetes HTTP
- [x] Implementer POST avec body JSON
- [x] Implementer PUT avec body JSON
- [x] Implementer DELETE
- [x] Implementer PATCH
- [x] Gestion des headers personnalises en argument
- [x] Gestion du body en argument (pour POST/PUT/PATCH)

### Architecture
- [x] Extraire la logique HTTP dans le module `http/` (client, request_builder, response)
- [x] Completer le module `models/` (response, collection, environment)
- [x] Creer `errors.rs` a la racine de `src/` (types d'erreurs centralises avec thiserror)
- [x] Creer le module `config/` (settings, chemins ~/.apix/, preferences)
- [x] Migrer de `trpl::block_on` vers `#[tokio::main]`
- [x] Gestion d'erreurs propre avec anyhow/thiserror (remplacer les `.expect()`)

### Tests
- [x] Tests unitaires pour le module `models/`
- [x] Tests unitaires pour le module `http/`

---

## Phase 2 — Integration TUI

- [x] Ajouter les dependances Ratatui + Crossterm
- [x] Creer le module `tui/` (mod, ui, events, state)
- [x] Layout deux panneaux (requete / reponse)
- [x] Navigation clavier
- [x] Scroll du body reponse
- [x] Indicateur de chargement
- [x] Gestion de l'etat applicatif dans `app.rs`

---

## Phase 3 — JSON & headers dynamiques

- [x] Pretty print JSON dans la reponse
- [x] Coloration syntaxique JSON
- [x] Gestion des reponses non-JSON (fallback texte brut)
- [x] Saisie dynamique des headers dans la TUI

---

## Phase 4 — Historique

- [x] Creer le module `models/history.rs` (HistoryEntry, History avec load/save)
- [x] Sauvegarde automatique des requetes dans `~/.apix/history.json`
- [x] Consultation de l'historique dans la TUI (touche h, panneau dedie)
- [x] Re-execution rapide depuis l'historique (Enter charge dans Request+Response)

---

## Phase 5 — Collections

- [x] Creation de collections (modele Collection/CollectionEntry/Collections + TUI navigation 2 niveaux)
- [x] Ajout / suppression de requetes dans une collection
- [x] Sauvegarde dans `~/.apix/collections.json`
- [x] Panneau TUI collections (touche c, liste collections, liste requetes, edition nom)

---

## Phase 6 — Environnements

- [x] Fichier d'environnement (`~/.apix/environments.json`) — modele Environment/Environments avec load/save
- [x] Support des variables (`{{BASE_URL}}`, `{{TOKEN}}`) — fonction substitute_variables()
- [x] Substitution automatique avant envoi (TUI + CLI)
- [x] Panneau TUI environnements (touche e, navigation 2 niveaux, edition variables, indicateur [ENV: nom])

---

## Phase 7 — Authentification

- [x] Bearer Token (Auth::BearerToken → header Authorization: Bearer <token>)
- [x] Basic Auth (Auth::BasicAuth → header Authorization: Basic base64(user:pass))
- [x] API Key (Auth::ApiKey → header custom configurable)
- [x] Champ Auth dans la TUI (selecteur type ←→, edition, Tab champs, barre d'aide)
- [x] Support des variables d'environnement dans l'auth ({{TOKEN}})
- [x] Auth sauvegardee dans collections et historique

---

## Phase 8 — Cookies

- [x] Activer le cookie store reqwest (feature `cookies` + `cookie_store(true)`)
- [x] Cookies geres automatiquement entre les requetes d'une session (partage via `Arc` interne du Client)
- [x] Affichage elargi des headers reponse (5 au lieu de 3)
- [x] Coloration distincte des headers Set-Cookie (jaune)

---

## Phase 9 — Sidebar fixe + Tabs Request/Response

- [x] Layout 3 colonnes : sidebar (24 chars) | request (50%) | response (50%)
- [x] Sidebar toujours visible avec 3 sections : History / Collections / Environments
- [x] Section active en hauteur 60%, sections inactives 20% chacune
- [x] Request panel : tabs Params | Headers | Body | Auth
- [x] Response panel : tabs Body | Headers (complet avec scroll) | Cookies (parsing Set-Cookie)
- [x] Navigation : Tab cycle Sidebar → Request → Response, [ et ] pour les tabs
- [x] Query params (Request.params) avec percent-encoding dans l'URL envoyee
- [x] Nouvelle methode `m` pour cycler la methode HTTP, `e` pour editer l'URL
- [x] Copie/sauvegarde reponse : y (presse-papiers) et w (fichier) toujours disponibles
