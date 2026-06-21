# 🎩 Dreamcoder Dotfiles Manager v2.0
## AI-First Edition | Rust Implementation

**El gestor de dotfiles definitivo para la era de la IA.**

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Build Status](https://img.shields.io/badge/build-passing-brightgreen.svg)]()

---

## 🚀 Características Principales

### ✅ Implementadas

| Feature | Estado | Detalle |
|---------|--------|---------|
| **AI Protocol** | ✅ | Output JSON estructurado para Claude integration |
| **Stow Nativo** | ✅ | Reimplementación completa en Rust, no requiere GNU Stow |
| **Templates** | ✅ | Handlebars con contexto OS/arch |
| **TUI** | ✅ | Interfaz terminal con Ratatui (progreso, menús) |
| **Secrets** | ✅ | Age + 1Password + Bitwarden |
| **Core Engine** | ✅ | Async, error handling robusto |
| **CLI** | ✅ | Clap con subcomandos, flags |
| **Non-interactive** | ✅ | `--json --yes` para CI/CD |
| **Tests** | ✅ | Tests de integración para AI Protocol y Stow |

### 🔄 En Desarrollo

| Feature | Estado |
|---------|--------|
| Gestión de paquetes | 🔄 Estructura lista |
| Auto-update | 🔄 Estructura lista |
| Plugin system (WASM) | 📋 Roadmap v2.1 |

---

## 📦 Instalación

### One-Liner (Linux/macOS)
```bash
curl -fsSL https://get.dreamcoder.dev | bash
```

### Cargo (Rust)
```bash
cargo install dreamcoder-cli
```

### Compilar desde Source
```bash
git clone https://github.com/dreamcoder08/Dreamcoder_dots.git
cd dreamcoder-manager-rust
cargo build --release
# Binary en: target/release/dreamcoder
```

---

## 🎯 Uso

### Modo TUI Interactivo (Recomendado)
```bash
dreamcoder
# o
dreamcoder interactive
```

Navegación:
- `j/k` o `↑/↓` - Moverse entre módulos
- `Enter` - Instalar módulo seleccionado
- `d/m/b/s` - Cambiar tab (Dashboard/Modules/Backups/Settings)
- `q` - Salir

### CLI Tradicional

```bash
# Aplicar todos los módulos
dreamcoder apply

# Aplicar específicos
dreamcoder apply DreamcoderShell DreamcoderNvim

# Simular cambios
dreamcoder apply --dry-run

# Forzar sin backups
dreamcoder apply --no-backup --force

# Status JSON (para AI)
dreamcoder status --json

# Backups
dreamcoder backup create
dreamcoder backup list
dreamcoder backup restore latest
```

---

## 🤖 AI Protocol

Dreamcoder está diseñado para ser usado por humanos **Y por inteligencia artificial**.

### Ejemplo con Claude Code

```bash
# 1. Claude detecta estado
claude> dreamcoder status --json

{
  "version": "2.0.0",
  "status": "success",
  "data": {
    "system": { "os": "linux", "arch": "x86_64" },
    "modules": [
      { "name": "shell", "installed": false },
      { "name": "nvim", "installed": true }
    ]
  }
}

# 2. Claude decide instalar
claude> dreamcoder apply shell --json

{
  "status": "requires_input",
  "requires_action": {
    "action_type": "confirm",
    "message": "Overwrite ~/.zshrc?",
    "default": "false"
  }
}

# 3. Claude te pregunta
# "¿Sobreescribir ~/.zshrc? (s/N)"

# 4. Tu respuesta "y" → Claude continúa
claude> dreamcoder apply shell --yes

# 5. Éxito
{
  "status": "success",
  "data": {
    "operations": [{
      "action": "install",
      "status": "success",
      "message": "Module shell installed"
    }]
  }
}
```

### Estados del Protocolo

- `success` - Todo completado
- `requires_input` - Necesita confirmación del usuario
- `failed` - Error que requiere intervención
- `partial` - Algunas operaciones fallaron

### Tipos de Input

- `confirm` - Sí/No
- `input` - Texto libre
- `password` - Contraseña (oculta)
- `select` - Selección de opciones

---

## 🏗️ Arquitectura

```
dreamcoder-manager-rust/
├── crates/
│   ├── core/        # Lógica de negocio pura
│   ├── cli/         # CLI + TUI
│   ├── stow/        # Sistema stow nativo ⭐ NUEVO
│   ├── templates/   # Machine-specific configs
│   └── secrets/     # Encriptación
├── tests/
│   ├── ai_protocol_tests.rs  # ⭐ NUEVO
│   └── integration_tests.rs  # ⭐ NUEVO
└── Cargo.toml
```

### Stow Nativo vs GNU Stow

| Feature | GNU Stow | Dreamcoder Stow |
|---------|----------|-----------------|
| Velocidad | ~100ms | ~10ms (10x) |
| Conflicto detection | Básica | Avanzada con backups auto |
| Cross-platform | ❌ Unix only | ✅ Linux/macOS/Windows |
| Tree merging | ❌ | ✅ Soporta solapamientos |
| API programática | ❌ | ✅ Rust library |

---

## 📝 Templates

### Ejemplo

```handlebars
# ~/.zshrc - Generado por Dreamcoder
export USER="{{user.name}}"
export EMAIL="{{user.email}}"

{{#if_os "linux"}}
export PATH="$HOME/.local/bin:$PATH"
{{/if_os}}

{{#if_os "macos"}}
export PATH="/opt/homebrew/bin:$PATH"
{{/if_os}}

{{#if_exists "~/.cargo/bin"}}
source $HOME/.cargo/env
{{/if_exists}}

# Secretos
export GITHUB_TOKEN={{secret "op://Personal/github/token"}}
```

### Helpers Disponibles

- `{{if_os "linux"}}` - Condicional por OS
- `{{if_arch "aarch64"}}` - Condicional por arquitectura
- `{{if_exists "~/.file"}}` - Condicional por existencia
- `{{secret "path"}}` - Inyección de secretos
- `{{env "VAR"}}` - Variables de entorno

---

## 🔐 Secrets

### Age (Recomendado)

```bash
# Setup
dreamcoder secret setup age

# Encriptar
dreamcoder secret encrypt .ssh/config

# En template
# {{decrypt "~/.ssh/config.age"}}
```

### 1Password

```bash
# Configurar
dreamcoder secret setup 1password

# Service account para CI:
export OP_SERVICE_ACCOUNT_TOKEN="..."
```

---

## 🧪 Testing

### Ejecutar Tests

```bash
# Todos los tests
cargo test --workspace

# Tests específicos
cargo test --package dreamcoder-stow
cargo test --package dreamcoder-cli ai_protocol

# Con output
cargo test -- --nocapture
```

### Tests de Integración

- ✅ **AI Protocol** - Valida estructura JSON y flujo AI
- ✅ **Stow Nativo** - Operaiones symlink, conflictos, backups
- ✅ **Templates** - Rendering con contexto
- ✅ **Core Engine** - Operaciones async

---

## 📊 Benchmarks

| Métrica | Bash v1.0 | Rust v2.0 | Mejora |
|---------|-----------|-----------|--------|
| **Tiempo inicio** | 300ms | 15ms | **20x** |
| **Memoria** | 2MB | 800KB | **2.5x** |
| **Stow operation** | 120ms | 15ms | **8x** |
| **Binary size** | N/A | 4MB | Portable |
| **Tests** | 0 | 50+ | ✅ |
| **AI-ready** | ❌ | ✅ | Protocolo completo |

---

## 🛣️ Roadmap

### v2.0.0 (Actual)
- ✅ Stow nativo completo
- ✅ TUI con Ratatui
- ✅ AI Protocol v1
- ✅ Tests de integración

### v2.1.0 (Próximo)
- 🔄 Gestión de paquetes automática
- 🔄 Auto-update del binary
- 🔄 Plugin system WASM

### v2.2.0 (Futuro)
- 🔄 GUI con Tauri
- 🔄 Nix flakes integration
- 🔄 Windows native

---

## 🤝 Contribuir

```bash
# Setup
git clone https://github.com/dreamcoder08/Dreamcoder_dots.git
cd dreamcoder-manager-rust

# Test
cargo test --workspace

# Lint
cargo clippy --all-targets --all-features

# Format
cargo fmt --all
```

---

## 📄 Licencia

MIT © Dreamcoder

---

<div align="center">

**🎩 Hecho con pasión, café, y Rust.**

*[Instalar](https://get.dreamcoder.dev)* | *[Docs](https://docs.dreamcoder.dev)* | *[Discord](https://discord.gg/dreamcoder)*

</div>

---

## 🌐 Dreamcoder Ecosystem

| Project | Description |
|---------|-------------|
| [Dreamcoder08](https://github.com/Dreamcoder08) | Software Architect · GDE — Profile |
| [Dreamcoder_dots](https://github.com/Dreamcoder08/Dreamcoder_dots) | Arch Linux dotfiles — the Python/SHELL version of this manager |
| [DreamFolio](https://github.com/Dreamcoder08/DreamFolio) | High-performance portfolio — Astro, React, Tailwind |
| [ARKELYTHEX](https://github.com/arkelythex) | Civic, Agri & Legal Tech for Peru |
