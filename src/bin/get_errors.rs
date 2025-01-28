use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Command;

#[derive(Debug, Deserialize, Serialize)]
struct CompilerMessage {
    message: String,
    level: String,
    spans: Vec<Span>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Span {
    file_name: String,
    line_start: u32,
    line_end: u32,
}

fn analyze_rust_errors(
    file_path: &str,
    filter_level: Option<&str>,
) -> Result<HashMap<String, u32>, String> {
    // Run rustc with JSON output
    let output = Command::new("rustc")
        .args(&["--error-format=json", file_path])
        .output()
        .map_err(|e| format!("Failed to execute rustc: {}", e))?;

    // Parse JSON output
    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut error_counts: HashMap<String, u32> = HashMap::new();

    for line in stderr.lines() {
        if let Ok(message) = serde_json::from_str::<CompilerMessage>(line) {
            if let Some(filter) = filter_level {
                if message.level == filter {
                    *error_counts.entry(message.level).or_insert(0) += 1;
                }
            } else {
                *error_counts.entry(message.level).or_insert(0) += 1;
            }
        }
    }

    Ok(error_counts)
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return Err("Usage: rust-error-analyzer <file_path> [error_level]".to_string());
    }

    let file_path = &args[1];
    let filter_level = args.get(2).map(|s| s.as_str());

    match analyze_rust_errors(file_path, filter_level) {
        Ok(counts) => {
            println!("Error Analysis Results:");
            for (level, count) in counts {
                println!("{}: {}", level, count);
            }
            Ok(())
        }
        Err(e) => Err(format!("Error: {}", e)),
    }
}
