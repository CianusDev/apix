# Apix — API eXecutor

> Alternative légère à Postman, entièrement dans votre terminal.

Apix est un client HTTP interactif en TUI (Terminal User Interface) écrit en Rust.
Envoyez des requêtes, explorez les réponses, gérez vos collections et environnements —
sans jamais quitter votre terminal.

```
┌─ APIX ──────────────────────────────────────────────────────────────────────┐
│ POST   https://api.example.com/users                              [ENV:prod] │
└─────────────────────────────────────────────────────────────────────────────┘
┌─ Request ────────────────────┐┌─ Response ──────────────────────────────────┐
│ 1:Params│2:Headers│3:Body│4:Auth ││ ● 200 OK   POST                              │
│──────────────────────────────││ 1:Body│2:Headers│3:Cookies                    │
│ ▸ Content-Type: application/ ││──────────────────────────────────────────────│
│   Authorization: Bearer tok… ││ {                                            │
│                              ││   "id": 42,                                  │
│                              ││   "name": "Alice",                           │
│                              ││   "email": "alice@example.com"               │
│                              ││ }                                            │
├──────────────────────────────┴┴─────────────────────────────────────────────┤
│ h:History  c:Collections  E:Envs  e:URL  m:method  s:send  Tab:panel  q:quit│
└─────────────────────────────────────────────────────────────────────────────┘
```

## Installation

### Méthode rapide (Linux & macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/CianusDev/apix/main/install.sh | bash
```

Le script détecte automatiquement votre plateforme, télécharge le bon binaire
et l'installe dans `~/.local/bin` ou `/usr/local/bin`.

### Téléchargement manuel

Téléchargez le binaire pour votre plateforme depuis la
[page Releases](https://github.com/CianusDev/apix/releases/latest) :

| Plateforme          | Fichier                                              |
|---------------------|------------------------------------------------------|
| Linux x86_64        | `apix-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz`       |
| macOS Intel         | `apix-vX.Y.Z-x86_64-apple-darwin.tar.gz`            |
| macOS Apple Silicon | `apix-vX.Y.Z-aarch64-apple-darwin.tar.gz`           |
| Windows x86_64      | `apix-vX.Y.Z-x86_64-pc-windows-msvc.zip`            |

```bash
# Exemple Linux
tar xzf apix-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz
mv apix ~/.local/bin/
```

### Depuis les sources (Rust requis)

```bash
git clone https://github.com/CianusDev/apix.git
cd apix
cargo build --release
./target/release/apix
```

Ou directement via Cargo :

```bash
cargo install --git https://github.com/CianusDev/apix
```

> **Rust 2024 edition** requis. Installez via [rustup.rs](https://rustup.rs).

---

## Utilisation

Lancez simplement `apix` dans votre terminal pour ouvrir l'interface :

```bash
apix
```

---

## Raccourcis clavier

### Globaux (toujours disponibles)

| Touche       | Action                            |
|--------------|-----------------------------------|
| `s`          | Envoyer la requête                |
| `e`          | Éditer l'URL                      |
| `m`          | Changer la méthode HTTP           |
| `Tab`        | Basculer Request ↔ Response       |
| `h`          | Ouvrir/fermer le tiroir Historique|
| `c`          | Ouvrir/fermer le tiroir Collections|
| `E` (Maj)   | Ouvrir/fermer le tiroir Environments|
| `Esc`        | Fermer le tiroir ouvert           |
| `q` / Ctrl+C | Quitter                           |

### Panneau Request

| Touche    | Action                               |
|-----------|--------------------------------------|
| `1`       | Tab Params                           |
| `2`       | Tab Headers                          |
| `3`       | Tab Body                             |
| `4`       | Tab Auth                             |
| `↑` / `↓` | Naviguer dans la liste              |
| `Enter`   | Éditer l'item sélectionné            |
| `a`       | Ajouter un item                      |
| `d`       | Supprimer l'item sélectionné         |

### Panneau Response

| Touche    | Action                               |
|-----------|--------------------------------------|
| `1`       | Tab Body                             |
| `2`       | Tab Headers                          |
| `3`       | Tab Cookies                          |
| `↑` / `↓` | Scroller le contenu                 |
| `y`       | Copier dans le presse-papiers        |
| `w`       | Sauvegarder dans un fichier          |

### Tiroir Historique (`h`)

| Touche    | Action                               |
|-----------|--------------------------------------|
| `↑` / `↓` | Naviguer                            |
| `Enter`   | Charger la requête                   |
| `d`       | Supprimer l'entrée                   |
| `/`       | Rechercher dans l'historique         |
| `Esc`     | Fermer le tiroir                     |

### Tiroir Collections (`c`)

| Touche    | Action                               |
|-----------|--------------------------------------|
| `↑` / `↓` | Naviguer                            |
| `Enter`   | Ouvrir / charger une requête         |
| `n`       | Nouvelle collection                  |
| `a`       | Sauvegarder la requête courante      |
| `d`       | Supprimer                            |
| `Esc`     | Retour / fermer                      |

### Tiroir Environments (`E`)

| Touche    | Action                               |
|-----------|--------------------------------------|
| `↑` / `↓` | Naviguer                            |
| `Enter`   | Activer / désactiver l'environnement |
| `v`       | Voir les variables                   |
| `n`       | Nouvel environnement                 |
| `a`       | Ajouter une variable                 |
| `d`       | Supprimer                            |
| `Esc`     | Retour / fermer                      |

### Édition de corps (Body)

| Touche     | Action                          |
|------------|---------------------------------|
| `Enter`    | Nouvelle ligne (avec indentation automatique) |
| `Tab`      | Indenter (2 espaces)            |
| `Ctrl+f`   | Formater le JSON                |
| `↑` / `↓`  | Se déplacer ligne par ligne    |
| `Esc`      | Valider et fermer l'éditeur     |

---

## Fonctionnalités

- **Méthodes HTTP** : GET, POST, PUT, DELETE, PATCH
- **URL bar persistante** avec badge méthode coloré
- **Query params** avec édition inline
- **Headers** personnalisés
- **Body** avec éditeur multi-ligne et formatage JSON auto
- **Authentification** : Bearer Token, Basic Auth, API Key
- **Historique** des requêtes avec recherche
- **Collections** de requêtes sauvegardées
- **Environnements** avec variables substituées (`{{VAR}}`)
- **Réponse** : body JSON colorisé, headers, cookies
- **Clipboard** : copier la réponse en un raccourci
- **Cookies** persistants entre les requêtes

---

## Stack technique

| Composant       | Technologie              |
|-----------------|--------------------------|
| Langage         | Rust (edition 2024)      |
| Runtime async   | Tokio                    |
| Client HTTP     | Reqwest 0.13             |
| TUI             | Ratatui 0.30 + Crossterm 0.29 |
| Sérialisation   | Serde / serde_json       |
| Gestion erreurs | Anyhow / Thiserror       |
| Stockage        | JSON dans `~/.apix/`     |

---

## Données persistantes

Apix stocke ses données dans `~/.apix/` :

```
~/.apix/
├── history.json       # Historique des requêtes
├── collections.json   # Collections sauvegardées
└── environments.json  # Environnements et variables
```

---

## Contribuer

Les contributions sont bienvenues !

```bash
git clone https://github.com/CianusDev/apix.git
cd apix
cargo test        # Lancer les tests
cargo clippy      # Linter
cargo fmt         # Formater
```

---

## Licence

MIT — voir [LICENSE](LICENSE) pour les détails.
