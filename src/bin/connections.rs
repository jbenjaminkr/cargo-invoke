use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::Command;

/// Simplifies a single type by removing references and wrappers like `Option`, `Arc`, `Box`, etc.,
/// and returning the *final* type after stripping `::` paths.
/// e.g., Option<Result<Box<foo::Bar>>> -> Bar
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
                    // Recursively unwrap the inside — but we only care about the final type name
                    // for the single final return. The logic for capturing *all* subtypes is handled below.
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
/// e.g., `HashMap<A, Vec<B>>` -> ["A", "Vec<B>"] -> then "Vec<B>" -> ["B"] -> so final = ["A", "B"].
/// We'll return all final unwrapped types (i.e., remove Option, Box, Arc, etc.).
///
/// Example:
/// `HashMap<D, Vec<E>>` -> we get "D" and "Vec<E>" -> unwrap_type("Vec<E>") -> we parse out "E" as well.
fn extract_all_inner_types(ty: &str) -> Vec<String> {
    let mut results = Vec::new();

    // Top-level parse: if there's generics, let's isolate them:
    // e.g. "HashMap<A, Vec<B>>"
    // We'll find the first '<' and its matching '>' to extract "A, Vec<B>".
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

        // Inside "A, Vec<B>"
        let inside = &ty[start + 1..end];
        // Split at top-level commas
        let mut comma_depth = 0;
        let mut last_pos = 0;
        for (i, c) in inside.chars().enumerate() {
            match c {
                '<' => comma_depth += 1,
                '>' => comma_depth -= 1,
                ',' if comma_depth == 0 => {
                    // We found a top-level comma
                    let segment = inside[last_pos..i].trim();
                    results.push(segment.to_string());
                    last_pos = i + 1;
                }
                _ => {}
            }
        }
        // Final segment
        let segment = inside[last_pos..].trim();
        if !segment.is_empty() {
            results.push(segment.to_string());
        }
    }

    // If we found no generics, it's just a single type
    if results.is_empty() {
        // Return this as final unwrapped type
        return vec![unwrap_type(ty)];
    } else {
        // We have subtypes. For each subtype, we might have further generics.
        // So we recursively parse them as well and flatten everything.
        let mut final_list = Vec::new();
        for sub in results {
            // e.g. sub could be "Vec<B>"
            // parse again
            let deeper = extract_all_inner_types(&sub);
            final_list.extend(deeper);
        }
        return final_list;
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let generate_png = args.contains(&"--png".to_string());

    // 1. Read known classes
    let impls_path = PathBuf::from("architecture/impls.rs");
    let structs_path = PathBuf::from("architecture/structs.rs");
    let connections_path = PathBuf::from("diagrams/connections.mermaid");

    let mut known_classes = HashSet::new();

    // Regex to find `impl Something {` or `impl Something for AnotherThing {`
    let impl_name_regex =
        Regex::new(r"(?m)^impl(?:<[^>]+>)?\s+(?:\w+(?:<[^>]+>)?\s+for\s+)?(\w+)(?:<[^>]+>)?\s*\{")
            .unwrap();

    // Also gather from `struct SomeName {`
    let struct_name_regex =
        Regex::new(r"(?m)^(?:#\[[^\]]+\])*\s*(?:pub\s+)?struct\s+(\w+)").unwrap();

    // read impls
    if impls_path.exists() {
        let content = fs::read_to_string(&impls_path)?;
        for cap in impl_name_regex.captures_iter(&content) {
            known_classes.insert(cap[1].to_string());
        }
        for cap in struct_name_regex.captures_iter(&content) {
            known_classes.insert(cap[1].to_string());
        }
    }

    // read structs
    let mut structs_content = String::new();
    if structs_path.exists() {
        structs_content = fs::read_to_string(&structs_path)?;
        for cap in struct_name_regex.captures_iter(&structs_content) {
            known_classes.insert(cap[1].to_string());
        }
    }

    // 2. Parse each struct block to find fields referencing known classes
    let struct_block_regex =
        Regex::new(r"(?s)(?:#\[[^\]]+\]\s*)*(?:pub\s+)?struct\s+(\w+)\s*\{([^}]*)\}").unwrap();
    let field_regex = Regex::new(r"(?m)^\s*(?:pub\s+)?(\w+)\s*:\s*([^,]+)").unwrap();

    let mut relationships: Vec<(String, String, String)> = Vec::new();

    for block_cap in struct_block_regex.captures_iter(&structs_content) {
        let struct_name = block_cap[1].trim();
        let struct_body = block_cap[2].trim();

        for field_cap in field_regex.captures_iter(struct_body) {
            let field_type_raw = field_cap[2].trim();

            // Use extract_all_inner_types to capture all final unwrapped types
            let types_found = extract_all_inner_types(field_type_raw);

            // For each final type, if known, record a "has" relationship
            for t in types_found {
                if known_classes.contains(&t) {
                    relationships.push((struct_name.to_string(), "has".to_string(), t));
                }
            }
        }
    }

    // 3. Also parse function-call relationships from impls (skip `fn new`)
    let impl_block_regex = Regex::new(
        r"(?s)impl(?:<[^>]+>)?\s+(?:\w+(?:<[^>]+>)?\s+for\s+)?(\w+)(?:<[^>]+>)?\s*\{(.*?)\}",
    )
    .unwrap();
    let fn_regex = Regex::new(r"(?m)fn\s+(\w+)\s*\([^)]*\)\s*(?:->\s*([^;]+))?;").unwrap();

    if impls_path.exists() {
        let impls_content = fs::read_to_string(&impls_path)?;
        for block_cap in impl_block_regex.captures_iter(&impls_content) {
            let struct_name = block_cap[1].trim();
            let block_body = &block_cap[2];

            for fn_cap in fn_regex.captures_iter(block_body) {
                let fn_name = fn_cap[1].trim();
                // Skip "new"
                if fn_name == "new" {
                    continue;
                }

                let ret_ty_raw = fn_cap.get(2).map_or("", |m| m.as_str());
                if !ret_ty_raw.is_empty() {
                    let simplified = unwrap_type(ret_ty_raw);
                    if known_classes.contains(&simplified) {
                        relationships.push((
                            struct_name.to_string(),
                            fn_name.to_string(),
                            simplified,
                        ));
                    }
                }
            }
        }
    }

    // 4. Write out connections.mermaid
    let mut file = fs::File::create(&connections_path)?;
    writeln!(file, "graph LR")?;
    writeln!(file)?;

    for (a, label, b) in &relationships {
        writeln!(file, "    {} --> |{}| {}", a, label, b)?;
    }

    println!("Generated diagram at {}", connections_path.display());
    println!("Known classes used for matching: {:?}", known_classes);

    // 5. If --png flag is present, run the PNG generation
    if generate_png {
        println!("Generating PNG...");
        let status = Command::new("cargo")
            .args(&[
                "run",
                "--bin",
                "generate_mermaid_png",
                "--",
                connections_path.to_str().unwrap(),
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
