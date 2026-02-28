<div align="center">

# apix

**A lightweight Postman alternative for your terminal**

[![Release](https://img.shields.io/github/v/release/CianusDev/apix?style=flat-square)](https://github.com/CianusDev/apix/releases)
[![License](https://img.shields.io/github/license/CianusDev/apix?style=flat-square)](LICENSE)
[![Build](https://img.shields.io/github/actions/workflow/status/CianusDev/apix/release.yml?style=flat-square&label=CI)](https://github.com/CianusDev/apix/actions)

```
curl -fsSL https://raw.githubusercontent.com/CianusDev/apix/main/install.sh | bash
```

</div>

---

`apix` is a terminal UI (TUI) HTTP client written in Rust.
Send requests, inspect responses, manage collections and environments — all without leaving your terminal.

```
┌─ APIX ─────────────────────────────────────────────────────────────────────┐
│  POST  https://api.example.com/users                            [ENV:prod] │
└────────────────────────────────────────────────────────────────────────────┘
┌─ Request ──────────────────────┐┌─ Response ────────────────────────────────┐
│ 1:Params│2:Headers│3:Body│4:Auth ││ ● 201 Created   POST                      │
│────────────────────────────────││ 1:Body│2:Headers│3:Cookies                 │
│ ▸ Authorization: Bearer tok…  ││──────────────────────────────────────────── │
│   Content-Type: application/… ││ {                                           │
│                                ││   "id": 42,                                │
│                                ││   "name": "Alice",                         │
│                                ││   "token": "eyJhbGci..."                   │
│                                ││ }                                           │
├────────────────────────────────┴┴───────────────────────────────────────────┤
│ h:History  c:Collections  e:Envs  u:URL  m:method  s:send  Tab:switch  q:quit│
└─────────────────────────────────────────────────────────────────────────────┘
```

## Installation

### One-liner (Linux & macOS)

```bash
curl -fsSL https://raw.githubusercontent.com/CianusDev/apix/main/install.sh | bash
```

Auto-detects your platform and installs the binary to `~/.local/bin`.

### Download manually

Grab the latest binary from the [**Releases page**](https://github.com/CianusDev/apix/releases):

| Platform            | File                                                       |
|---------------------|------------------------------------------------------------|
| Linux x86_64        | `apix-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz`             |
| macOS Intel         | `apix-vX.Y.Z-x86_64-apple-darwin.tar.gz`                  |
| macOS Apple Silicon | `apix-vX.Y.Z-aarch64-apple-darwin.tar.gz`                 |
| Windows x86_64      | `apix-vX.Y.Z-x86_64-pc-windows-msvc.zip`                  |

```bash
# Extract and install (Linux/macOS)
tar xzf apix-*.tar.gz
mv apix ~/.local/bin/
```

### Build from source

Requires [Rust](https://rustup.rs) (edition 2024):

```bash
git clone https://github.com/CianusDev/apix.git
cd apix
cargo build --release
./target/release/apix
```

## Features

- **Full HTTP support** — GET, POST, PUT, DELETE, PATCH
- **Persistent URL bar** with color-coded method badge
- **Query params** — inline key=value editor
- **Headers** — add, edit, delete
- **Body editor** — multi-line with auto-indent and JSON formatting (`Ctrl+f`)
- **Authentication** — Bearer token, Basic Auth, API Key
- **Environments** — define variables like `{{base_url}}` and switch between `dev` / `prod`
- **Collections** — save and reload requests
- **History** — full request log with search
- **JSON response** — syntax-highlighted body, headers, cookies tabs
- **Clipboard** — copy response body with `y`
- **Persistent cookies** — shared across requests in the same session

## Keybindings

### Global

| Key         | Action                         |
|-------------|--------------------------------|
| `s`         | Send request                   |
| `u`         | Edit URL                       |
| `m`         | Cycle method (GET→POST→…)      |
| `Tab`       | Switch focus Request ↔ Response|
| `h`         | Toggle History drawer          |
| `c`         | Toggle Collections drawer      |
| `e`         | Toggle Environments drawer     |
| `Esc`       | Close drawer / cancel edit     |
| `q` / Ctrl+C| Quit                           |

### Request panel

| Key        | Action                          |
|------------|---------------------------------|
| `1`        | Params tab                      |
| `2`        | Headers tab                     |
| `3`        | Body tab                        |
| `4`        | Auth tab                        |
| `[` / `]`  | Cycle tabs (previous / next)    |
| `↑` / `↓`  | Navigate list                  |
| `Enter`    | Edit selected item              |
| `a`        | Add new item                    |
| `d`        | Delete selected item            |

### Response panel

| Key        | Action                          |
|------------|---------------------------------|
| `1`        | Body tab                        |
| `2`        | Headers tab                     |
| `3`        | Cookies tab                     |
| `↑` / `↓`  | Scroll                         |
| `y`        | Copy body to clipboard          |
| `w`        | Save body to file               |

### Body editor

| Key     | Action                          |
|---------|---------------------------------|
| `Enter` | New line (auto-indent)          |
| `Tab`   | Insert 2 spaces                 |
| `Ctrl+f`| Format as JSON                  |
| `Esc`   | Save and exit editor            |

### Drawers (History / Collections / Environments)

| Key     | Action                                         |
|---------|------------------------------------------------|
| `↑` / `↓` | Navigate                                   |
| `Enter` | Load / open / activate                         |
| `d`     | Delete                                         |
| `n`     | New (collection / environment)                 |
| `v`     | View variables (Environments)                  |
| `a`     | Add variable / save current request            |
| `/`     | Search (History only)                          |
| `Esc`   | Go back / close drawer                         |

## Data storage

apix stores everything locally in `~/.apix/`:

```
~/.apix/
├── history.json        # Request history
├── collections.json    # Saved collections
└── environments.json   # Environments & variables
```

## Tech stack

| Component    | Library                     |
|--------------|-----------------------------|
| Language     | Rust (2024 edition)         |
| TUI          | Ratatui 0.30 + Crossterm 0.29 |
| HTTP client  | Reqwest 0.13                |
| Async        | Tokio                       |
| Serialization| Serde / serde_json          |

## Contributing

```bash
cargo test    # run all tests
cargo clippy  # lint
cargo fmt     # format
```

Pull requests welcome.

## License

[MIT](LICENSE)
