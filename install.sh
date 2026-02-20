#!/usr/bin/env bash
# install.sh — Apix installer
# Usage: curl -fsSL https://raw.githubusercontent.com/CianusDev/apix/main/install.sh | bash

set -euo pipefail

REPO="CianusDev/apix"
BINARY="apix"
INSTALL_DIR="${APIX_INSTALL_DIR:-}"

# ── Colors ─────────────────────────────────────────────────────────────────────
if [ -t 1 ]; then
  RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
  CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'
else
  RED=''; GREEN=''; YELLOW=''; CYAN=''; BOLD=''; RESET=''
fi

info()    { echo -e "${CYAN}${BOLD}→${RESET} $*"; }
success() { echo -e "${GREEN}${BOLD}✓${RESET} $*"; }
warn()    { echo -e "${YELLOW}${BOLD}!${RESET} $*"; }
error()   { echo -e "${RED}${BOLD}✗${RESET} $*" >&2; exit 1; }

# ── Detect platform ────────────────────────────────────────────────────────────
detect_target() {
  local os arch
  os=$(uname -s)
  arch=$(uname -m)

  case "$os" in
    Linux*)
      case "$arch" in
        x86_64)  echo "x86_64-unknown-linux-gnu" ;;
        *)        error "Architecture Linux non supportée: $arch. Compilez depuis les sources: cargo install --git https://github.com/${REPO}" ;;
      esac
      ;;
    Darwin*)
      case "$arch" in
        x86_64)  echo "x86_64-apple-darwin" ;;
        arm64)   echo "aarch64-apple-darwin" ;;
        *)        error "Architecture macOS non supportée: $arch" ;;
      esac
      ;;
    MINGW*|MSYS*|CYGWIN*|Windows*)
      error "Windows détecté. Téléchargez le binaire .zip depuis :\nhttps://github.com/${REPO}/releases/latest"
      ;;
    *)
      error "Système non supporté: $os"
      ;;
  esac
}

# ── Resolve install directory ──────────────────────────────────────────────────
resolve_install_dir() {
  if [ -n "$INSTALL_DIR" ]; then
    echo "$INSTALL_DIR"
    return
  fi

  # Prefer /usr/local/bin if writable, else ~/.local/bin
  if [ -w "/usr/local/bin" ]; then
    echo "/usr/local/bin"
  else
    local local_bin="$HOME/.local/bin"
    mkdir -p "$local_bin"
    echo "$local_bin"
  fi
}

# ── Check dependencies ─────────────────────────────────────────────────────────
check_deps() {
  for cmd in curl tar; do
    command -v "$cmd" &>/dev/null || error "Commande requise non trouvée: $cmd"
  done
}

# ── Get latest release version ─────────────────────────────────────────────────
get_latest_version() {
  local version
  version=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' \
    | sed -E 's/.*"([^"]+)".*/\1/')

  [ -n "$version" ] || error "Impossible de récupérer la version depuis GitHub. Vérifiez votre connexion."
  echo "$version"
}

# ── Main ───────────────────────────────────────────────────────────────────────
main() {
  echo ""
  echo -e "${BOLD}  Apix — API eXecutor${RESET}"
  echo -e "  ${CYAN}https://github.com/${REPO}${RESET}"
  echo ""

  check_deps

  local target version filename url tmp_dir install_dir

  info "Détection de la plateforme..."
  target=$(detect_target)
  success "Plateforme: ${BOLD}${target}${RESET}"

  info "Récupération de la dernière version..."
  version=$(get_latest_version)
  success "Version: ${BOLD}${version}${RESET}"

  filename="${BINARY}-${version}-${target}.tar.gz"
  url="https://github.com/${REPO}/releases/download/${version}/${filename}"

  info "Téléchargement de ${filename}..."
  tmp_dir=$(mktemp -d)
  trap 'rm -rf "$tmp_dir"' EXIT

  curl -fsSL --progress-bar -o "${tmp_dir}/${filename}" "$url" \
    || error "Échec du téléchargement depuis:\n  ${url}\n\nVérifiez que la release ${version} existe pour la cible ${target}."

  info "Extraction..."
  tar xzf "${tmp_dir}/${filename}" -C "$tmp_dir"

  install_dir=$(resolve_install_dir)
  info "Installation dans ${BOLD}${install_dir}${RESET}..."

  # Ask for sudo if directory not writable
  if [ ! -w "$install_dir" ]; then
    warn "Droits insuffisants pour $install_dir, utilisation de sudo..."
    sudo mv "${tmp_dir}/${BINARY}" "${install_dir}/${BINARY}"
    sudo chmod +x "${install_dir}/${BINARY}"
  else
    mv "${tmp_dir}/${BINARY}" "${install_dir}/${BINARY}"
    chmod +x "${install_dir}/${BINARY}"
  fi

  echo ""
  success "${BOLD}apix ${version}${RESET} installé dans ${BOLD}${install_dir}/${BINARY}${RESET}"

  # PATH check
  if ! echo ":${PATH}:" | grep -q ":${install_dir}:"; then
    echo ""
    warn "${install_dir} n'est pas dans votre PATH."
    echo ""
    echo "  Ajoutez cette ligne à votre ~/.bashrc ou ~/.zshrc :"
    echo -e "  ${CYAN}export PATH=\"${install_dir}:\$PATH\"${RESET}"
    echo ""
    echo "  Puis rechargez votre shell :"
    echo -e "  ${CYAN}source ~/.bashrc${RESET}  (ou ~/.zshrc)"
  fi

  echo ""
  echo -e "  Lancez ${BOLD}apix${RESET} pour démarrer !"
  echo ""
}

main "$@"
