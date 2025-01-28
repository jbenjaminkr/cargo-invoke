use regex::Regex;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::io;

/// Holds information about each method in a trait.
#[derive(Debug)]
struct MethodInfo {
    name: String,
    params: String,
    return_type: String,
}

/// Represents each trait: its name, methods, and references to other traits.
#[derive(Debug)]
struct TraitInfo {
    name: String,
    methods: Vec<MethodInfo>,
    references: BTreeSet<String>,
}

fn main() -> io::Result<()> {
    // 1. Read all contents of "traits.rs"
    let contents = fs::read_to_string("traits.rs")?;

    // 2. Identify trait declarations (just the keyword "trait" and name).
    //    We'll do the bracket capturing ourselves (to handle nested braces).
    //    Pattern captures something like:
    //       pub trait TraitName
    //       or
    //       trait TraitName
    let trait_decl_re = Regex::new(r"(?:pub\s+)?trait\s+([A-Za-z0-9_]+)").unwrap();

    // Step A: Collect all trait names for reference-scanning
    // (We'll do a quick pass just to find all names, ignoring braces)
    let mut all_trait_names = HashSet::new();
    for mat in trait_decl_re.captures_iter(&contents) {
        let trait_name = mat[1].to_string();
        all_trait_names.insert(trait_name);
    }

    // We'll store the final traits here
    let mut traits_map = HashMap::new();

    // 3. Actually parse each trait’s body by matching braces manually.
    //
    // Approach:
    //   - For each match of trait_decl_re, we find the start of the '{' that follows.
    //   - Then we parse until we find the matching '}' at the same nesting level.
    //
    let mut search_start = 0;
    while let Some(m) = trait_decl_re.find_at(&contents, search_start) {
        // m.start() is the start of "trait" or "pub trait"
        // Capture the name
        let caps = trait_decl_re.captures(&contents[m.start()..]).unwrap();
        let trait_name = caps[1].to_string();

        // Absolute start index of the name in the full file
        let trait_name_start = m.start();

        // We need to find the first '{' after this match
        // Let's scan from the end of the regex match
        let after_trait_index = trait_name_start + m.end() - m.start();
        let brace_pos = match contents[after_trait_index..].find('{') {
            Some(pos) => after_trait_index + pos,
            None => {
                // No brace => no body => skip
                search_start = after_trait_index + 1;
                continue;
            }
        };

        // Now, we parse braces to find the matching '}' from brace_pos
        let (body_text, end_pos) = match extract_braced_block(&contents, brace_pos) {
            Some((b, e)) => (b, e),
            None => {
                // Could not parse braces => skip
                search_start = brace_pos + 1;
                continue;
            }
        };

        // Now we have the full text inside { ... } as `body_text`.
        // We'll store a TraitInfo, parse out methods, references, etc.
        let methods = extract_methods(&body_text);
        let references = find_references(&body_text, &all_trait_names, &trait_name);

        let trait_info = TraitInfo {
            name: trait_name.clone(),
            methods,
            references,
        };
        traits_map.insert(trait_name, trait_info);

        // Move search_start beyond this trait so we find the next one
        search_start = end_pos + 1;
    }

    // 4. Generate Mermaid output
    let mut mermaid = String::new();
    mermaid.push_str("```mermaid\n");
    mermaid.push_str("classDiagram\n\n");

    // Print each trait as a class with methods
    for ti in traits_map.values() {
        mermaid.push_str(&format!("class {} {{\n", ti.name));
        for m in &ti.methods {
            if m.return_type.is_empty() {
                // No return
                mermaid.push_str(&format!("  + {}({})\n", m.name, m.params));
            } else {
                // With return
                mermaid.push_str(&format!("  + {}({}) {}\n", m.name, m.params, m.return_type));
            }
        }
        mermaid.push_str("}\n\n");
    }

    // Add references: T --> U
    let mut edges = BTreeSet::new();
    for ti in traits_map.values() {
        for r in &ti.references {
            let edge = format!("{} --> {}", ti.name, r);
            edges.insert(edge);
        }
    }
    for e in edges {
        mermaid.push_str(&e);
        mermaid.push('\n');
    }

    mermaid.push_str("\n```");

    let mermaid_clean = mermaid
        .replace("&'a User", "&User")
        .replace("value: &'a User", "value: &User");

    let re_lt = Regex::new(r"\s*<\s*").unwrap(); // "Arc < T" -> "Arc<T"
    let re_gt = Regex::new(r"\s*>\s*").unwrap(); // "T >" -> "T>"
    let re_amp = Regex::new(r"\s*&\s*").unwrap(); // "& self" -> "&self"

    // 2. Apply them to your existing Mermaid string `mermaid`
    let mermaid_no_spaces = re_lt.replace_all(&mermaid_clean, "<");
    let mermaid_no_spaces = re_gt.replace_all(&mermaid_no_spaces, ">");
    let mermaid_no_spaces = re_amp.replace_all(&mermaid_no_spaces, "&");

    // 3. Write the cleaned string to architecture.md
    fs::write("architecture.md", mermaid_no_spaces.as_ref())?;

    // 6. Print success
    println!("Generated architecture.md. Below is a snippet showing the captured methods:\n");

    // Print a small snippet to show the user the methods we found:
    for ti in traits_map.values() {
        println!("Trait: {}", ti.name);
        for m in &ti.methods {
            if m.return_type.is_empty() {
                println!("   fn {}({});", m.name, m.params);
            } else {
                println!("   fn {}({}) -> {};", m.name, m.params, m.return_type);
            }
        }
        println!("");
    }

    Ok(())
}

/// Extracts the text within the braces starting at `brace_start` in `contents`,
/// including nested braces. Returns (body, end_index), where `end_index` is the
/// position of the closing '}' in `contents`.
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
            // The first brace is at brace_start
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
                    // Extract everything between brace_start+1 and end_pos-1
                    let body = &contents[brace_start + 1..end_pos];
                    return Some((body.to_string(), end_pos));
                }
            }
        }
    }
    None
}

/// Extract methods from the trait body text.
/// We look for lines of form:
///   fn <method>(...) -> ...;
/// Possibly multi-line. We'll use a single regex pass with DOT matches newlines.
fn extract_methods(body_text: &str) -> Vec<MethodInfo> {
    // We'll try a simpler approach:
    // "fn <name> (paramstuff) -> optionalret ;"
    // with possible whitespace or newlines in between
    //
    // We'll use a lazy approach that won't break on nested generics, but good enough
    // for your standard trait definitions.
    //
    // Regex breakdown:
    //   fn\s+([A-Za-z0-9_]+)  captures method name
    //   \(\s*(.*?)\s*\)       captures everything in parentheses (non-greedy)
    //   (?:\s*->\s*([^;]+))?  optionally captures return type up to a semicolon
    //   \s*;                  end of signature
    let mut methods = Vec::new();
    let method_re =
        Regex::new(r"fn\s+([A-Za-z0-9_]+)\s*\(\s*(.*?)\s*\)(?:\s*->\s*([^;]+))?\s*;").unwrap();

    for caps in method_re.captures_iter(body_text) {
        let name = caps[1].trim().to_string();
        let params = caps[2].trim().to_string();
        let return_type = caps.get(3).map_or("", |m| m.as_str()).trim().to_string();

        methods.push(MethodInfo {
            name,
            params,
            return_type,
        });
    }
    methods
}

/// Find references to other trait names in the given body text.
/// We'll mark it as referencing "X" if the text contains "X".
/// Skip references to itself.
fn find_references(
    body_text: &str,
    all_names: &HashSet<String>,
    self_name: &str,
) -> BTreeSet<String> {
    let mut refs = BTreeSet::new();
    for other in all_names {
        if other == self_name {
            continue;
        }
        // naive string contains
        if body_text.contains(other) {
            refs.insert(other.clone());
        }
    }
    refs
}
