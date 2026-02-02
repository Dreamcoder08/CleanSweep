use dreamcoder_core::{Config, DreamcoderEngine};
use std::io::{stdout, Write};

pub async fn run() -> anyhow::Result<()> {
    println!("🎩 Dreamcoder Interactive Mode");
    println!("================================\n");
    
    let config = Config::load().unwrap_or_default();
    let engine = DreamcoderEngine::new(config)?;
    
    let modules = engine.detect_modules()?;
    
    // Simple TUI loop
    loop {
        println!("\nMain Menu:");
        println!("  [1] List modules");
        println!("  [2] Apply all");
        println!("  [3] Backup");
        println!("  [4] Status");
        println!("  [q] Quit");
        
        print!("\nSelect: ");
        stdout().flush()?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        match input.trim() {
            "1" => {
                println!("\nAvailable modules:");
                for (i, m) in modules.iter().enumerate() {
                    println!("  {}. {}", i + 1, m.name);
                }
            }
            "2" => {
                println!("\nApplying all modules...");
                // TODO: Implement apply with progress
            }
            "3" => {
                println!("\nCreating backup...");
                // TODO: Implement backup
            }
            "4" => {
                println!("\nStatus...");
                // TODO: Show status
            }
            "q" | "Q" => break,
            _ => println!("Invalid option"),
        }
    }
    
    println!("\nGoodbye! 👋");
    Ok(())
}
