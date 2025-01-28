use regex::Regex;
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

#[derive(Debug, Clone)]
struct ClassDefinition {
    name: String,
    properties: String,
}

#[derive(Debug, Clone)]
struct ClassAssignment {
    elements: Vec<String>,
    class_name: String,
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input_file.mermaid>", args[0]);
        std::process::exit(1);
    }

    let input_path = Path::new(&args[1]);
    if !input_path.exists() {
        eprintln!("Input file does not exist");
        std::process::exit(1);
    }

    let content = fs::read_to_string(input_path)?;
    let cleaned_content = clean_mermaid_file(&content)?;

    // Write to new file with "_clean" suffix
    let output_path = input_path.with_file_name(format!(
        "{}_clean.mermaid",
        input_path.file_stem().unwrap().to_str().unwrap()
    ));
    let mut file = File::create(output_path)?;
    file.write_all(cleaned_content.as_bytes())?;

    Ok(())
}

fn clean_mermaid_file(content: &str) -> io::Result<String> {
    let mut output = String::new();
    let mut nodes = HashSet::new();

    // Extract components
    let class_defs = extract_all_class_definitions(content);
    let class_assignments = extract_all_class_assignments(content);
    let clean_content = extract_clean_content(content, &mut nodes);

    // Build output
    output.push_str("flowchart TB\n\n");

    // Add cleaned diagram content
    output.push_str(&clean_content);
    output.push_str("\n");

    // Add sorted class definitions
    let mut sorted_defs: Vec<_> = class_defs.iter().collect();
    sorted_defs.sort_by(|a, b| a.name.cmp(&b.name));

    for def in sorted_defs {
        output.push_str(&format!("classDef {} {}\n", def.name, def.properties));
    }

    // Add sorted class assignments
    let mut sorted_assignments = class_assignments.clone();
    sorted_assignments.sort_by(|a, b| {
        a.elements
            .join(",")
            .cmp(&b.elements.join(","))
            .then(a.class_name.cmp(&b.class_name))
    });

    for assignment in sorted_assignments {
        output.push_str(&format!(
            "class {} {}\n",
            assignment.elements.join(","),
            assignment.class_name
        ));
    }

    Ok(output)
}

fn extract_all_class_definitions(content: &str) -> Vec<ClassDefinition> {
    let mut defs = Vec::new();
    let class_def_re = Regex::new(r"classDef\s+(\w+)\s+([^;\n]+)").unwrap();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("classDef ") {
            if let Some(caps) = class_def_re.captures(line) {
                defs.push(ClassDefinition {
                    name: caps[1].to_string(),
                    properties: caps[2].to_string(),
                });
            }
        }
    }

    defs
}

fn extract_all_class_assignments(content: &str) -> Vec<ClassAssignment> {
    let mut assignments = Vec::new();
    let class_assignment_re = Regex::new(r"^class\s+([\w,]+)\s+(\w+)").unwrap();
    let inline_style_re = Regex::new(r"(\w+):::(\w+)").unwrap();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("class ") {
            if let Some(caps) = class_assignment_re.captures(trimmed) {
                let elements = caps[1].split(',').map(|s| s.trim().to_string()).collect();
                assignments.push(ClassAssignment {
                    elements,
                    class_name: caps[2].to_string(),
                });
            }
        }

        for caps in inline_style_re.captures_iter(trimmed) {
            assignments.push(ClassAssignment {
                elements: vec![caps[1].to_string()],
                class_name: caps[2].to_string(),
            });
        }
    }

    assignments
}

fn extract_clean_content(content: &str, nodes: &mut HashSet<String>) -> String {
    let mut cleaned_lines = Vec::new();
    let mut in_init_block = false;
    let node_re = Regex::new(r"\[([^\]]+)\]").unwrap();

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip init blocks
        if trimmed.starts_with("%%{") {
            in_init_block = true;
            continue;
        }
        if in_init_block {
            if trimmed.contains("}%%") {
                in_init_block = false;
            }
            continue;
        }

        // Skip class definitions and assignments
        if trimmed.starts_with("classDef ") || trimmed.starts_with("class ") {
            continue;
        }

        // Skip certain comments
        if trimmed.starts_with("%%") {
            if trimmed.contains("-->") || trimmed.contains("class") {
                continue;
            }
        }

        // Skip flowchart directive
        if trimmed == "flowchart TB" {
            continue;
        }

        // Extract nodes
        for cap in node_re.captures_iter(trimmed) {
            nodes.insert(cap[1].to_string());
        }

        // Clean and add content
        let cleaned_line = line.replace("&nbsp;", " ");
        if !cleaned_line.trim().is_empty() {
            cleaned_lines.push(cleaned_line);
        }
    }

    cleaned_lines.join("\n")
}
