use dreamcoder_core::{Config, DreamcoderEngine, SystemInfo};
use crate::args::*;
use crate::Cli;
use anyhow::Result;
use colored::*;
use console::Term;
use indicatif::{ProgressBar, ProgressStyle};

pub async fn apply(args: ApplyArgs, cli: Cli) -> Result<()> {
    let config = load_config(&cli).await?;
    let engine = DreamcoderEngine::new(config)?;
    
    // Detectar o usar módulos especificados
    let modules = if args.modules.is_empty() {
        engine.detect_modules()?
    } else {
        let all = engine.detect_modules()?;
        all.into_iter()
            .filter(|m| args.modules.contains(&m.name))
            .collect()
    };
    
    if modules.is_empty() {
        println!("{}", "No modules to install".yellow());
        return Ok(());
    }
    
    // Mostrar plan
    if !cli.non_interactive {
        println!("\n{}", "📦 Installation Plan".bold().blue());
        for (i, m) in modules.iter().enumerate() {
            println!("  {}. {} {}", 
                i + 1,
                "◉".green(),
                m.name.cyan()
            );
            if let Some(desc) = &m.description {
                println!("     {}", desc.dimmed());
            }
        }
        println!();
    }
    
    // Confirmar si no es non-interactive
    if !cli.non_interactive && !cli.yes {
        if !confirm("Apply these changes?")? {
            println!("Cancelled.");
            return Ok(());
        }
    }
    
    // Aplicar con barra de progreso si hay TTY
    let term = Term::stdout();
    let use_progress = term.is_term() && !cli.non_interactive;
    
    let result = if use_progress {
        let pb = ProgressBar::new(modules.len() as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg}")?
            .progress_chars("#>-"));
        
        let options = dreamcoder_core::ApplyOptions {
            modules,
            backup: !args.no_backup,
            dry_run: args.dry_run,
            skip_hooks: args.no_hooks,
        };
        
        let result = engine.apply(options).await?;
        pb.finish_with_message("Done!");
        result
    } else {
        // Modo AI-First: output estructurado
        let options = dreamcoder_core::ApplyOptions {
            modules,
            backup: !args.no_backup,
            dry_run: args.dry_run,
            skip_hooks: args.no_hooks,
        };
        
        let result = engine.apply(options).await?;
        
        if cli.json {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        
        result
    };
    
    // Resumen
    if !cli.json {
        print_summary(&result).await?;
    }
    
    Ok(())
}

pub async fn list(args: ListArgs, cli: Cli) -> Result<()> {
    let config = load_config(&cli).await?;
    let engine = DreamcoderEngine::new(config)?;
    
    let modules = engine.detect_modules()?;
    
    if cli.json {
        println!("{}", serde_json::to_string_pretty(&modules)?);
        return Ok(());
    }
    
    println!("\n{}", "Available Modules".bold().blue());
    println!("{}", "==================".blue());
    
    for m in &modules {
        let status = if args.installed {
            // Check if installed
            "●".green()
        } else {
            "○".white()
        };
        
        println!("  {} {}", status, m.name.cyan().bold());
        
        if let Some(desc) = &m.description {
            println!("    {}", desc);
        }
        
        if !m.symlinks.is_empty() {
            println!("    {} symlinks", m.symlinks.len().to_string().dimmed());
        }
    }
    
    println!("\n{} modules found\n", modules.len().to_string().yellow());
    
    Ok(())
}

pub async fn status(_args: StatusArgs, cli: Cli) -> Result<()> {
    let config = load_config(&cli).await?;
    let sys_info = SystemInfo::detect();
    
    if cli.json {
        let status = serde_json::json!({
            "config": config,
            "system": sys_info,
            "version": env!("CARGO_PKG_VERSION"),
        });
        println!("{}", serde_json::to_string_pretty(&status)?);
        return Ok(());
    }
    
    println!("\n{}", "🎩 Dreamcoder Status".bold().blue());
    println!("{}", "====================".blue());
    
    println!("\n{}", "System".bold());
    println!("  OS:       {}", sys_info.os.cyan());
    println!("  Arch:     {}", sys_info.arch.cyan());
    println!("  Host:     {}", sys_info.hostname.cyan());
    println!("  User:     {}", sys_info.username.cyan());
    
    println!("\n{}", "Configuration".bold());
    println!("  Root:     {}", config.dotfiles_root.display().to_string().cyan());
    println!("  Backups:  {}", config.backup_dir.display().to_string().cyan());
    println!("  Auto:     {}", if config.auto_backup { "✓".green() } else { "✗".red() });
    
    Ok(())
}

pub async fn backup(cmd: BackupCommands, cli: Cli) -> Result<()> {
    use dreamcoder_core::backup::BackupManager;
    
    let config = load_config(&cli).await?;
    let manager = BackupManager::new(&config);
    
    match cmd {
        BackupCommands::Create { force } => {
            if !force && !manager.has_changes().await? {
                println!("{}", "No changes detected. Use --force to backup anyway.".yellow());
                return Ok(());
            }
            
            let path = manager.create().await?;
            
            if cli.json {
                println!("{}", serde_json::json!({
                    "success": true,
                    "path": path,
                }));
            } else {
                println!("{} Backup created at {:?}", "✓".green(), path);
            }
        }
        
        BackupCommands::List { size } => {
            let backups = manager.list().await?;
            
            if cli.json {
                println!("{}", serde_json::to_string_pretty(&backups)?);
                return Ok(());
            }
            
            println!("\n{}", "Backups".bold().blue());
            for (i, b) in backups.iter().enumerate() {
                let size_str = if size {
                    // Calculate size
                    " (calculating...)".dimmed().to_string()
                } else {
                    String::new()
                };
                
                println!("  {}. {}{}", 
                    i + 1,
                    b.path.file_name().unwrap_or_default().to_string_lossy().cyan(),
                    size_str
                );
            }
            
            if backups.is_empty() {
                println!("  No backups found");
            }
        }
        
        _ => {
            println!("Not implemented yet");
        }
    }
    
    Ok(())
}

pub async fn secret(cmd: SecretCommands, _cli: Cli) -> Result<()> {
    match cmd {
        SecretCommands::Setup { provider } => {
            println!("Setting up {:?} as secret provider...", provider);
            // TODO: Implement setup
        }
        _ => {
            println!("Secret commands not implemented yet");
        }
    }
    Ok(())
}

pub async fn install_deps(_args: InstallDepsArgs, _cli: Cli) -> Result<()> {
    println!("Package management not implemented yet");
    Ok(())
}

pub async fn update(_args: UpdateArgs, _cli: Cli) -> Result<()> {
    println!("Auto-update not implemented yet");
    Ok(())
}

pub async fn init(_args: InitArgs, _cli: Cli) -> Result<()> {
    println!("Init command not implemented yet");
    Ok(())
}

// Helpers
async fn load_config(cli: &Cli) -> Result<Config> {
    let mut config = Config::load().unwrap_or_default();
    
    if let Some(root) = &cli.root {
        config.dotfiles_root = root.clone();
    }
    
    if cli.non_interactive {
        config.ai.non_interactive = true;
    }
    
    if cli.yes {
        config.ai.auto_confirm = true;
    }
    
    Ok(config)
}

fn confirm(msg: &str) -> Result<bool> {
    use dialoguer::Confirm;
    
    Ok(Confirm::new()
        .with_prompt(msg)
        .default(true)
        .interact()?)
}

async fn print_summary(result: &dreamcoder_core::OperationResult) -> Result<()> {
    println!("\n{}", "Summary".bold().blue());
    println!("{}", "=======".blue());
    
    let success_count = result.operations.iter().filter(|o| matches!(o.status, dreamcoder_core::Status::Success)).count();
    let failed_count = result.operations.len() - success_count;
    
    if success_count > 0 {
        println!("  {} {} operations successful", "✓".green(), success_count);
    }
    
    if failed_count > 0 {
        println!("  {} {} operations failed", "✗".red(), failed_count);
    }
    
    // Show operations that need input
    let pending: Vec<_> = result.operations.iter()
        .filter(|o| matches!(o.status, dreamcoder_core::Status::PendingInput))
        .collect();
    
    if !pending.is_empty() {
        println!("\n{}", "Pending Input:".yellow().bold());
        for op in pending {
            if let Some(input) = &op.requires_input {
                println!("  - {}: {}", op.action, input.message);
            }
        }
    }
    
    println!();
    Ok(())
}
