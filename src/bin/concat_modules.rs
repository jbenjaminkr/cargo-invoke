use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

fn concat_rust_files<P: AsRef<Path>>(dir_path: P, output: &mut File) -> std::io::Result<()> {
    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            concat_rust_files(&path, output)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            // Write module path as a comment
            writeln!(output, "\n// File: {:?}", path)?;

            // Read and write the file content
            let content = fs::read_to_string(&path)?;
            writeln!(output, "{}\n", content)?;
        }
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let mut output = File::create("all_modules.rs")?;

    // Write a wrapper module to ensure everything compiles
    writeln!(output, "mod all_modules {{\n")?;

    // Concatenate all files
    concat_rust_files("src/modules", &mut output)?;

    // Close the module
    writeln!(output, "}}")?;

    println!("Created all_modules.rs");
    Ok(())
}
