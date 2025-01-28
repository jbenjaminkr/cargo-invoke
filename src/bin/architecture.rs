use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct CodeComponents {
    use_statements: HashSet<String>,
    // Key = struct name, Value = struct definition
    structs: HashMap<String, String>,
    // We'll store each impl block with function signatures
    impls: Vec<String>,
}

impl CodeComponents {
    fn new() -> Self {
        CodeComponents {
            use_statements: HashSet::new(),
            structs: HashMap::new(),
            impls: Vec::new(),
        }
    }

    fn add_use_statements(&mut self, statements: Vec<String>) {
        self.use_statements.extend(statements);
    }

    fn add_structs(&mut self, structs: Vec<String>) {
        // We'll parse out the struct name to keep them unique in a HashMap
        let struct_name_regex =
            Regex::new(r"(?m)^(?:#\[[^\]]+\]\s*)*(?:pub\s+)?struct\s+(\w+)").unwrap();
        for s in structs {
            if let Some(capt) = struct_name_regex.captures(&s) {
                let name = capt[1].to_string();
                self.structs.insert(name, s.clone());
            }
        }
    }

    fn add_impls(&mut self, mut impls: Vec<String>) {
        self.impls.append(&mut impls);
    }
}

struct CodeExtractor {
    use_regex: Regex,
    struct_regex: Regex,
    // We'll capture entire impl block (with group(1) = struct name, group(2) = block contents)
    impl_regex: Regex,
    // A regex for function signatures, capturing function name and optional return type
    fn_signature_regex: Regex,
}

impl CodeExtractor {
    fn new() -> Self {
        CodeExtractor {
            use_regex: Regex::new(r"^\s*use\s+[^;]+;").unwrap(),
            struct_regex: Regex::new(
                r"(?m)^(?:#\[[^\]]+\]\s*)*(?:pub\s+)?struct\s+\w+(?:<[^>]+>)?\s*\{[^}]*\}"
            ).unwrap(),
            impl_regex: Regex::new(
                r"(?ms)^\s*(?:#\[[^\]]*\])*\s*impl(?:<[^>]+>)?\s+(?:\w+(?:<[^>]+>)?\s+for\s+)?(\w+)(?:<[^>]+>)?\s*(?:where\s+[^{]*?)?\s*\{(.*)\n\s*\}\s*"
            ).unwrap(),
            fn_signature_regex: Regex::new(
                r"(?m)^\s*(?:#\[[^\]]*\]\s*)*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)\s*(?:<[^>]*>)?\s*\([^)]*\)\s*(?:->\s*(?:[^{};]|<[^>]*>)*)?(?:\{|;)"
            ).unwrap(),
                                    }
    }

    fn extract_components(&self, content: &str) -> CodeComponents {
        let mut components = CodeComponents::new();

        // **a. Extract Use Statements**
        for use_cap in self.use_regex.find_iter(content) {
            components.add_use_statements(vec![use_cap.as_str().to_string()]);
        }

        // **b. Extract Struct Definitions**
        for struct_cap in self.struct_regex.find_iter(content) {
            components.add_structs(vec![struct_cap.as_str().to_string()]);
        }

        // **c. Extract Impl Blocks**
        for impl_cap in self.impl_regex.captures_iter(content) {
            let struct_name = impl_cap.get(1).map_or("???", |m| m.as_str());
            let impl_content = impl_cap.get(2).unwrap().as_str();

            let mut impl_block = String::new();
            impl_block.push_str(&format!("impl {} {{\n", struct_name));

            // Find all function signatures within the impl content
            for fn_cap in self.fn_signature_regex.captures_iter(impl_content) {
                // Extract the function name
                let fn_name = fn_cap.get(1).map_or("", |m| m.as_str());

                // Skip if the function name is 'new'
                if fn_name == "new" {
                    continue;
                }

                // Extract the entire function signature
                let fn_sig_match = fn_cap.get(0).unwrap().as_str().trim();
                let fn_sig = fn_sig_match.split('{').next().unwrap().trim();

                impl_block.push_str(&format!("    {};\n", fn_sig));
            }

            impl_block.push_str("}");
            components.add_impls(vec![impl_block]);
        }

        components
    }
}

// Recursively process directories
fn process_directory(dir_path: &Path) -> io::Result<CodeComponents> {
    println!("Processing directory: {:?}", dir_path); // Add this
    let extractor = CodeExtractor::new();
    let mut all_components = CodeComponents::new();

    if !dir_path.exists() {
        println!("Directory does not exist: {:?}", dir_path); // Add this
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Source directory not found",
        ));
    }

    if dir_path.ends_with("bin") {
        return Ok(all_components);
    }

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        println!("Found entry: {:?}", path); // Add this
        if path.is_dir() {
            println!("Processing subdirectory: {:?}", path); // Add this
            let sub_components = process_directory(&path)?;
            all_components.add_use_statements(sub_components.use_statements.into_iter().collect());
            all_components
                .add_structs(sub_components.structs.into_iter().map(|(_, v)| v).collect());
            all_components.add_impls(sub_components.impls);
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            println!("Processing Rust file: {:?}", path); // Add this
            let content = fs::read_to_string(&path)?;
            let components = extractor.extract_components(&content);
            println!("Found {} impls in {:?}", components.impls.len(), path); // Add this
            all_components.add_use_statements(components.use_statements.into_iter().collect());
            all_components.add_structs(components.structs.into_iter().map(|(_, v)| v).collect());
            all_components.add_impls(components.impls);
        }
    }

    Ok(all_components)
}
// Sort & write to architecture: prelude.rs, structs.rs, impls.rs
fn write_file(
    path: &Path,
    use_statements: &HashSet<String>,
    content: &[String],
    is_prelude: bool,
    sort: bool,
) -> io::Result<()> {
    let mut file = File::create(path)?;

    if is_prelude {
        let mut sorted_uses: Vec<_> = use_statements.iter().collect();
        sorted_uses.sort();
        for stmt in sorted_uses {
            writeln!(file, "{}", stmt)?;
        }
    } else {
        writeln!(file, "use prelude::*;\n")?;

        let mut content_to_write = content.to_vec();
        if sort {
            content_to_write.sort_by(|a, b| extract_name(a).cmp(&extract_name(b)));
        }

        for item in &content_to_write {
            writeln!(file, "{}", item)?;
        }
    }

    Ok(())
}

fn extract_name(s: &str) -> String {
    for line in s.lines() {
        if line.trim_start().starts_with('#') {
            continue;
        }
        if line.contains("struct") || line.contains("impl") {
            let tokens: Vec<&str> = line.split_whitespace().collect();
            if let Some(i) = tokens.iter().position(|&t| t == "struct" || t == "impl") {
                if let Some(name) = tokens.get(i + 1) {
                    return name.trim_matches('{').to_lowercase();
                }
            }
        }
    }
    "".to_string()
}

fn write_architecture_files(arch_dir: &Path, components: &CodeComponents) -> io::Result<()> {
    // prelude.rs
    write_file(
        &arch_dir.join("prelude.rs"),
        &components.use_statements,
        &[],
        true,
        false,
    )?;

    // structs.rs
    let mut sorted_structs: Vec<&String> = components.structs.values().collect();
    if sorted_structs.len() > 1 {
        sorted_structs.sort_by(|a, b| extract_name(a).cmp(&extract_name(b)));
    }
    let struct_contents: Vec<String> = sorted_structs.into_iter().cloned().collect();

    write_file(
        &arch_dir.join("structs.rs"),
        &HashSet::new(),
        &struct_contents,
        false,
        false,
    )?;

    // impls.rs
    let mut sorted_impls = components.impls.clone();
    sorted_impls.sort_by(|a, b| extract_name(a).cmp(&extract_name(b)));

    write_file(
        &arch_dir.join("impls.rs"),
        &HashSet::new(),
        &sorted_impls,
        false,
        false,
    )?;

    Ok(())
}

fn main() -> io::Result<()> {
    println!("Running architecture extraction..."); // Debug statement

    let src_dir = PathBuf::from("src/modules");
    let arch_dir = PathBuf::from("architecture");

    fs::create_dir_all(&arch_dir)?;

    // Extract components
    let components = process_directory(&src_dir)?;

    // Debug: Print captured impls
    println!("Captured impls:");
    for impl_block in &components.impls {
        println!("{}", impl_block);
    }

    // Write architecture files
    write_architecture_files(&arch_dir, &components)?;

    println!(
        "Successfully processed {} structs and {} impl blocks.",
        components.structs.len(),
        components.impls.len()
    );
    println!(
        "Files written to:\n  {:?}/prelude.rs\n  {:?}/structs.rs\n  {:?}/impls.rs",
        arch_dir, arch_dir, arch_dir
    );

    Ok(())
}
