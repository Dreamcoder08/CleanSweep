use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
#[command(name = "dreamcoder")]
#[command(about = "🎩 Dreamcoder Dotfiles Manager - AI-First Edition")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Modo no-interactivo (para CI/CD y AI)
    #[arg(long, global = true, env = "DREAMCODER_NON_INTERACTIVE")]
    pub non_interactive: bool,

    /// Output en formato JSON (AI-First Protocol)
    #[arg(long, global = true, env = "DREAMCODER_JSON")]
    pub json: bool,

    /// Auto-confirmar todas las preguntas
    #[arg(long, global = true, env = "DREAMCODER_YES")]
    pub yes: bool,

    /// Directorio de dotfiles (override)
    #[arg(short, long, global = true, env = "DREAMCODER_ROOT")]
    pub root: Option<PathBuf>,

    /// Nivel de verbosity
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    /// Aplica configuraciones (instala/actualiza módulos)
    #[command(alias = "sync")]
    Apply(ApplyArgs),

    /// Detecta módulos disponibles
    #[command(alias = "ls")]
    List(ListArgs),

    /// Muestra estado actual
    #[command(alias = "st")]
    Status(StatusArgs),

    /// Gestión de backups
    #[command(subcommand)]
    Backup(BackupCommands),

    /// Gestión de secretos
    #[command(subcommand)]
    Secret(SecretCommands),

    /// Instala dependencias del sistema
    #[command(alias = "deps")]
    InstallDeps(InstallDepsArgs),

    /// Actualiza el manager
    #[command(alias = "upgrade")]
    Update(UpdateArgs),

    /// Inicializa un nuevo repositorio de dotfiles
    Init(InitArgs),

    /// Modo interactivo TUI
    Interactive,
}

#[derive(Debug, Clone, Args)]
pub struct ApplyArgs {
    /// Módulos específicos a instalar (default: todos)
    #[arg(value_name = "MODULE")]
    pub modules: Vec<String>,

    /// Simula cambios sin aplicarlos (dry-run)
    #[arg(long)]
    pub dry_run: bool,

    /// Omite backup antes de aplicar
    #[arg(long)]
    pub no_backup: bool,

    /// Omite hooks post-instalación
    #[arg(long)]
    pub no_hooks: bool,

    /// Forza sobreescritura de archivos existentes
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Debug, Clone, Args)]
pub struct ListArgs {
    /// Muestra también módulos ya instalados
    #[arg(long)]
    pub installed: bool,

    /// Output formato tabla
    #[arg(long)]
    pub table: bool,
}

#[derive(Debug, Clone, Args)]
pub struct StatusArgs {
    /// Muestra información detallada
    #[arg(short, long)]
    pub detailed: bool,
}

#[derive(Debug, Clone, Subcommand)]
pub enum BackupCommands {
    /// Crea un nuevo backup
    Create {
        /// Fuerza backup aunque no haya cambios
        #[arg(short, long)]
        force: bool,
    },

    /// Lista backups existentes
    List {
        /// Muestra tamaño de cada backup
        #[arg(long)]
        size: bool,
    },

    /// Restaura un backup
    Restore {
        /// Nombre del backup o "latest"
        #[arg(value_name = "BACKUP")]
        backup: String,
    },

    /// Elimina backups antiguos
    Clean {
        /// Mantener solo los N más recientes
        #[arg(short, long, default_value = "5")]
        keep: usize,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum SecretCommands {
    /// Encripta un archivo
    Encrypt {
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /// Desencripta un archivo
    Decrypt {
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },

    /// Configura el proveedor de secretos
    Setup {
        #[arg(value_enum)]
        provider: SecretProviderArg,
    },
}

#[derive(Debug, Clone, clap::ValueEnum)]
pub enum SecretProviderArg {
    Age,
    Sops,
    Onepassword,
    Bitwarden,
}

#[derive(Debug, Clone, Args)]
pub struct InstallDepsArgs {
    /// Instala solo dependencias del módulo especificado
    #[arg(value_name = "MODULE")]
    pub module: Option<String>,

    /// Muestra qué instalaría sin instalar
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Args)]
pub struct UpdateArgs {
    /// Actualiza a versión específica (default: latest)
    #[arg(value_name = "VERSION")]
    pub version: Option<String>,

    /// Verifica si hay actualizaciones sin aplicarlas
    #[arg(long)]
    pub check: bool,
}

#[derive(Debug, Clone, Args)]
pub struct InitArgs {
    /// Directorio donde inicializar (default: ~/Dreamcoder_dots)
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,

    /// URL del repositorio git remoto
    #[arg(short, long)]
    pub remote: Option<String>,

    /// Incluye templates de ejemplo
    #[arg(long)]
    pub with_examples: bool,
}
