use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, PartialEq)]
enum DiagramType {
    Default,
    Class,
}

/// Checks if mmdc is installed on the system
fn check_mmdc_installed() -> bool {
    Command::new("mmdc").arg("--version").output().is_ok()
}

/// Installs mmdc using npm if it's not already installed
fn install_mmdc() -> Result<(), Box<dyn Error>> {
    println!("Installing mermaid-cli...");

    let npm_result = Command::new("npm")
        .args(["install", "-g", "@mermaid-js/mermaid-cli"])
        .output()?;

    if !npm_result.status.success() {
        return Err("Failed to install mermaid-cli using npm".into());
    }

    println!("Successfully installed mermaid-cli");
    Ok(())
}

fn convert_to_format(
    input_path: &Path,
    input_file: &String,
    diagram_type: DiagramType,
    output_format: &str,
) -> Result<(), Box<dyn Error>> {
    if !input_path.exists() {
        return Err(format!("Input file {:?} does not exist", input_path).into());
    }

    if input_path.extension().and_then(|ext| ext.to_str()) != Some("mermaid") {
        return Err("Input file must be a .mermaid file".into());
    }

    let mut content = fs::read_to_string(input_path)?;

    match diagram_type {
        DiagramType::Class => {
            content.push_str("\n%%{init: {'theme': 'base', 'themeVariables': { 'primaryColor': '#fff4dd', 'fontSize': '16px' }}}%%\n");
        }
        DiagramType::Default => {
            content.push_str("\n%%{init: {'theme': 'default' }}%%\n");
        }
    }

    let temp_path = input_path.with_extension("temp.mmd");
    fs::write(&temp_path, content)?;

    let mut output_path = PathBuf::from("visuals");
    fs::create_dir_all(&output_path)?;
    output_path.push(input_file);
    output_path.set_extension(output_format);

    let mut command = Command::new("mmdc");
    command.args([
        "-i",
        temp_path.to_str().unwrap(),
        "-o",
        output_path.to_str().unwrap(),
    ]);

    let config_path = if diagram_type == DiagramType::Default {
        PathBuf::from("assets/default.config.json")
    } else {
        PathBuf::from("assets/mermaid.config.json")
    };

    if config_path.exists() {
        command.args(["--configFile", config_path.to_str().unwrap()]);
    }

    let css_path = PathBuf::from("assets/mermaid.css");
    if css_path.exists() {
        command.args(["--cssFile", css_path.to_str().unwrap()]);
    }

    let output = command.output()?;

    fs::remove_file(&temp_path)?;

    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to convert file: {}", error_message).into());
    }

    println!(
        "Successfully converted {:?} to {:?}",
        input_path, output_path
    );

    #[cfg(target_os = "macos")]
    Command::new("open")
        .arg("-a")
        .arg("Google Chrome")
        .arg(&output_path)
        .spawn()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 || args.len() > 4 {
        eprintln!(
            "Usage: {} <input_file_without_extension> [-C] [--png]",
            args[0]
        );
        eprintln!("  -C      Generate class diagram with specific styling");
        eprintln!("  --png   Output as PNG instead of SVG");
        std::process::exit(1);
    }

    if !check_mmdc_installed() {
        println!("mermaid-cli not found. Attempting to install...");
        install_mmdc()?;
    }

    let diagram_type = if args.contains(&"-C".to_string()) {
        DiagramType::Class
    } else {
        DiagramType::Default
    };

    let output_format = if args.contains(&"--png".to_string()) {
        "png"
    } else {
        "svg"
    };

    let input_file = if args[1] == "-C" || args[1] == "--png" {
        &args[2]
    } else {
        &args[1]
    };

    let mut input_path = PathBuf::from("diagrams");
    input_path.push(input_file);
    input_path.set_extension("mermaid");

    match convert_to_format(&input_path, input_file, diagram_type, output_format) {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
