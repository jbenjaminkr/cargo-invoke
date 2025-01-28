use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct CommandsManifest {
    commands: HashMap<String, CommandInfo>,
}

#[derive(Debug, Deserialize, Clone)]
struct CommandInfo {
    description: String,
    usage: String,
    examples: Vec<String>,
    parameters: Vec<Parameter>,
}

#[derive(Debug, Deserialize, Clone)]
struct Parameter {
    name: String,
    description: String,
}

fn bin_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/bin")
}

fn load_commands_manifest() -> Result<CommandsManifest, Box<dyn Error>> {
    let manifest_path = bin_dir().join("commands.toml");
    let manifest_content = fs::read_to_string(&manifest_path)?;
    let manifest: CommandsManifest = toml::from_str(&manifest_content)?;
    Ok(manifest)
}

fn get_available_commands(manifest: &CommandsManifest) -> Vec<(String, CommandInfo)> {
    let mut commands = Vec::new();
    for (cmd_name, cmd_info) in &manifest.commands {
        let cmd_path = bin_dir().join(format!("{}.rs", cmd_name));
        if cmd_path.exists() {
            commands.push((cmd_name.clone(), cmd_info.clone()));
        } else {
            eprintln!("Warning: No script found for command '{}'.", cmd_name);
        }
    }
    commands.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    commands
}

fn show_help(manifest: &CommandsManifest) {
    println!("Usage:");
    println!("  cargo invoke <command> [args...]  # Run a specific command");
    println!("  cargo invoke --help                # Show this help message\n");

    let commands = get_available_commands(manifest);
    if commands.is_empty() {
        println!("No commands found in src/bin directory.");
        return;
    }

    println!("Available commands:");
    for (cmd, info) in &commands {
        println!("  {:<15} {}", cmd, info.description);
    }

    println!("\nUse 'cargo invoke <command> --help' for more information on a specific command.");
}

fn show_command_help(command: &str, info: &CommandInfo) {
    println!("Usage:\n  {}\n", info.usage);
    println!("Description:\n  {}\n", info.description);

    if !info.parameters.is_empty() {
        println!("Parameters:");
        for param in &info.parameters {
            println!("  {:<20} {}", param.name, param.description);
        }
        println!();
    }

    if !info.examples.is_empty() {
        println!("Examples:");
        for example in &info.examples {
            println!("  {}", example);
        }
        println!();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let manifest = load_commands_manifest()?;
    let original_args: Vec<String> = env::args().collect();
    let (invoke_subcommand, args) = if original_args.len() > 1 && original_args[1] == "invoke" {
        (
            true,
            original_args
                .iter()
                .skip(2)
                .cloned()
                .collect::<Vec<String>>(),
        )
    } else {
        (
            false,
            original_args
                .iter()
                .skip(1)
                .cloned()
                .collect::<Vec<String>>(),
        )
    };

    if args.is_empty() {
        show_help(&manifest);
        return Ok(());
    }

    let command = &args[0];
    match command.as_str() {
        "--help" | "-h" => {
            show_help(&manifest);
        }
        _ => {
            let commands = get_available_commands(&manifest);
            if let Some((_, cmd_info)) = commands.iter().find(|(cmd, _)| cmd == command) {
                if args.len() > 1 && (args[1] == "--help" || args[1] == "-h") {
                    show_command_help(command, cmd_info);
                    return Ok(());
                }

                let status = Command::new(command).args(&args[1..]).status()?;

                if !status.success() {
                    return Err(format!(
                        "Command '{}' failed with exit code {:?}",
                        command,
                        status.code()
                    )
                    .into());
                }
            } else {
                println!("Unknown command: {}", command);
                show_help(&manifest);
            }
        }
    }

    Ok(())
}
