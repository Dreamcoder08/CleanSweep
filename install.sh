#!/bin/bash
# =============================================================================
# Dreamcoder Manager Installer
# Instala el binary de Rust desde GitHub releases o compila desde source
# =============================================================================

set -euo pipefail

REPO="dreamcoder08/Dreamcoder_dots"
BINARY="dreamcoder"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

detect_platform() {
    local os=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)
    
    case "$arch" in
        x86_64) arch="x86_64" ;;
        arm64|aarch64) arch="aarch64" ;;
        *) log_error "Arquitectura no soportada: $arch"; exit 1 ;;
    esac
    
    case "$os" in
        linux) os="linux" ;;
        darwin) os="macos" ;;
        *) log_error "OS no soportado: $os"; exit 1 ;;
    esac
    
    echo "${BINARY}-${os}-${arch}"
}

download_binary() {
    local platform=$1
    local version=${2:-latest}
    local tmpdir=$(mktemp -d)
    
    if [ "$version" = "latest" ]; then
        local url="https://github.com/${REPO}/releases/latest/download/${platform}.tar.gz"
    else
        local url="https://github.com/${REPO}/releases/download/${version}/${platform}.tar.gz"
    fi
    
    log_info "Descargando desde ${url}..."
    
    if command -v curl &>/dev/null; then
        curl -fsSL "$url" -o "${tmpdir}/dreamcoder.tar.gz"
    elif command -v wget &>/dev/null; then
        wget -q "$url" -O "${tmpdir}/dreamcoder.tar.gz"
    else
        log_error "Se requiere curl o wget"
        exit 1
    fi
    
    log_info "Extrayendo..."
    tar -xzf "${tmpdir}/dreamcoder.tar.gz" -C "$tmpdir"
    
    echo "${tmpdir}/${BINARY}"
}

install_binary() {
    local binary_path=$1
    
    # Crear directorio si no existe
    mkdir -p "$INSTALL_DIR"
    
    # Copiar binary
    cp "$binary_path" "$INSTALL_DIR/${BINARY}"
    chmod +x "$INSTALL_DIR/${BINARY}"
    
    log_success "Binary instalado en $INSTALL_DIR/${BINARY}"
    
    # Verificar si está en PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        log_warn "$INSTALL_DIR no está en tu PATH"
        echo ""
        echo "Agrega esto a tu ~/.bashrc o ~/.zshrc:"
        echo "export PATH=\"$INSTALL_DIR:\$PATH\""
    fi
}

build_from_source() {
    log_info "Compilando desde source..."
    
    if ! command -v cargo &>/dev/null; then
        log_error "Rust no está instalado. Instala desde https://rustup.rs"
        exit 1
    fi
    
    local tmpdir=$(mktemp -d)
    cd "$tmpdir"
    
    log_info "Clonando repositorio..."
    git clone --depth 1 "https://github.com/${REPO}.git"
    cd Dreamcoder_dots/dreamcoder-manager-rust
    
    log_info "Compilando (release mode)..."
    cargo build --release
    
    echo "target/release/${BINARY}"
}

main() {
    echo "🎩 Dreamcoder Manager Installer"
    echo "================================"
    echo ""
    
    local method=${1:-auto}
    local version=${2:-latest}
    
    case "$method" in
        auto|download)
            local platform=$(detect_platform)
            log_info "Plataforma detectada: $platform"
            
            local binary_path=$(download_binary "$platform" "$version")
            install_binary "$binary_path"
            ;;
        build|source)
            local binary_path=$(build_from_source)
            install_binary "$binary_path"
            ;;
        *)
            echo "Uso: $0 [auto|build] [version]"
            echo ""
            echo "Ejemplos:"
            echo "  $0              # Descarga latest binary"
            echo "  $0 auto v2.0.0  # Descarga versión específica"
            echo "  $0 build        # Compila desde source"
            exit 1
            ;;
    esac
    
    echo ""
    log_success "Instalación completada!"
    echo ""
    echo "Prueba con: dreamcoder --version"
    echo "o: dreamcoder --help"
}

main "$@"
