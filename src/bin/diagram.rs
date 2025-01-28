use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
struct Field {
    name: String,
    type_name: String,
}

#[derive(Debug)]
struct StructDef {
    name: String,
    fields: Vec<Field>,
}

fn extract_struct_info(content: &str) -> Vec<StructDef> {
    let mut structs = Vec::new();
    // Modified to handle multiline struct definitions better
    let struct_regex =
        Regex::new(r"(?m)^\s*pub struct ([A-Za-z_][A-Za-z0-9_]*)\s*\{([\s\S]*?)\}").unwrap();
    // Modified to handle both public and private fields, including comments
    let field_regex =
        Regex::new(r"(?m)^\s*(?:pub\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*:\s*([^,\n]+)(?:,|\n|$)")
            .unwrap();

    for struct_cap in struct_regex.captures_iter(content) {
        let struct_name = struct_cap[1].to_string();
        let struct_body = &struct_cap[2];

        // Debug print to verify captures
        println!("Found struct: {}", struct_name);

        let mut fields = Vec::new();
        for field_cap in field_regex.captures_iter(struct_body) {
            let field_name = field_cap[1].to_string();
            let field_type = field_cap[2].trim().to_string();

            // Debug print to verify field captures
            println!("  Field: {} : {}", field_name, field_type);

            fields.push(Field {
                name: field_name,
                type_name: field_type,
            });
        }

        structs.push(StructDef {
            name: struct_name,
            fields,
        });
    }

    // Rest of the function remains the same...
    let mut unique_structs: HashMap<String, StructDef> = HashMap::new();
    for struct_def in structs {
        match unique_structs.get(&struct_def.name) {
            Some(existing) if existing.fields.len() < struct_def.fields.len() => {
                unique_structs.insert(struct_def.name.clone(), struct_def);
            }
            None => {
                unique_structs.insert(struct_def.name.clone(), struct_def);
            }
            _ => {}
        }
    }

    unique_structs.into_values().collect()
}

fn generate_mermaid(structs: &[StructDef]) -> String {
    let mut mermaid = String::from("classDiagram\n");
    let mut relationships = HashSet::new();

    // Generate class definitions
    for struct_def in structs {
        mermaid.push_str(&format!("    class {} {{\n", struct_def.name));
        for field in &struct_def.fields {
            // Clean up the type name for display
            let clean_type = field
                .type_name
                .replace("Arc<", "")
                .replace("Rc<", "")
                .replace("RwLock<", "")
                .replace(">", "")
                .replace("Vec<", "Vec~")
                .replace("Option<", "Option~")
                .replace("HashMap<", "Map~");

            // Check if the type contains 'pub' to determine visibility
            let visibility = if field.type_name.starts_with("pub ") {
                "+"
            } else {
                "-"
            };
            mermaid.push_str(&format!(
                "        {}{} {}\n",
                visibility, clean_type, field.name
            ));
        }
        mermaid.push_str("    }\n\n");
    }

    // Rest of the function remains the same...
    for struct_def in structs {
        for field in &struct_def.fields {
            for other_struct in structs {
                if field.type_name.contains(&other_struct.name) {
                    let relationship =
                        format!("    {} --> {} : has\n", struct_def.name, other_struct.name);
                    relationships.insert(relationship);
                }
            }
        }
    }

    for relationship in relationships {
        mermaid.push_str(&relationship);
    }

    mermaid
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        std::process::exit(1);
    }

    let input_file = &args[1];

    // Try common directories, including current dir and architecture dir
    let mut input_path = if input_file.ends_with(".rs") {
        PathBuf::from(input_file)
    } else {
        let mut path = PathBuf::from(input_file);
        path.set_extension("rs");
        path
    };

    // If file doesn't exist in current dir, check architecture dir
    if !input_path.exists() {
        input_path = PathBuf::from("architecture");
        input_path.push(if input_file.ends_with(".rs") {
            input_file.to_string()
        } else {
            format!("{}.rs", input_file)
        });
    }

    // Check if the input file exists
    if !input_path.exists() {
        eprintln!(
            "Error: Input file '{}' does not exist.",
            input_path.display()
        );
        std::process::exit(1);
    }

    // Read the content of the input file
    let content = fs::read_to_string(&input_path)?;

    // Extract struct info and generate the Mermaid diagram
    let structs = extract_struct_info(&content);
    let mermaid = generate_mermaid(&structs);

    // Prepare the output file path in the 'diagrams' directory
    let mut output_path = PathBuf::from("diagrams");
    fs::create_dir_all(&output_path)?; // Ensure the 'diagrams' directory exists

    // Use the input filename without extension for the output
    let output_name = input_path.file_stem().unwrap_or_default();
    output_path.push(output_name);
    output_path.set_extension("mermaid");

    // Write the Mermaid diagram to the output file
    fs::write(&output_path, mermaid)?;
    println!(
        "Generated class diagram for '{}' and saved to '{}'",
        input_path.display(),
        output_path.display()
    );

    Ok(())
}
