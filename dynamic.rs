use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
struct Script {
    name: String,
    path: PathBuf,
}

fn find_scripts(dir: &Path) -> Vec<Script> {
    let mut scripts = Vec::new();

    // Check both root and src/bin directories
    let search_paths = vec![dir.to_path_buf(), dir.join("src").join("bin")];

    for search_path in search_paths {
        if let Ok(entries) = fs::read_dir(search_path) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "rs") {
                    // Skip main.rs and lib.rs
                    if let Some(file_name) = path.file_stem() {
                        if file_name != "main" && file_name != "lib" {
                            let name = file_name.to_string_lossy().into_owned();
                            scripts.push(Script {
                                name,
                                path: path.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    scripts.sort_by(|a, b| a.name.cmp(&b.name));
    scripts
}

fn find_deps_path(current_dir: &Path) -> PathBuf {
    // First check if we're in a workspace by looking for Cargo.toml with [workspace]
    let mut dir = current_dir.to_path_buf();
    while dir.parent().is_some() {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(contents) = fs::read_to_string(&cargo_toml) {
                if contents.contains("[workspace]") {
                    // Found workspace root
                    return dir.join("target/debug/deps");
                }
            }
        }
        dir = dir.parent().unwrap().to_path_buf();
    }

    // If no workspace found, use the current directory
    current_dir.join("target/debug/deps")
}

fn compile_script(script: &Script) -> Result<(), Box<dyn Error>> {
    println!("Compiling {}...", script.name);

    // Use cargo build --bin instead of rustc
    let status = Command::new("cargo")
        .args(&["build", "--bin", &script.name])
        .status()?;

    if !status.success() {
        return Err(format!("Failed to compile {}", script.name).into());
    }

    Ok(())
}

fn run_script(script: &Script, args: &[String]) -> Result<(), Box<dyn Error>> {
    println!("Running {}...", script.name);

    let status = Command::new("cargo")
        .args(&["run", "--bin", &script.name, "--"])
        .args(args)
        .status()?;

    if !status.success() {
        return Err(format!(
            "Script {} failed with exit code {:?}",
            script.name,
            status.code()
        )
        .into());
    }

    Ok(())
}
fn show_help(current_dir: &Path) {
    println!("Usage:");
    println!("  cargo invoke <script> [args...]  # Run a specific script");
    println!("  cargo invoke --help             # Show this help message\n");

    let scripts = find_scripts(current_dir);
    if scripts.is_empty() {
        println!("No Rust scripts found");
        return;
    }

    println!("Available scripts:");
    for script in scripts {
        println!("  {}", script.name);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();

    // Skip the first argument (executable name)
    let args: Vec<String> = if args.len() > 1 && args[1] == "invoke" {
        // If called as `cargo invoke`, skip first two args
        args.iter().skip(2).cloned().collect()
    } else {
        // If called directly, skip first arg
        args.iter().skip(1).cloned().collect()
    };

    let current_dir = std::env::current_dir()?;

    // Create target/debug directory if it doesn't exist
    fs::create_dir_all("target/debug")?;

    if args.is_empty() {
        show_help(&current_dir);
        return Ok(());
    }

    let command = &args[0];

    match command.as_str() {
        "--help" | "-h" => {
            show_help(&current_dir);
        }
        _ => {
            // Check if the command is a script name
            let scripts = find_scripts(&current_dir);
            if let Some(script) = scripts.iter().find(|s| s.name == *command) {
                // If it is a script name, run it with any remaining args
                let script_args = args[1..].to_vec();
                compile_script(script)?;
                run_script(script, &script_args)?;
            } else {
                println!("Unknown script: {}", command);
                show_help(&current_dir);
            }
        }
    }

    Ok(())
}
