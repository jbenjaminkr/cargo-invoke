use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

/// Simplifies a single type by removing references and wrappers like `Option`, `Arc`, `Box`, etc.
fn unwrap_type(mut ty: &str) -> String {
    ty = ty.trim();
    while ty.starts_with('&') {
        ty = ty[1..].trim_start();
    }

    let wrappers = [
        "Option", "Result", "Box", "Arc", "Rc", "Mutex", "HashSet", "Vec", "HashMap",
    ];
    let mut result = ty.to_string();

    'outer: loop {
        for w in &wrappers {
            let prefix = format!("{}<", w);
            if result.starts_with(&prefix) {
                if let Some(start) = result.find('<') {
                    let mut depth = 0;
                    let mut end = start + 1;
                    for (i, c) in result[start + 1..].chars().enumerate() {
                        if c == '<' {
                            depth += 1;
                        } else if c == '>' {
                            if depth == 0 {
                                end = start + 1 + i;
                                break;
                            } else {
                                depth -= 1;
                            }
                        }
                    }
                    let inside = &result[start + 1..end];
                    result = unwrap_type(inside);
                    continue 'outer;
                }
            }
        }
        break;
    }

    if let Some(colpos) = result.rfind("::") {
        result = result[colpos + 2..].to_string();
    }
    result.trim().to_string()
}

/// Recursively parses a generic type for all "top-level" arguments.
fn extract_all_inner_types(ty: &str) -> Vec<String> {
    let mut results = Vec::new();

    if let Some(start) = ty.find('<') {
        let mut depth = 0;
        let mut end = start + 1;
        let chars: Vec<char> = ty.chars().collect();
        for (i, c) in chars[start + 1..].iter().enumerate() {
            if *c == '<' {
                depth += 1;
            } else if *c == '>' {
                if depth == 0 {
                    end = start + 1 + i;
                    break;
                } else {
                    depth -= 1;
                }
            }
        }

        let inside = &ty[start + 1..end];
        let mut comma_depth = 0;
        let mut last_pos = 0;
        for (i, c) in inside.chars().enumerate() {
            match c {
                '<' => comma_depth += 1,
                '>' => comma_depth -= 1,
                ',' if comma_depth == 0 => {
                    let segment = inside[last_pos..i].trim();
                    results.push(segment.to_string());
                    last_pos = i + 1;
                }
                _ => {}
            }
        }
        let segment = inside[last_pos..].trim();
        if !segment.is_empty() {
            results.push(segment.to_string());
        }
    }

    if results.is_empty() {
        return vec![unwrap_type(ty)];
    } else {
        let mut final_list = Vec::new();
        for sub in results {
            let deeper = extract_all_inner_types(&sub);
            final_list.extend(deeper);
        }
        return final_list;
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let generate_png = args.contains(&"--png".to_string());

    // We only read from `structs.rs`
    let structs_path = PathBuf::from("architecture/structs.rs");
    let state_diagram_path = PathBuf::from("diagrams/state_diagram.mermaid");

    let mut known_classes = HashSet::new();

    // We'll find "struct SomeName {" lines
    let struct_name_regex =
        Regex::new(r"(?m)^(?:#\[[^\]]+\])*\s*(?:pub\s+)?struct\s+(\w+)").unwrap();

    // If `architecture/structs.rs` exists, read it
    let mut structs_content = String::new();
    if structs_path.exists() {
        structs_content = fs::read_to_string(&structs_path)?;
        for cap in struct_name_regex.captures_iter(&structs_content) {
            known_classes.insert(cap[1].to_string());
        }
    }

    // Parse each struct block to find fields referencing known classes
    let struct_block_regex =
        Regex::new(r"(?s)(?:#\[[^\]]+\]\s*)*(?:pub\s+)?struct\s+(\w+)\s*\{([^}]*)\}").unwrap();
    let field_regex = Regex::new(r"(?m)^\s*(?:pub\s+)?(\w+)\s*:\s*([^,]+)").unwrap();

    let mut relationships: Vec<(String, String, String)> = Vec::new();

    // For each struct block in `structs_content`, parse fields
    for block_cap in struct_block_regex.captures_iter(&structs_content) {
        let struct_name = block_cap[1].trim();
        let struct_body = block_cap[2].trim();

        for field_cap in field_regex.captures_iter(struct_body) {
            let field_type_raw = field_cap[2].trim();
            let types_found = extract_all_inner_types(field_type_raw);

            for t in types_found {
                if known_classes.contains(&t) {
                    relationships.push((struct_name.to_string(), "has".to_string(), t));
                }
            }
        }
    }

    // Write out `state_diagram.mermaid`
    let mut file = fs::File::create(&state_diagram_path)?;
    writeln!(file, "graph LR")?;
    writeln!(file)?;

    for (a, label, b) in &relationships {
        writeln!(file, "    {} --> |{}| {}", a, label, b)?;
    }

    println!("Generated diagram at {}", state_diagram_path.display());
    println!("Known classes used for matching: {:?}", known_classes);

    // Optionally generate PNG if `--png` is passed
    if generate_png {
        println!("Generating PNG...");
        let status = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "generate_mermaid_png",
                "--",
                state_diagram_path.to_str().unwrap(),
            ])
            .status()?;

        if !status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to generate PNG",
            ));
        }
    }

    Ok(())
}
