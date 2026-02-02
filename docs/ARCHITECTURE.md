# 🏗️ Dreamcoder Architecture - AI-First Design

## Resumen Ejecutivo

Dreamcoder v2.0 es un dotfiles manager reimaginado para la era de la IA. A diferencia de los gestores tradicionales (Stow, yadm, bare git), está diseñado desde cero para ser **usado por humanos Y por inteligencia artificial**.

**Filosofía Central:** *"Si un humano puede hacerlo, Claude (u otra IA) debería poder hacerlo también - sin hacks, sin parsing de texto, mediante un protocolo estructurado."*

---

## 🎯 Por Qué Rust (y no Go, Python o Bash)

| Criterio | Bash | Python | Go | **Rust** |
|----------|------|--------|-----|----------|
| **Single Binary** | ❌ | ❌ | ✅ | ✅ |
| **Runtime** | Necesita shell | Necesita Python | ~2MB | **~800KB** |
| **Performance** | Lento | Medio | Rápido | **Máxima** |
| **Memoria** | Variable | Alta | Media | **Baja** |
| **Seguridad** | ❌ | ⚠️ | ⚠️ | **✅ Memory-safe** |
| **Cross-compile** | N/A | Difícil | Fácil | **Fácil** |
| **CLI Libraries** | N/A | Click/Rich | Cobra/Bubble | **Clap/Ratatui** |
| **AI Protocol** | Imposible | Lento | OK | **Perfecto** |

**Decisión:** Rust ofrece el mejor balance de portability, performance y capacidad de expresión de tipos para un protocolo AI robusto.

---

## 🧬 Arquitectura de Capas

```
┌─────────────────────────────────────────────────────────────────┐
│                      LAYER 1: PRESENTATION                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐           │
│  │   CLI Args   │  │  TUI/Visual  │  │ AI Protocol  │           │
│  │   (Clap)     │  │  (Ratatui)   │  │  (JSON-RPC)  │           │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘           │
└─────────┼─────────────────┼─────────────────┼───────────────────┘
          │                 │                 │
          └─────────────────┼─────────────────┘
                            │
┌───────────────────────────┼─────────────────────────────────────┐
│                   LAYER 2: APPLICATION                          │
│  ┌───────────────────────┐  ┌───────────────────────────────┐  │
│  │   Command Handlers    │  │   AI Protocol Engine          │  │
│  │   • Apply             │  │   • Structured Output         │  │
│  │   • Backup            │  │   • Input Requirements        │  │
│  │   • Status            │  │   • State Machine             │  │
│  └───────────┬───────────┘  └───────────────┬───────────────┘  │
└──────────────┼──────────────────────────────┼───────────────────┘
               │                              │
┌──────────────┼──────────────────────────────┼───────────────────┐
│              │         LAYER 3: DOMAIN      │                   │
│  ┌───────────┴──────────┐  ┌────────────────┴──────────────┐   │
│  │   Module System      │  │   Template Engine             │   │
│  │   • Detection        │  │   • Handlebars                │   │
│  │   • Installation     │  │   • Context Resolution        │   │
│  │   • Conflict Check   │  │   • OS-specific blocks        │   │
│  └───────────┬──────────┘  └────────────────┬──────────────┘   │
│              │                              │                  │
│  ┌───────────┴──────────┐  ┌────────────────┴──────────────┐   │
│  │   Backup System      │  │   Secret Management           │   │
│  │   • Change Detection │  │   • Age (offline)             │   │
│  │   • Rotation         │  │   • 1Password API             │   │
│  │   • Restore          │  │   • Bitwarden API             │   │
│  └───────────┬──────────┘  └────────────────┬──────────────┘   │
└──────────────┼──────────────────────────────┼───────────────────┘
               │                              │
┌──────────────┼──────────────────────────────┼───────────────────┐
│              │     LAYER 4: INFRASTRUCTURE  │                   │
│  ┌───────────┴──────────┐  ┌────────────────┴──────────────┐   │
│  │   Filesystem         │  │   State Persistence           │   │
│  │   • Symlinks         │  │   • JSON State                │   │
│  │   • Atomic ops       │  │   • Checksums                 │   │
│  │   • Permissions      │  │   • History                   │   │
│  └──────────────────────┘  └───────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 🤖 AI Protocol - Especificación Técnica

### Concepto Central

En lugar de output de texto libre, Dreamcoder emite **estructuras JSON** que describen:
1. Qué operaciones se realizaron
2. Cuál es el estado actual
3. Qué decisiones requieren input del usuario

### Estados de Operación

```rust
enum Status {
    Pending,      // Esperando ejecutarse
    Running,      // En progreso
    Success,      // Completado OK
    Failed,       // Error
    Skipped,      // Omitido (no hay cambios)
    PendingInput, // Requiere decisión del usuario
}
```

### Flujo AI-First

#### Escenario 1: Aplicación Exitosa
```bash
$ dreamcoder apply --json
```

```json
{
  "version": "2.0.0",
  "request_id": "uuid-1234",
  "status": "success",
  "timestamp": "2025-02-02T15:30:00Z",
  "data": {
    "operations": [
      {
        "id": "op-001",
        "action": "backup",
        "status": "success",
        "message": "Backup created at backup-20250202-153000",
        "metadata": {
          "path": "/home/user/.config/dreamcoder-backups/backup-20250202-153000",
          "size": 2457600,
          "files": 42
        }
      },
      {
        "id": "op-002",
        "action": "install",
        "status": "success",
        "message": "Module DreamcoderShell installed",
        "metadata": {
          "module": "DreamcoderShell",
          "symlinks_created": 4
        }
      }
    ]
  },
  "requires_action": null
}
```

**Claude puede:** Confirmar al usuario que todo salió bien.

---

#### Escenario 2: Conflicto Detectado
```bash
$ dreamcoder apply DreamcoderShell --json
```

```json
{
  "status": "requires_input",
  "data": {
    "operations": [
      {
        "id": "op-003",
        "action": "install",
        "status": "pending_input",
        "message": "Conflicts detected",
        "metadata": {
          "module": "DreamcoderShell",
          "conflicts": ["~/.zshrc", "~/.bashrc"]
        },
        "requires_input": {
          "input_type": "confirm",
          "message": "Overwrite existing files: ~/.zshrc, ~/.bashrc?",
          "default": "false"
        }
      }
    ]
  },
  "requires_action": {
    "action_type": "confirm",
    "message": "Overwrite existing files: ~/.zshrc, ~/.bashrc?",
    "default": "false"
  }
}
```

**Claude puede:**
1. Leer el conflicto
2. Presentarte la opción: *"¿Sobreescribir ~/.zshrc y ~/.bashrc? (s/N)"*
3. Si respondes "S", ejecutar: `dreamcoder apply DreamcoderShell --yes`
4. Si respondes "N", ofrecer alternativas (merge, skip, backup)

---

#### Escenario 3: Input Requerido (Password)
```json
{
  "status": "requires_input",
  "requires_action": {
    "action_type": "password",
    "message": "Unlock 1Password vault to access SSH keys",
    "default": null
  }
}
```

**Claude puede:**
- Si estás presente: Pedirte la contraseña
- Si es CI/CD: Fallar con mensaje claro de que se necesita `OP_SERVICE_ACCOUNT_TOKEN`

---

## 🧩 Sistema de Módulos

### Detección Automática

```rust
// Busca directorios que coincidan con el patrón
let modules = engine.discover_modules()?;
// → [DreamcoderShell, DreamcoderNvim, DreamcoderTmux, ...]
```

### Estructura de Módulo

```
DreamcoderShell/
├── .zshrc                          # Archivo base (copiado/symlink)
├── .zshrc.tmpl                     # Template (si necesita variables)
├── .dreamcoder.toml                # Metadata y configuración
├── .hooks/
│   ├── pre-install.sh             # Validaciones/preparación
│   └── post-install.sh            # Configuración post-instalación
└── packages.toml                  # Dependencias por OS
```

### Metadata (.dreamcoder.toml)

```toml
[module]
name = "DreamcoderShell"
description = "Zsh configuration with custom prompts"
version = "2.0.0"

[packages]
arch = ["zsh", "zsh-completions", "zsh-syntax-highlighting"]
macos = ["zsh", "zsh-completions"]
ubuntu = ["zsh", "zsh-autosuggestions"]

[os.linux]
path = "linux/"  # Override específico para Linux

[os.macos]
path = "macos/"  # Override específico para macOS

[hooks]
pre_install = "scripts/check-zsh.sh"
post_install = "scripts/set-default-shell.sh"
```

---

## 🎨 Sistema de Templates

### Contexto Disponible

```handlebars
# {{module.name}} - Template
# User: {{user.name}} <{{user.email}}>
# Generated: {{timestamp}}

{{#if_os "linux"}}
# Linux-specific configuration
export PATH="$HOME/.local/bin:$PATH"
{{/if_os}}

{{#if_os "macos"}}
# macOS-specific configuration
export PATH="/opt/homebrew/bin:$PATH"
export HOMEBREW_NO_AUTO_UPDATE=1
{{/if_os}}

{{#if_exists "~/.cargo/bin"}}
# Rust environment
source $HOME/.cargo/env
{{/if_exists}}

# Secrets (encriptados)
export GITHUB_TOKEN={{secret "op://Personal/github/token"}}
```

### Helpers Built-in

- `{{if_os "linux"}}` - Contenido condicional por OS
- `{{if_arch "aarch64"}}` - Contenido condicional por arquitectura
- `{{if_exists "~/.file"}}` - Contenido condicional por existencia de archivo
- `{{secret "path"}}` - Inyección de secretos desde provider configurado
- `{{env "VARNAME"}}` - Variables de entorno

---

## 🔐 Sistema de Secrets

### Providers

1. **Age (Recomendado)**
   - 100% offline
   - Criptografía moderna (X25519)
   - Sin dependencias de terceros

2. **1Password**
   - Integración nativa con 1Password CLI
   - Service Accounts para CI/CD
   - Biometric unlock en desktop

3. **Bitwarden**
   - Similar a 1P
   - Opción open-source

### Workflow

```bash
# 1. Setup inicial
dreamcoder secret setup age
> Generating age keypair...
> Public key: age1ql3z7... (add to recipients file)

# 2. Encriptar archivo sensible
dreamcoder secret encrypt .ssh/config
> Created: .ssh/config.age
> Original removed securely

# 3. En template:
# {{decrypt "~/.ssh/config.age"}}

# 4. Al aplicar:
# - Se desencripta temporalmente
# - Se usa para crear symlink
# - Se limpia del filesystem
```

---

## 📦 Gestión de Paquetes

### Detección Automática de OS

```rust
enum PackageManager {
    Pacman,  // Arch
    Apt,     // Debian/Ubuntu
    Brew,    // macOS
    Dnf,     // Fedora
    Zypper,  // openSUSE
    Nix,     // NixOS
    Scoop,   // Windows
}
```

### Declaración de Dependencias

```toml
# packages.toml en módulo
[packages]
# Detectado automáticamente según OS
common = ["git", "curl"]

[packages.pacman]
packages = ["zsh", "zsh-completions", "fzf"]
post_install = "sudo chsh -s $(which zsh)"

[packages.brew]
taps = ["homebrew/cask-fonts"]
packages = ["zsh", "fzf", "font-meslo-lg-nerd-font"]
casks = ["kitty"]

[packages.apt]
packages = ["zsh", "fzf"]
ppa = ["ppa:agornostal/ulauncher"]
```

### Integración con Apply

```bash
$ dreamcoder apply DreamcoderShell
# 1. Detecta que estás en Arch
# 2. Verifica si zsh está instalado → No
# 3. Pregunta: "Install missing packages: zsh, zsh-completions?"
# 4. Ejecuta: sudo pacman -S zsh zsh-completions
# 5. Continúa con instalación del módulo
```

---

## 🔄 Backup y Estado

### Detección de Cambios (Hash-based)

```rust
// Calcula hash de todos los archivos gestionados
let current_hash = backup.calculate_state_hash()?;
let last_hash = fs::read_to_string(".last-hash")?;

if current_hash != last_hash {
    // Hay cambios, crear backup
    backup.create()?;
}
```

### Rotación Automática

- Mantiene máximo N backups (default: 5)
- Elimina automáticamente los más antiguos
- Compresión opcional con zstd

### Restore

```bash
# Listar disponibles
dreamcoder backup list
# 1. backup-20250202-153000 (2.4MB)
# 2. backup-20250115-090000 (2.1MB)

# Restaurar último
dreamcoder backup restore latest

# Restaurar específico
dreamcoder backup restore backup-20250115-090000
```

---

## 🚀 Compilación y Distribución

### Desarrollo

```bash
# Build debug
cargo build

# Build release (optimizado)
cargo build --release

# Tests
cargo test --workspace

# Check
cargo clippy --all-targets
```

### Release

```bash
# Cross-compilation
# Linux x86_64
cargo build --release --target x86_64-unknown-linux-musl

# macOS x86_64
cargo build --release --target x86_64-apple-darwin

# macOS ARM
cargo build --release --target aarch64-apple-darwin

# Windows
cargo build --release --target x86_64-pc-windows-gnu
```

### Tamaños Binarios (estimados)

| Plataforma | Tamaño | Comprimido |
|------------|--------|------------|
| Linux x64  | 4.2MB  | 1.8MB      |
| macOS ARM  | 3.8MB  | 1.6MB      |
| Windows    | 4.5MB  | 2.0MB      |

---

## 🎓 Mejores Prácticas

### Para Usuarios Humanos

1. **Empieza simple**: `dreamcoder init` + `dreamcoder apply`
2. **Usa templates solo cuando sea necesario**: No over-engineering
3. **Secrets siempre encriptados**: Nunca commitear keys
4. **Backup antes de cambios grandes**: `dreamcoder backup create --force`

### Para Integración AI

1. **Siempre usar `--json`**: Output parseable garantizado
2. **Modo non-interactive en CI**: `DREAMCODER_NON_INTERACTIVE=1`
3. **Service accounts para secrets**: No depender de UI de 1P
4. **Dry-run primero**: `--dry-run` para validar cambios

---

## 🔮 Roadmap Técnico

### v2.0 (Actual)
- ✅ Core en Rust
- ✅ AI Protocol v1
- ✅ Templates básicos
- ✅ Age secrets
- ✅ Stow nativo

### v2.1 (Q2 2025)
- 🔄 TUI completo (Ratatui)
- 🔄 AI Protocol v2 (streaming)
- 🔄 Plugin system (WASM)
- 🔄 Cloud sync (S3/Backblaze)

### v2.2 (Q4 2025)
- 🔄 GUI nativa (Tauri)
- 🔄 Nix flakes integration
- 🔄 Windows native (no WSL)

---

## 📊 Comparativa con Alternativas

| Feature | Stow | Chezmoi | Yadm | **Dreamcoder** |
|---------|------|---------|------|----------------|
| Single binary | ❌ | ✅ | ❌ | **✅** |
| AI Protocol | ❌ | ❌ | ❌ | **✅** |
| Templates | ❌ | ✅ | ⚠️ | **✅** |
| Secrets | ❌ | ✅ | ❌ | **✅** |
| Auto-packages | ❌ | ❌ | ❌ | **✅** |
| Native symlinks | ❌ | ✅* | ❌ | **✅** |
| Cross-platform | ⚠️ | ✅ | ⚠️ | **✅** |

*Chezmoi usa archivos reales, no symlinks

---

**Conclusión:** Dreamcoder representa la evolución natural de los dotfiles managers: de herramientas para organizar archivos a **plataformas de gestión de entorno de desarrollo** integradas con el flujo de trabajo moderno (AI-assisted).

---

*Documento v1.0 - Dreamcoder Architecture Team*
