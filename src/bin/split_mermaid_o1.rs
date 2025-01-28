use regex::Regex;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Structure to hold a top-level subgraph's data.
#[derive(Default, Debug)]
struct Subgraph {
    name: String,
    lines: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Expect one argument: the path to the .mermaid file
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input mermaid file>", args[0]);
        std::process::exit(1);
    }
    let input_path = Path::new(&args[1]);
    if !input_path.exists() {
        eprintln!("Error: file {:?} does not exist.", input_path);
        std::process::exit(1);
    }

    // Read lines, skipping any that start with "%%{init"
    let file = fs::File::open(input_path)?;
    let reader = BufReader::new(file);

    let mut all_lines = Vec::new();
    for line_result in reader.lines() {
        let line = line_result?;
        if !line.trim_start().starts_with("%%{init") {
            all_lines.push(line);
        }
    }

    // Separate out class-related lines (classDef or class) for later
    let mut class_lines: Vec<String> = Vec::new();
    let mut filtered_lines: Vec<String> = Vec::new();

    for line in all_lines {
        let trimmed = line.trim_start();
        if trimmed.starts_with("classDef ") || trimmed.starts_with("class ") {
            class_lines.push(line);
        } else {
            filtered_lines.push(line);
        }
    }

    // We'll parse top-level subgraphs. Subgraphs with names containing "&nbsp;"
    // will be **skipped** entirely.
    let mut subgraphs: Vec<Subgraph> = Vec::new();
    let mut current_subgraph: Option<Subgraph> = None;
    let mut nesting_level = 0usize;
    let mut skipping_nbsp_subgraph = false;

    for line in filtered_lines {
        let trimmed = line.trim_start();

        if trimmed.starts_with("subgraph ") {
            // "subgraph MyName"
            let tokens: Vec<&str> = trimmed.split_whitespace().collect();
            if tokens.len() >= 2 {
                let subg_name = tokens[1].to_string();

                // If we encounter a subgraph name with "&nbsp;", we skip it (and everything in it) entirely
                if subg_name.contains("&nbsp;") {
                    // We'll turn on a flag to skip lines until we match the "end" that closes it
                    skipping_nbsp_subgraph = true;
                    nesting_level += 1;
                    continue;
                }

                // If nesting_level == 0, this is a new top-level subgraph
                if nesting_level == 0 && !skipping_nbsp_subgraph {
                    let mut sg = Subgraph::default();
                    sg.name = subg_name;
                    sg.lines.push(line.clone());
                    current_subgraph = Some(sg);
                } else if !skipping_nbsp_subgraph {
                    // We are inside an existing subgraph
                    if let Some(sg) = current_subgraph.as_mut() {
                        sg.lines.push(line.clone());
                    }
                }
                nesting_level += 1;
            } else {
                // Malformed subgraph line?
                // If we are inside a valid subgraph (and not skipping), add it
                if nesting_level > 0 && !skipping_nbsp_subgraph {
                    if let Some(sg) = current_subgraph.as_mut() {
                        sg.lines.push(line.clone());
                    }
                }
            }
        } else if trimmed == "end" {
            // If skipping, reduce nesting. If we come back to zero, we're done skipping.
            if skipping_nbsp_subgraph {
                nesting_level = nesting_level.saturating_sub(1);
                if nesting_level == 0 {
                    skipping_nbsp_subgraph = false;
                }
                // We skip adding this 'end' to current_subgraph because it's part of the skipped subgraph
                continue;
            }

            // If not skipping, this end belongs to our current subgraph
            if let Some(sg) = current_subgraph.as_mut() {
                sg.lines.push(line.clone());
            }

            // Decrease nesting
            if nesting_level > 0 {
                nesting_level -= 1;
            }

            // If nesting hits 0, we've closed a top-level subgraph
            if nesting_level == 0 {
                if let Some(sg) = current_subgraph.take() {
                    subgraphs.push(sg);
                }
            }
        } else {
            // Normal line
            if nesting_level > 0 && !skipping_nbsp_subgraph {
                if let Some(sg) = current_subgraph.as_mut() {
                    sg.lines.push(line.clone());
                }
            }
            // If nesting_level == 0 or skipping_nbsp_subgraph == true, we do nothing
        }
    }

    // If there's an unclosed top-level subgraph (rare), add it
    if let Some(sg) = current_subgraph {
        subgraphs.push(sg);
    }

    // Sort the class lines for stable output
    class_lines.sort();

    // Prepare output directory with the same name as the input's stem
    let file_stem = input_path.file_stem().unwrap_or_default().to_string_lossy();
    let parent_dir = input_path.parent().unwrap_or_else(|| Path::new("."));
    let output_dir = parent_dir.join(file_stem.as_ref());
    fs::create_dir_all(&output_dir)?;

    // Minimal header for each subgraph file
    let header = "flowchart TB\n";

    // We'll use a regex to gather tokens from the subgraph lines
    // Then we only include class lines referencing any of those tokens.
    let token_regex = Regex::new(r"[A-Za-z0-9_]+")?;

    for sg in &subgraphs {
        // 1) Collect tokens from subgraph lines
        let mut tokens_in_subgraph = Vec::new();
        for l in &sg.lines {
            for cap in token_regex.captures_iter(l) {
                tokens_in_subgraph.push(cap[0].to_string());
            }
        }
        tokens_in_subgraph.sort();
        tokens_in_subgraph.dedup();

        // 2) Filter class lines to keep only those referencing at least one subgraph token
        let mut relevant_class_lines = Vec::new();
        for cl in &class_lines {
            // If any subgraph token appears in the class line, we keep it
            if tokens_in_subgraph.iter().any(|t| cl.contains(t)) {
                relevant_class_lines.push(cl.clone());
            }
        }

        // 3) Build the output for this subgraph
        let mut output = String::new();
        output.push_str(header);

        // Subgraph lines
        for line in &sg.lines {
            output.push_str(line);
            output.push('\n');
        }
        // Relevant class lines
        for cl in &relevant_class_lines {
            output.push_str(cl);
            output.push('\n');
        }

        // 4) Write to <subgraphName>.mermaid in the <file_stem> folder
        let out_path = output_dir.join(format!("{}.mermaid", sg.name));
        fs::write(&out_path, output)?;
        println!("Wrote subgraph to {:?}", out_path);
    }

    Ok(())
}
