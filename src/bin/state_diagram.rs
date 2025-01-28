use anyhow::{anyhow, Context, Result};
use std::io::Write;
use std::{
    env, fs,
    path::{Path, PathBuf},
};
use syn::{parse_file, Field, Item, Type, TypePath};
use walkdir::WalkDir; // Import walkdir for recursive directory traversal // For writing to files

/// Represents a relationship between two structs via a field.
struct StructRelationship {
    source: String,
    field: String,
    target: String,
    cardinality: String, // e.g., "||--||" or "||--o{"
}

fn main() -> Result<()> {
    // Step 1: Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return Err(anyhow!(
            "Usage: {} <structs_file.rs>",
            args.get(0).unwrap_or(&"script".to_string())
        ));
    }
    let filename = &args[1];

    // Step 2: Search for the file in the current directory and subdirectories
    let filepath = find_file_in_current_dir(filename)
        .with_context(|| format!("Failed to find file: {}", filename))?;

    // Step 3: Read the file content
    let content = fs::read_to_string(&filepath)
        .with_context(|| format!("Failed to read file: {:?}", filepath))?;

    // Step 4: Parse the Rust file
    let syntax = parse_file(&content)
        .with_context(|| format!("Failed to parse Rust file: {:?}", filepath))?;

    // Step 5: Extract struct definitions
    let struct_names = extract_struct_names(&syntax);

    // Step 6: Extract relationships between structs
    let relationships = extract_struct_relationships(&syntax, &struct_names);

    // Step 7: Prepare output file path
    let output_file = prepare_output_file(filename)?;

    // Step 8: Generate Mermaid ER diagram and write to file
    generate_mermaid_er_diagram(&relationships, &output_file)?;

    println!("ER diagram successfully written to {:?}", output_file);

    Ok(())
}

/// Searches the current directory and all subdirectories for the first file matching the given filename.
fn find_file_in_current_dir(filename: &str) -> Result<PathBuf> {
    let current_dir = env::current_dir()?;
    println!(
        "Searching for '{}' in directory: {:?} and its subdirectories...",
        filename, current_dir
    );

    for entry in WalkDir::new(&current_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if let Some(name) = entry.file_name().to_str() {
            println!("Checking file: {:?}", entry.path());
            if name == filename {
                println!("Found file: {:?}", entry.path());
                return Ok(entry.path().to_path_buf());
            }
        }
    }
    Err(anyhow!(
        "File '{}' not found in the current directory or its subdirectories.",
        filename
    ))
}

/// Extracts all struct names from the parsed syntax.
fn extract_struct_names(syntax: &syn::File) -> Vec<String> {
    syntax
        .items
        .iter()
        .filter_map(|item| {
            if let Item::Struct(s) = item {
                Some(s.ident.to_string())
            } else {
                None
            }
        })
        .collect()
}

/// Extracts relationships between structs based on their fields.
fn extract_struct_relationships(
    syntax: &syn::File,
    struct_names: &[String],
) -> Vec<StructRelationship> {
    let mut relationships = Vec::new();

    for item in &syntax.items {
        if let Item::Struct(s) = item {
            let source = s.ident.to_string();
            for field in &s.fields {
                if let Some((field_name, target, cardinality)) =
                    get_field_relationship(field, struct_names)
                {
                    relationships.push(StructRelationship {
                        source: source.clone(),
                        field: field_name,
                        target,
                        cardinality,
                    });
                }
            }
        }
    }

    relationships
}

/// Determines if a field establishes a relationship with another struct.
/// Returns the field name, target struct name, and relationship cardinality if a relationship exists.
fn get_field_relationship(
    field: &Field,
    struct_names: &[String],
) -> Option<(String, String, String)> {
    // Get the field name
    let field_name = field
        .ident
        .as_ref()
        .map(|ident| ident.to_string())
        .unwrap_or_else(|| "unnamed_field".to_string());

    // Analyze the field type to find potential struct relationships
    let (types, is_collection) = extract_types_from_type(&field.ty);

    for ty in types {
        // Exclude standard library types
        if is_external_type(&ty) {
            continue;
        }

        // Check if the type is one of the structs
        if struct_names.contains(&ty) {
            let cardinality = if is_collection {
                "||--o{".to_string() // One-to-many
            } else {
                "||--||".to_string() // One-to-one
            };
            return Some((field_name, ty, cardinality));
        }
    }

    None
}

/// Extracts all type identifiers from a given syn::Type and determines if it's a collection type.
fn extract_types_from_type(ty: &Type) -> (Vec<String>, bool) {
    let mut types = Vec::new();
    let mut is_collection = false;
    extract_types_recursive(ty, &mut types, &mut is_collection);
    (types, is_collection)
}

/// Helper function to recursively extract type identifiers and determine if it's a collection.
fn extract_types_recursive(ty: &Type, types: &mut Vec<String>, is_collection: &mut bool) {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            if let Some(segment) = path.segments.last() {
                let ident = segment.ident.to_string();

                // Check if the type is a known collection
                if is_collection_type(&ident) {
                    *is_collection = true;

                    // For collections like Vec<Approval>, extract Approval
                    if let syn::PathArguments::AngleBracketed(ref args) = segment.arguments {
                        for arg in &args.args {
                            if let syn::GenericArgument::Type(ref inner_ty) = arg {
                                extract_types_recursive(inner_ty, types, is_collection);
                            }
                        }
                    }
                } else {
                    types.push(ident.clone());

                    // If the segment has generic arguments, extract them as well
                    if let syn::PathArguments::AngleBracketed(ref args) = segment.arguments {
                        for arg in &args.args {
                            if let syn::GenericArgument::Type(ref inner_ty) = arg {
                                extract_types_recursive(inner_ty, types, is_collection);
                            }
                        }
                    }
                }
            }
        }
        Type::Reference(type_ref) => {
            extract_types_recursive(&type_ref.elem, types, is_collection);
        }
        Type::Tuple(tuple) => {
            for elem in &tuple.elems {
                extract_types_recursive(elem, types, is_collection);
            }
        }
        Type::Array(arr) => {
            extract_types_recursive(&arr.elem, types, is_collection);
        }
        Type::Paren(paren) => {
            extract_types_recursive(&paren.elem, types, is_collection);
        }
        _ => {}
    }
}

/// Determines if a type is a known collection type.
fn is_collection_type(ty: &str) -> bool {
    let collection_types = [
        "Vec", "HashMap", "HashSet", "BTreeMap", "BTreeSet", "Option", "Box",
    ];
    collection_types.contains(&ty)
}

/// Determines if a type is external (e.g., from std or other crates).
fn is_external_type(ty: &str) -> bool {
    // List of common standard library generic types to exclude
    let external_types = [
        "Vec", "HashMap", "HashSet", "Option", "Result", "Box", "Rc", "Arc", "Mutex", "RefCell",
        "String", "BTreeMap", "BTreeSet",
        // Add more types as needed
    ];

    external_types.contains(&ty)
}

/// Prepares the output file path in the diagrams/ directory based on the input filename.
fn prepare_output_file(input_filename: &str) -> Result<PathBuf> {
    let input_path = Path::new(input_filename);
    let base_name = input_path
        .file_stem()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("Invalid input filename: {}", input_filename))?;

    let mut output_path = PathBuf::from("diagrams");
    // Create diagrams/ directory if it doesn't exist
    fs::create_dir_all(&output_path).with_context(|| "Failed to create 'diagrams/' directory.")?;

    output_path.push(format!("{}.mermaid", base_name));

    Ok(output_path)
}

/// Generates the Mermaid ER diagram and writes it to the specified file.
/// Excludes the ```mermaid ``` pre/postfix.
fn generate_mermaid_er_diagram(
    relationships: &[StructRelationship],
    output_file: &PathBuf,
) -> Result<()> {
    let mut file = fs::File::create(output_file)
        .with_context(|| format!("Failed to create output file: {:?}", output_file))?;

    // Write the starting line for Mermaid ER diagram
    writeln!(file, "erDiagram")?;

    for rel in relationships {
        writeln!(
            file,
            "    {} {} {} : \"{}\"",
            rel.source, rel.cardinality, rel.target, rel.field
        )?;
    }

    Ok(())
}
