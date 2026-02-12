Voici le **PRD (Product Requirements Document)** de ton projet, rédigé en **Markdown** prêt à être utilisé dans ton repo (README ou dossier `/docs/PRD.md`).

---

# 📄 PRD — Exécuteur d’API CLI en Rust

## 1. 📌 Overview

### Nom du projet (temporaire)

**Apix** (API eXecutor CLI)
*(Nom modifiable ultérieurement)*

### Description

Apix est un exécuteur d’API en ligne de commande (CLI) développé en Rust, permettant d’envoyer des requêtes HTTP et d’inspecter les réponses via une interface terminal interactive construite avec **Ratatui**.

L’objectif est de proposer une alternative légère à Postman, entièrement en terminal, performante et extensible.

---

## 2. 🎯 Objectifs du produit

### Objectifs principaux

* Permettre l’exécution de requêtes HTTP (GET, POST, PUT, DELETE…)
* Afficher proprement les réponses (status, headers, body)
* Fournir une interface terminal moderne
* Permettre la sauvegarde et réutilisation des requêtes
* Gérer des environnements et variables

### Objectifs techniques (apprentissage Rust)

* Maîtrise de l’async/await avec Tokio
* Gestion d’erreurs avancée
* Architecture modulaire propre
* Manipulation JSON avec Serde
* Gestion d’état en TUI
* Ownership & Borrowing avancé

---

## 3. 👤 Utilisateurs cibles

* Développeurs backend
* Développeurs frontend
* Étudiants en informatique
* Utilisateurs Linux préférant le terminal
* Développeurs Rust souhaitant un outil natif

---

## 4. 🚀 Fonctionnalités

---

## 4.1 MVP (Version 1)

### Requêtes HTTP

* [ ] Support des méthodes :

  * GET
  * POST
  * PUT
  * DELETE
  * PATCH
* [ ] Entrée URL
* [ ] Ajout de headers
* [ ] Ajout body (JSON)
* [ ] Envoi requête
* [ ] Affichage :

  * Status code
  * Headers
  * Body

---

## 4.2 Interface Terminal

* [ ] Interface construite avec Ratatui
* [ ] Layout en deux panneaux :

  * Panneau requête
  * Panneau réponse
* [ ] Navigation clavier
* [ ] Scroll body réponse
* [ ] Indicateur de chargement

---

## 4.3 Gestion JSON

* [ ] Pretty print JSON
* [ ] Coloration syntaxique
* [ ] Indication erreur JSON invalide

---

## 4.4 Historique

* [ ] Sauvegarde automatique des requêtes
* [ ] Consultation historique
* [ ] Re-exécution rapide

Stockage local en JSON.

---

## 4.5 Collections

* [ ] Création collection
* [ ] Ajout requête à collection
* [ ] Suppression requête
* [ ] Sauvegarde locale

---

## 4.6 Environnements

* [ ] Fichier d’environnement

* [ ] Support variables :

  ```
  {{BASE_URL}}
  {{TOKEN}}
  ```

* [ ] Substitution automatique avant envoi

---

## 4.7 Authentification

* [ ] Bearer Token
* [ ] Basic Auth
* [ ] API Key

---

## 5. 🏗 Architecture technique

### Stack

| Composant       | Technologie         |
| --------------- | ------------------- |
| Langage         | Rust                |
| Runtime async   | Tokio               |
| HTTP client     | Reqwest             |
| TUI             | Ratatui + Crossterm |
| Sérialisation   | Serde               |
| Gestion erreurs | Anyhow / Thiserror  |
| Stockage        | Fichiers JSON       |

---

## 6. 🧠 Architecture logicielle

### Structure modulaire

```
src/
 ├── main.rs
 ├── app.rs
 ├── errors.rs
 ├── tui/
 ├── http/
 ├── models/
 ├── storage/
 └── config/
```

Voir `docs/architecture-tree.md` pour le détail complet.

### Séparation des responsabilités

| Module    | Responsabilité                                            |
| --------- | --------------------------------------------------------- |
| `http/`   | Envoi des requêtes et traitement des réponses             |
| `models/` | Structures de données pures (Request, Response, etc.)     |
| `tui/`    | Affichage Ratatui & interaction clavier                   |
| `storage/`| Persistance fichiers JSON (~/.apix/)                      |
| `config/` | Configuration, préférences, chargement des environnements |
| `errors`  | Types d'erreurs centralisés                               |

---

## 7. 🔁 Flux utilisateur

1. L’utilisateur lance `apix`
2. Il choisit méthode HTTP
3. Il entre URL
4. Il ajoute headers/body
5. Il appuie sur "Send"
6. L’application :

   * Construit RequestModel
   * Envoie via reqwest
   * Stocke réponse dans state
   * Affiche résultat

---

## 8. 📦 Stockage

Dossier local :

```
~/.apix/
 ├── history.json
 ├── collections.json
 └── environments.json
```

---

## 9. ⚡ Contraintes

* Doit fonctionner sous Linux, macOS et Windows
* Interface fluide (< 200ms interaction)
* Gestion propre des erreurs
* Aucune dépendance externe lourde

---

## 10. 🛣 Roadmap

### Phase 1 — CLI simple

* Requête HTTP sans UI

### Phase 2 — Intégration TUI

* Layout + interaction

### Phase 3 — JSON & headers dynamiques

### Phase 4 — Historique

### Phase 5 — Collections

### Phase 6 — Environnements

### Phase 7 — Auth & fonctionnalités avancées

---

## 11. 📊 Critères de succès

* Peut exécuter 100% des requêtes REST classiques
* Interface intuitive au clavier
* Pas de crash
* Code modulaire et testable
* Support JSON propre

---

## 12. 🔮 Évolutions futures

* Support GraphQL
* Support WebSocket
* Export OpenAPI
* Plugin system
* Mode headless scripting
* Version GUI future

---

## 13. 🎯 Vision long terme

Faire de Apix :

* Un outil CLI professionnel
* Un projet open-source mature
* Une démonstration avancée de maîtrise Rust
* Un projet portfolio fort
