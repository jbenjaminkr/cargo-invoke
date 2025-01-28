use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

#[derive(Default)]
struct MermaidClasses {
    definitions: HashMap<String, String>, // classDef name -> props
    assignments: Vec<(Vec<String>, String)>, // (nodes, class_name)
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <directory>", args[0]);
        std::process::exit(1);
    }

    let dir_path = Path::new(&args[1]);
    if !dir_path.is_dir() {
        eprintln!("Error: {} is not a directory", args[1]);
        std::process::exit(1);
    }

    // Start with flowchart TB and a blank line
    let mut merged_content = String::from("flowchart LR\n\n");
    let mut classes = MermaidClasses::default();

    // Read and process all .mermaid files
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("mermaid") {
            let content = fs::read_to_string(&path)?;
            process_mermaid_file(&content, &mut merged_content, &mut classes);
        }
    }

    // Append collected class definitions and assignments
    append_classes(&mut merged_content, &classes);

    // Create output file in the parent directory
    let parent_dir = dir_path.parent().unwrap_or_else(|| Path::new("."));
    let output_filename = parent_dir.join(format!(
        "{}.mermaid",
        dir_path.file_name().unwrap().to_string_lossy()
    ));
    let mut output_file = File::create(&output_filename)?;
    output_file.write_all(merged_content.as_bytes())?;

    println!(
        "Successfully merged Mermaid files into {}",
        output_filename.display()
    );
    Ok(())
}

fn process_mermaid_file(content: &str, merged_content: &mut String, classes: &mut MermaidClasses) {
    for line in content.lines() {
        let trimmed = line.trim();

        // Skip ALL lines starting with "flowchart"
        if trimmed.starts_with("flowchart") {
            continue;
        }

        if trimmed.starts_with("classDef ") {
            // Handle class definition
            let parts: Vec<&str> = trimmed["classDef ".len()..].splitn(2, ' ').collect();
            if parts.len() == 2 {
                classes
                    .definitions
                    .insert(parts[0].to_string(), parts[1].to_string());
            }
        } else if trimmed.starts_with("class ") {
            // Handle class assignment
            let parts: Vec<&str> = trimmed["class ".len()..].splitn(2, ' ').collect();
            if parts.len() == 2 {
                let nodes: Vec<String> =
                    parts[0].split(',').map(|s| s.trim().to_string()).collect();
                classes.assignments.push((nodes, parts[1].to_string()));
            }
        } else if !trimmed.is_empty() {
            // Add non-empty, non-class-related lines to merged content
            merged_content.push_str(line);
            merged_content.push('\n');
        }
    }
}

fn append_classes(merged_content: &mut String, classes: &MermaidClasses) {
    merged_content.push('\n');

    // Add class definitions
    for (name, props) in &classes.definitions {
        merged_content.push_str(&format!("classDef {} {}\n", name, props));
    }

    // Add class assignments
    for (nodes, class_name) in &classes.assignments {
        merged_content.push_str(&format!("class {} {}\n", nodes.join(","), class_name));
    }
}
