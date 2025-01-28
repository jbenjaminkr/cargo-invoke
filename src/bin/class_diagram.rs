use proc_macro2::TokenStream;
use quote::quote;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use syn::{
    parse_file, Field, Fields, FnArg, GenericArgument, ImplItem, ItemEnum, ItemImpl, ItemStruct,
    Pat, PathArguments, ReturnType, Type, TypePath, Variant,
};

#[derive(Debug)]
struct StructInfo {
    name: String,
    fields: Vec<FieldInfo>,
    methods: Vec<MethodInfo>,
}

#[derive(Debug)]
struct EnumInfo {
    name: String,
    variants: Vec<String>,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
struct Relationship {
    from: String,
    to: String,
    relationship_type: String,
}

#[derive(Debug)]
struct FieldInfo {
    name: String,
    type_name: String,
}

#[derive(Debug)]
struct MethodInfo {
    name: String,
    params: Vec<ParamInfo>,
    return_type: String,
}

#[derive(Debug)]
struct ParamInfo {
    name: String,
    type_name: String,
}

#[derive(Debug)]
struct TypeInfo {
    base_type: String,
    generic_params: Vec<String>,
}

fn main() -> std::io::Result<()> {
    let src_dir = Path::new("src/modules");
    let mut structs = Vec::new();
    let mut enums = Vec::new();

    fs::create_dir_all("diagrams")?;
    visit_dirs(src_dir, &mut structs, &mut enums)?;

    let relationships = extract_relationships(&structs, &enums);

    let mermaid = generate_mermaid_diagram(&structs, &enums, &relationships);
    let mut file = File::create("diagrams/class.mermaid")?;
    file.write_all(mermaid.as_bytes())?;

    Ok(())
}

fn visit_dirs(
    dir: &Path,
    structs: &mut Vec<StructInfo>,
    enums: &mut Vec<EnumInfo>,
) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, structs, enums)?;
            } else if path.extension().map_or(false, |ext| ext == "rs") {
                process_file(&path, structs, enums)?;
            }
        }
    }
    Ok(())
}

fn process_file(
    path: &Path,
    structs: &mut Vec<StructInfo>,
    enums: &mut Vec<EnumInfo>,
) -> std::io::Result<()> {
    let content = fs::read_to_string(path)?;
    let syntax = parse_file(&content).expect("Failed to parse Rust file");

    let mut struct_map: HashMap<String, usize> = HashMap::new();

    // First pass: collect structs and enums
    for item in syntax.items.iter() {
        match item {
            syn::Item::Struct(item_struct) => {
                let struct_info = process_struct(item_struct);
                struct_map.insert(struct_info.name.clone(), structs.len());
                structs.push(struct_info);
            }
            syn::Item::Enum(item_enum) => {
                let enum_info = process_enum(item_enum);
                enums.push(enum_info);
            }
            _ => {}
        }
    }

    // Second pass: collect impl blocks
    for item in syntax.items.iter() {
        if let syn::Item::Impl(item_impl) = item {
            if let Type::Path(type_path) = &*item_impl.self_ty {
                let struct_name = type_path
                    .path
                    .segments
                    .last()
                    .map(|seg| seg.ident.to_string())
                    .unwrap_or_default();

                if let Some(&struct_index) = struct_map.get(&struct_name) {
                    process_impl(item_impl, &mut structs[struct_index]);
                }
            }
        }
    }

    Ok(())
}

fn process_enum(item_enum: &ItemEnum) -> EnumInfo {
    let name = item_enum.ident.to_string();
    let variants = item_enum
        .variants
        .iter()
        .map(|variant| variant.ident.to_string())
        .collect();

    EnumInfo { name, variants }
}

fn extract_relationships(structs: &[StructInfo], enums: &[EnumInfo]) -> HashSet<Relationship> {
    let mut relationships = HashSet::new();
    let struct_names: HashSet<_> = structs.iter().map(|s| s.name.as_str()).collect();
    let enum_names: HashSet<_> = enums.iter().map(|e| e.name.as_str()).collect();

    for struct_info in structs {
        for field in &struct_info.fields {
            let type_info = parse_type_with_generics(&field.type_name);

            // Handle enum types
            if enum_names.contains(type_info.base_type.as_str()) {
                relationships.insert(Relationship {
                    from: struct_info.name.clone(),
                    to: type_info.base_type.clone(),
                    relationship_type: "has".to_string(),
                });
                continue;
            }

            // Handle struct types
            if struct_names.contains(type_info.base_type.as_str()) {
                relationships.insert(Relationship {
                    from: struct_info.name.clone(),
                    to: type_info.base_type.clone(),
                    relationship_type: "has".to_string(),
                });
            }

            // Handle special container types
            match type_info.base_type.as_str() {
                "Option" | "Vec" | "HashMap" => {
                    // Add relationship to the container type
                    relationships.insert(Relationship {
                        from: struct_info.name.clone(),
                        to: type_info.base_type.clone(),
                        relationship_type: "uses".to_string(),
                    });

                    // Add relationships to the generic type parameters
                    for generic_type in type_info.generic_params {
                        if struct_names.contains(generic_type.as_str())
                            || enum_names.contains(generic_type.as_str())
                        {
                            relationships.insert(Relationship {
                                from: struct_info.name.clone(),
                                to: generic_type,
                                relationship_type: "contains".to_string(),
                            });
                        }
                    }
                }
                _ => {}
            }
        }
    }

    relationships
}

fn parse_type_with_generics(type_str: &str) -> TypeInfo {
    let parts: Vec<&str> = type_str.split('<').collect();
    let base_type = parts[0].trim().to_string();

    let mut generic_params = Vec::new();
    if parts.len() > 1 {
        let generic_part = parts[1].trim_end_matches('>');
        for param in generic_part.split(',') {
            let clean_param = param
                .trim()
                .trim_start_matches('&')
                .trim_start_matches("mut ")
                .to_string();
            generic_params.push(clean_param);
        }
    }

    TypeInfo {
        base_type,
        generic_params,
    }
}

fn process_struct(item_struct: &ItemStruct) -> StructInfo {
    let name = item_struct.ident.to_string();
    let mut fields = Vec::new();

    if let Fields::Named(named_fields) = &item_struct.fields {
        for field in &named_fields.named {
            if let Some(ident) = &field.ident {
                fields.push(FieldInfo {
                    name: ident.to_string(),
                    type_name: get_type_name(&field.ty),
                });
            }
        }
    }

    StructInfo {
        name,
        fields,
        methods: Vec::new(),
    }
}

fn process_impl(item_impl: &ItemImpl, struct_info: &mut StructInfo) {
    for item in &item_impl.items {
        if let ImplItem::Fn(method) = item {
            let mut params = Vec::new();

            for input in &method.sig.inputs {
                match input {
                    FnArg::Typed(pat_type) => {
                        if let Pat::Ident(pat_ident) = &*pat_type.pat {
                            params.push(ParamInfo {
                                name: pat_ident.ident.to_string(),
                                type_name: get_type_name(&pat_type.ty),
                            });
                        }
                    }
                    FnArg::Receiver(_) => {}
                }
            }

            let return_type = match &method.sig.output {
                ReturnType::Default => "void".to_string(),
                ReturnType::Type(_, ty) => get_type_name(ty),
            };

            struct_info.methods.push(MethodInfo {
                name: method.sig.ident.to_string(),
                params,
                return_type,
            });
        }
    }
}

fn get_type_name(ty: &Type) -> String {
    match ty {
        Type::Path(TypePath { path, .. }) => {
            let mut full_type = String::new();

            if let Some(last_segment) = path.segments.last() {
                full_type.push_str(&last_segment.ident.to_string());

                if let PathArguments::AngleBracketed(args) = &last_segment.arguments {
                    let generic_args: Vec<String> = args
                        .args
                        .iter()
                        .filter_map(|arg| {
                            if let GenericArgument::Type(ty) = arg {
                                Some(get_type_name(ty))
                            } else {
                                None
                            }
                        })
                        .collect();

                    if !generic_args.is_empty() {
                        full_type.push('<');
                        full_type.push_str(&generic_args.join(", "));
                        full_type.push('>');
                    }
                }
            }

            full_type
        }
        Type::Reference(type_ref) => get_type_name(&type_ref.elem),
        _ => "unknown".to_string(),
    }
}

fn generate_mermaid_diagram(
    structs: &[StructInfo],
    enums: &[EnumInfo],
    relationships: &HashSet<Relationship>,
) -> String {
    let mut output = String::from("classDiagram\n");

    // Add enums with their variants as attributes
    for enum_info in enums {
        output.push_str(&format!("    class {} {{\n", enum_info.name));
        for variant in &enum_info.variants {
            output.push_str(&format!("        +{}\n", variant));
        }
        output.push_str("    }\n\n");
    }

    // Add structs with their fields and methods
    for struct_info in structs {
        output.push_str(&format!("    class {} {{\n", struct_info.name));

        // Add fields
        for field in &struct_info.fields {
            output.push_str(&format!("        +{}: {}\n", field.name, field.type_name));
        }

        // Add methods
        for method in &struct_info.methods {
            let params = method
                .params
                .iter()
                .map(|p| format!("{}: {}", p.name, p.type_name))
                .collect::<Vec<_>>()
                .join(", ");
            output.push_str(&format!(
                "        +{}({}) {}\n",
                method.name, params, method.return_type
            ));
        }

        output.push_str("    }\n\n");
    }

    // Add relationships
    for rel in relationships {
        output.push_str(&format!(
            "    {} --> {} : {}\n",
            rel.from, rel.to, rel.relationship_type
        ));
    }

    output
}
