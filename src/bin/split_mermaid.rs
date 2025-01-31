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

struct Subgraph {
    name: String,
    content: String,
    class_assignments: Vec<ClassAssignment>,
    used_classes: HashSet<String>,
    nodes: HashSet<String>,
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

    let output_dir = input_path.with_extension("");
    fs::create_dir_all(&output_dir)?;

    let content = fs::read_to_string(input_path)?;
    process_mermaid_file(&content, &output_dir)?;

    Ok(())
}

fn process_mermaid_file(content: &str, output_dir: &Path) -> io::Result<()> {
    // Extract all class definitions first
    let class_defs = extract_all_class_definitions(content);

    // Extract all class assignments (both global and inline)
    let class_assignments = extract_all_class_assignments(content);

    // Extract subgraphs with recursive nesting
    let subgraphs = extract_subgraphs(content);

    // Process each subgraph
    for subgraph in subgraphs {
        // Skip empty or wrapper subgraphs
        if subgraph.name.contains(".") || subgraph.content.trim().is_empty() {
            continue;
        }

        let mut output = String::new();
        output.push_str("flowchart TB\n\n");

        // Clean and add subgraph content
        let cleaned_content = clean_content(&subgraph.content);
        output.push_str(&cleaned_content);
        output.push('\n');

        // Track which classes are actually used
        let mut actually_used_classes = HashSet::new();

        // Process class assignments and collect used classes
        let mut relevant_assignments = Vec::new();
        for assignment in &class_assignments {
            // Filter elements to only those present in this subgraph
            let relevant_elements: Vec<String> = assignment
                .elements
                .iter()
                .filter(|elem| subgraph.nodes.contains(*elem))
                .cloned()
                .collect();

            if !relevant_elements.is_empty() {
                relevant_assignments.push(ClassAssignment {
                    elements: relevant_elements,
                    class_name: assignment.class_name.clone(),
                });
                actually_used_classes.insert(assignment.class_name.clone());
            }
        }

        // Sort and write class definitions for actually used classes
        let mut used_class_defs: Vec<&ClassDefinition> = class_defs
            .iter()
            .filter(|def| actually_used_classes.contains(&def.name))
            .collect();
        used_class_defs.sort_by(|a, b| a.name.cmp(&b.name));

        for def in used_class_defs {
            output.push_str(&format!("classDef {} {}\n", def.name, def.properties));
        }

        // Sort and write filtered class assignments
        relevant_assignments.sort_by(|a, b| {
            a.elements
                .join(",")
                .cmp(&b.elements.join(","))
                .then(a.class_name.cmp(&b.class_name))
        });

        for assignment in relevant_assignments {
            output.push_str(&format!(
                "class {} {}\n",
                assignment.elements.join(","),
                assignment.class_name
            ));
        }

        // Write to file
        let filename = format!("{}.mermaid", sanitize_filename(&subgraph.name));
        let output_path = output_dir.join(filename);
        let mut file = File::create(output_path)?;
        file.write_all(output.as_bytes())?;
    }

    Ok(())
}

fn extract_all_class_definitions(content: &str) -> Vec<ClassDefinition> {
    let mut defs = Vec::new();
    let class_def_re = Regex::new(r"classDef\s+(\w+)\s+([^;\n]+)").unwrap();

    for line in content.lines() {
        if line.trim().starts_with("classDef ") {
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

        // Extract explicit class assignments
        if trimmed.starts_with("class ") {
            if let Some(caps) = class_assignment_re.captures(trimmed) {
                let elements = caps[1].split(',').map(|s| s.trim().to_string()).collect();
                assignments.push(ClassAssignment {
                    elements,
                    class_name: caps[2].to_string(),
                });
            }
        }

        // Extract inline styles
        for caps in inline_style_re.captures_iter(trimmed) {
            assignments.push(ClassAssignment {
                elements: vec![caps[1].to_string()],
                class_name: caps[2].to_string(),
            });
        }
    }

    assignments
}

fn extract_subgraphs(content: &str) -> Vec<Subgraph> {
    let mut subgraphs = Vec::new();
    let mut lines = content.lines().peekable();

    while let Some(line) = lines.next() {
        if line.trim().starts_with("subgraph ") {
            if let Some(subgraph) = extract_single_subgraph(line, &mut lines) {
                subgraphs.push(subgraph);
            }
        }
    }

    subgraphs
}

fn extract_single_subgraph<'a, I>(
    first_line: &str,
    lines: &mut std::iter::Peekable<I>,
) -> Option<Subgraph>
where
    I: Iterator<Item = &'a str>,
{
    let mut content = String::new();
    let mut all_nodes = HashSet::new();
    let mut depth = 1;
    let name_re = Regex::new(r#"subgraph\s+(?:"([^"]+)"|([^\s\[]+))"#).unwrap();
    let node_re = Regex::new(r"\[([^\]]+)\]").unwrap();

    // Extract current subgraph name
    let name = name_re
        .captures(first_line)
        .and_then(|caps| {
            caps.get(1)
                .or_else(|| caps.get(2))
                .map(|m| m.as_str().to_string())
        })
        .unwrap_or_else(|| "unnamed".to_string());

    // Add the main subgraph name to nodes
    all_nodes.insert(name.clone());
    content.push_str(first_line);
    content.push('\n');

    // Process content until matching end
    while let Some(line) = lines.next() {
        let trimmed = line.trim();

        if trimmed.starts_with("subgraph ") {
            depth += 1;
            // Extract and add nested subgraph name
            if let Some(caps) = name_re.captures(trimmed) {
                if let Some(nested_name) = caps.get(1).or_else(|| caps.get(2)) {
                    all_nodes.insert(nested_name.as_str().to_string());
                }
            }
        } else if trimmed == "end" {
            depth -= 1;
            if depth == 0 {
                content.push_str(line);
                content.push('\n');
                break;
            }
        }

        // Extract nodes from this line
        for cap in node_re.captures_iter(trimmed) {
            all_nodes.insert(cap[1].to_string());
        }

        content.push_str(line);
        content.push('\n');
    }

    let used_classes = extract_used_classes(&content);
    Some(Subgraph {
        name,
        content,
        class_assignments: Vec::new(),
        used_classes,
        nodes: all_nodes,
    })
}

fn extract_used_classes(content: &str) -> HashSet<String> {
    let mut classes = HashSet::new();
    let inline_style_re = Regex::new(r":::(\w+)").unwrap();
    let class_assignment_re = Regex::new(r"^class\s+[\w,]+\s+(\w+)").unwrap();

    for line in content.lines() {
        let trimmed = line.trim();

        // Extract inline styles
        for cap in inline_style_re.captures_iter(trimmed) {
            classes.insert(cap[1].to_string());
        }

        // Extract class names from assignments
        if let Some(caps) = class_assignment_re.captures(trimmed) {
            classes.insert(caps[1].to_string());
        }
    }

    classes
}

fn clean_content(content: &str) -> String {
    content
        .lines()
        .map(|line| line.replace("&nbsp;", " "))
        .collect::<Vec<_>>()
        .join("\n")
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
