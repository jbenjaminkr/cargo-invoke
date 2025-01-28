use regex::Regex;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::io::{self, Write};

#[derive(Debug)]
struct TraitInfo {
    name: String,
    references: BTreeSet<String>,
}

fn main() -> io::Result<()> {
    // 1. Read all contents of "traits.rs"
    let contents = fs::read_to_string("traits.rs")?;

    // 2. Identify trait declarations (just the keyword "trait" and name).
    let trait_decl_re = Regex::new(r"(?:pub\s+)?trait\s+([A-Za-z0-9_]+)").unwrap();

    // Collect all trait names for reference-scanning
    let mut all_trait_names = HashSet::new();
    for mat in trait_decl_re.captures_iter(&contents) {
        all_trait_names.insert(mat[1].to_string());
    }

    // We'll store the final traits here
    let mut traits_map: HashMap<String, TraitInfo> = HashMap::new();

    // 3. Find each trait’s braced body and discover references
    let mut search_start = 0;
    while let Some(m) = trait_decl_re.find_at(&contents, search_start) {
        let caps = trait_decl_re.captures(&contents[m.start()..]).unwrap();
        let trait_name = caps[1].to_string();
        let trait_name_start = m.start();

        // Find the first '{' after the trait
        let after_trait_index = trait_name_start + m.end() - m.start();
        let brace_pos = match contents[after_trait_index..].find('{') {
            Some(pos) => after_trait_index + pos,
            None => {
                // No brace => skip
                search_start = after_trait_index + 1;
                continue;
            }
        };

        // Extract the braced block text
        let (body_text, end_pos) = match extract_braced_block(&contents, brace_pos) {
            Some((b, e)) => (b, e),
            None => {
                search_start = brace_pos + 1;
                continue;
            }
        };

        // Collect references (any known trait names appearing in the body)
        let references = find_references(&body_text, &all_trait_names, &trait_name);

        // Insert into map
        traits_map.insert(
            trait_name.clone(),
            TraitInfo {
                name: trait_name,
                references,
            },
        );

        // Move on
        search_start = end_pos + 1;
    }

    // 4. Generate Mermaid output (no methods, just class { } and references)
    let mut mermaid = String::new();
    mermaid.push_str("```mermaid\n");
    mermaid.push_str("classDiagram\n\n");

    // Class definitions without methods
    for tinfo in traits_map.values() {
        mermaid.push_str(&format!("class {} {{\n", tinfo.name));
        mermaid.push_str("}\n\n");
    }

    // Add references: T --> U
    let mut edges = BTreeSet::new();
    for tinfo in traits_map.values() {
        for r in &tinfo.references {
            edges.insert(format!("{} --> {}", tinfo.name, r));
        }
    }
    for edge in edges {
        mermaid.push_str(&edge);
        mermaid.push('\n');
    }

    mermaid.push_str("\n```");

    let mermaid_clean = mermaid
        .replace("&'a User", "&User")
        .replace("value: &'a User", "value: &User");

    // 5. Write to architecture_light.md
    fs::write("architecture_light.md", mermaid_clean)?;

    println!("Generated architecture_light.md with lighter diagram (no method signatures).");
    Ok(())
}

/// Extracts the text within braces at `brace_start`, respecting nested braces.
/// Returns (body, end_index) where `end_index` is the position of the closing brace.
fn extract_braced_block(contents: &str, brace_start: usize) -> Option<(String, usize)> {
    let mut brace_count = 0;
    let mut end_pos = brace_start;
    let chars: Vec<_> = contents.char_indices().collect();
    let mut in_brace_region = false;

    for i in 0..chars.len() {
        let (idx, ch) = chars[i];
        if idx < brace_start {
            continue;
        }
        if !in_brace_region {
            if idx == brace_start && ch == '{' {
                in_brace_region = true;
                brace_count = 1;
            }
            continue;
        } else {
            if ch == '{' {
                brace_count += 1;
            } else if ch == '}' {
                brace_count -= 1;
                if brace_count == 0 {
                    end_pos = idx;
                    let body = &contents[brace_start + 1..end_pos];
                    return Some((body.to_string(), end_pos));
                }
            }
        }
    }
    None
}

/// Finds references to other trait names in body_text.
/// Skips references to itself.
fn find_references(
    body_text: &str,
    all_names: &HashSet<String>,
    self_name: &str,
) -> BTreeSet<String> {
    let mut refs = BTreeSet::new();
    for candidate in all_names {
        if candidate == self_name {
            continue;
        }
        if body_text.contains(candidate) {
            refs.insert(candidate.clone());
        }
    }
    refs
}
