use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use syn::{parse_file, Fields, Item, ItemStruct};

// fn is_wrapper_struct(item_struct: &ItemStruct) -> bool {
//     match &item_struct.fields {
//         // If it has no fields at all
//         Fields::Unit => true,

//         // If it has named fields but they're very simple
//         Fields::Named(fields) => {
//             fields.named.is_empty() ||
//             // Or if it only has primitive type fields
//             fields.named.iter().all(|field| {
//                 if let syn::Type::Path(type_path) = &field.ty {
//                     if let Some(segment) = type_path.path.segments.last() {
//                         let type_name = segment.ident.to_string();
//                         return is_primitive_type(&type_name);
//                     }
//                 }
//                 false
//             })
//         },

//         // If it has tuple fields but they're very simple
//         Fields::Unnamed(fields) => {
//             fields.unnamed.is_empty() ||
//             fields.unnamed.iter().all(|field| {
//                 if let syn::Type::Path(type_path) = &field.ty {
//                     if let Some(segment) = type_path.path.segments.last() {
//                         let type_name = segment.ident.to_string();
//                         return is_primitive_type(&type_name);
//                     }
//                 }
//                 false
//             })
//         }
//     }
// }

fn is_primitive_type(type_name: &str) -> bool {
    let primitives = [
        "u8", "u16", "u32", "u64", "u128", "i8", "i16", "i32", "i64", "i128", "f32", "f64", "bool",
        "char", "str", "String", "Vec", "Array",
    ];
    primitives.iter().any(|&p| p == type_name)
}

fn transform_struct_to_trait(item_struct: &ItemStruct) -> TokenStream {
    let struct_name = &item_struct.ident;
    let trait_name = format_ident!("{}", struct_name);
    let vis = &item_struct.vis;

    let methods = match &item_struct.fields {
        Fields::Named(fields) => fields
            .named
            .iter()
            .map(|field| {
                let field_name = &field.ident;
                let field_type = &field.ty;
                let getter_name = format_ident!("get_{}", field_name.as_ref().unwrap());
                let setter_name = format_ident!("set_{}", field_name.as_ref().unwrap());

                quote! {
                    fn #getter_name(&self) -> #field_type;
                    fn #setter_name(&mut self, value: #field_type);
                }
            })
            .collect::<Vec<_>>(),
        _ => vec![],
    };

    quote! {
        #vis trait #trait_name {
            #(#methods)*
        }
    }
}

fn process_file_content(content: String) -> std::io::Result<String> {
    let syntax_tree = parse_file(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    let mut output = String::new();

    // List of types to skip
    let skip_types = [
        // "Address",
        // "BlockHash",
        // "Hash",
        // "Signature",
        // "DateTime",
        // "Duration",
        // "IpAddr",
        // "Instant",
        // "SystemTime",
    ];

    for item in syntax_tree.items {
        match item {
            Item::Struct(item_struct) => {
                // Skip if it's in our skip list
                if skip_types.contains(&item_struct.ident.to_string().as_str()) {
                    continue;
                }

                // Skip if it's a wrapper struct
                if is_wrapper_struct(&item_struct) {
                    continue;
                }

                let trait_def = transform_struct_to_trait(&item_struct);
                let trait_str = trait_def.to_string();

                // Skip if the trait would be empty
                if !trait_str.contains("fn") {
                    continue;
                }

                output.push_str(&trait_str);
                output.push_str("\n\n");
            }
            _ => {}
        }
    }

    Ok(output)
}

fn is_wrapper_struct(item_struct: &ItemStruct) -> bool {
    match &item_struct.fields {
        Fields::Unit => true,
        Fields::Named(fields) => fields.named.is_empty(),
        Fields::Unnamed(fields) => fields.unnamed.is_empty(),
    }
}

fn process_directory<P: AsRef<Path>>(
    dir_path: P,
    concat_output: &mut File,
    traits_output: &mut File,
) -> std::io::Result<()> {
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            process_directory(&path, concat_output, traits_output)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            println!("Processing: {:?}", path);

            // Read file content
            let content = fs::read_to_string(&path)?;

            // Write to concatenated file
            writeln!(concat_output, "\n// File: {:?}", path)?;
            writeln!(concat_output, "{}\n", content)?;

            // Transform and write to traits file
            writeln!(traits_output, "\n// Generated traits for: {:?}", path)?;
            let traits = process_file_content(content)?;
            writeln!(traits_output, "{}\n", traits)?;
        }
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    // Create output files
    let mut concat_output = File::create("modules.rs")?;
    let mut traits_output = File::create("traits.rs")?;

    // Write module wrappers
    writeln!(concat_output, "mod modules {{\n")?;
    writeln!(traits_output, "mod generated_traits {{\n")?;

    // Process all files
    process_directory("src/modules", &mut concat_output, &mut traits_output)?;

    // Close modules
    writeln!(concat_output, "}}")?;
    writeln!(traits_output, "}}")?;

    println!("Created modules.rs and traits.rs");
    Ok(())
}
