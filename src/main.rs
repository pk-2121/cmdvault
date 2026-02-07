use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser)]
#[command(name = "cmdvault")]
#[command(about = "Project-specific command manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Add {
        name: String,
        command: String,
    },
    List,
    Run {
        name: String,
    },
    Remove {
        name: String,
    },
}

#[derive(Serialize, Deserialize)]
struct Vault {
    commands: HashMap<String, String>,
}

fn vault_path() -> PathBuf {
    let cwd = std::env::current_dir().unwrap();
    let hash = format!("{:x}", md5::compute(cwd.to_string_lossy().as_bytes()));

    let mut path = dirs::home_dir().unwrap();
    path.push(".cmdvault");

    fs::create_dir_all(&path).unwrap();

    path.push(format!("{}.json", hash));
    path
}

fn load_vault() -> Vault {
    let path = vault_path();
    if path.exists() {
        let data = fs::read_to_string(path).unwrap();
        serde_json::from_str(&data).unwrap()
    } else {
        Vault {
            commands: HashMap::new(),
        }
    }
}

fn save_vault(vault: &Vault) {
    let path = vault_path();
    let data = serde_json::to_string_pretty(vault).unwrap();
    fs::write(path, data).unwrap();
}

fn main() {
    let cli = Cli::parse();
    let mut vault = load_vault();

    match cli.command {
        Commands::Add { name, command } => {
            vault.commands.insert(name.clone(), command);
            save_vault(&vault);
            println!("Command '{}' added.", name);
        }

        Commands::List => {
            for (name, cmd) in vault.commands {
                println!("{} → {}", name, cmd);
            }
        }

        Commands::Run { name } => {
            if let Some(cmd) = vault.commands.get(&name) {
                #[cfg(target_os = "windows")]
                {
                    Command::new("cmd")
                        .arg("/C")
                        .arg(cmd)
                        .status()
                        .unwrap();
                }

                #[cfg(not(target_os = "windows"))]
                {
                    Command::new("sh")
                        .arg("-c")
                        .arg(cmd)
                        .status()
                        .unwrap();
                }
            } else {
                println!("Command not found.");
            }
        }

        Commands::Remove { name } => {
            vault.commands.remove(&name);
            save_vault(&vault);
            println!("Command '{}' removed.", name);
        }
    }
}
