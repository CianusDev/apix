```
src/
├── main.rs                  # Point d'entree, parsing args
├── app.rs                   # Etat applicatif global (requete, reponse, collection active, env)
│
├── http/
│   ├── mod.rs
│   ├── client.rs            # Envoi des requetes via reqwest
│   ├── request_builder.rs   # Construction de la requete (headers, body, auth, variables)
│   └── response.rs          # Parsing et formatage de la reponse
│
├── models/
│   ├── mod.rs
│   ├── request.rs           # Struct Request, enum Method
│   ├── response.rs          # Struct Response (status, headers, body)
│   ├── collection.rs        # Struct Collection
│   └── environment.rs       # Struct Environment, variables
│
├── tui/
│   ├── mod.rs
│   ├── ui.rs                # Rendu Ratatui (layout, panneaux)
│   ├── events.rs            # Gestion clavier / evenements
│   └── state.rs             # Etat UI uniquement (panneau actif, scroll, curseur)
│
├── storage/
│   ├── mod.rs
│   ├── history.rs           # Lecture/ecriture ~/.apix/history.json
│   └── collections.rs       # Lecture/ecriture ~/.apix/collections.json
│
├── config/
│   ├── mod.rs
│   └── settings.rs          # Chemins ~/.apix/, preferences, chargement env
│
└── errors.rs                # Types d'erreurs centralises (thiserror)
```

## Separation des responsabilites

| Module    | Responsabilite                                            |
|-----------|-----------------------------------------------------------|
| `http/`   | Envoi des requetes et traitement des reponses             |
| `models/` | Structures de donnees pures (Request, Response, etc.)     |
| `tui/`    | Affichage Ratatui & interaction clavier                   |
| `storage/`| Persistance fichiers JSON (~/.apix/)                      |
| `config/` | Configuration, preferences, chargement des environnements |
| `errors`  | Types d'erreurs centralises                               |

## Regles app.rs vs tui/state.rs

- **`app.rs`** — etat qui existe meme sans TUI : requete en cours, reponse recue, collection active, environnement selectionne
- **`tui/state.rs`** — etat purement visuel : panneau actif, position du scroll, curseur, mode edition, taille du terminal
