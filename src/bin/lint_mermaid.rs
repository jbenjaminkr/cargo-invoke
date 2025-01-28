use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Debug)]
struct MermaidFormatter {
    formatted: String,
    indent_level: usize,
    in_subgraph: bool,
    class_assignments: Vec<String>,
}

impl MermaidFormatter {
    fn new() -> Self {
        MermaidFormatter {
            formatted: String::new(),
            indent_level: 0,
            in_subgraph: false,
            class_assignments: Vec::new(),
        }
    }

    fn add_line(&mut self, line: &str) {
        if !line.is_empty() {
            self.formatted.push_str(&"    ".repeat(self.indent_level));
            self.formatted.push_str(line);
            self.formatted.push('\n');
        }
    }

    fn format_mermaid(&mut self, input: &str) {
        let lines: Vec<&str> = input.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let trimmed = lines[i].trim();
            if trimmed.is_empty() {
                i += 1;
                continue;
            }

            // Handle initialization blocks
            if trimmed.starts_with("%%{") {
                self.add_line(trimmed);
                i += 1;
                continue;
            }

            // Handle comments
            if trimmed.starts_with("%%") {
                self.add_line(trimmed);
                i += 1;
                continue;
            }

            // Handle class assignments (:::)
            if trimmed.contains(":::") {
                self.class_assignments.push(trimmed.to_string());
                i += 1;
                continue;
            }

            // Handle direction statements
            if trimmed.starts_with("direction ") {
                self.add_line(trimmed);
                i += 1;
                continue;
            }

            match trimmed {
                s if s.starts_with("subgraph") => {
                    self.add_line(s);
                    self.indent_level += 1;
                    self.in_subgraph = true;
                }
                "end" => {
                    self.indent_level = self.indent_level.saturating_sub(1);
                    self.add_line("end");
                    if self.in_subgraph {
                        self.in_subgraph = false;
                    }
                }
                // Root level definitions go at base indentation
                s if s.starts_with("class ")
                    || s.starts_with("classDef ")
                    || s.starts_with("style ") =>
                {
                    self.add_line(s);
                }
                // Diagram type declarations
                s if s.starts_with("graph ")
                    || s.starts_with("sequenceDiagram")
                    || s.starts_with("classDiagram")
                    || s.starts_with("stateDiagram")
                    || s.starts_with("gantt")
                    || s.starts_with("pie")
                    || s.starts_with("flowchart")
                    || s.starts_with("erDiagram") =>
                {
                    self.add_line(s);
                }
                s => {
                    self.add_line(s);
                }
            }
            i += 1;
        }

        // Add sorted class assignments at the end
        if !self.class_assignments.is_empty() {
            self.formatted.push('\n');
            self.class_assignments.sort();
            let assignments = std::mem::take(&mut self.class_assignments);
            for assignment in assignments {
                self.add_line(&assignment);
            }
        }
    }
}

pub fn format_mermaid(input: &str) -> String {
    let mut formatter = MermaidFormatter::new();
    formatter.format_mermaid(input);
    formatter.formatted
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();

    // Parse command line arguments
    let mut in_place = false;
    let mut input_file = None;
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "-i" | "--inplace" => {
                in_place = true;
                i += 1;
            }
            arg => {
                if input_file.is_none() {
                    input_file = Some(arg.to_string());
                } else {
                    eprintln!("Usage: {} [--inplace|-i] <input_file>", args[0]);
                    std::process::exit(1);
                }
                i += 1;
            }
        }
    }

    let input_file = match input_file {
        Some(file) => file,
        None => {
            eprintln!("Usage: {} [--inplace|-i] <input_file>", args[0]);
            std::process::exit(1);
        }
    };

    let input_path = Path::new(&input_file);
    let input_content = fs::read_to_string(input_path)?;

    let formatted = format_mermaid(&input_content);

    if in_place {
        // Write back to the same file
        fs::write(input_path, formatted)?;
        println!("File formatted in-place: {}", input_path.display());
    } else {
        // Create output filename by appending "_formatted" before the extension
        let output_path = input_path.with_file_name(format!(
            "{}_formatted{}",
            input_path.file_stem().unwrap().to_str().unwrap(),
            input_path.extension().map_or("", |ext| {
                if ext == "mermaid" {
                    ".mermaid"
                } else {
                    ""
                }
            })
        ));

        fs::write(&output_path, formatted)?;
        println!("Formatted file saved as: {}", output_path.display());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_formatting() {
        let input = "graph TD\nA-->B\n   C --> D\n";
        let expected = "graph TD\n    A-->B\n    C --> D\n";
        assert_eq!(format_mermaid(input), expected);
    }

    #[test]
    fn test_subgraph_formatting() {
        let input = "graph TD\nsubgraph One\nA-->B\nend\n";
        let expected = "graph TD\nsubgraph One\n    A-->B\nend\n";
        assert_eq!(format_mermaid(input), expected);
    }

    #[test]
    fn test_nested_subgraphs() {
        let input = "graph TD\nsubgraph One\nsubgraph Two\nA-->B\nend\nend\n";
        let expected = "graph TD\nsubgraph One\n    subgraph Two\n        A-->B\n    end\nend\n";
        assert_eq!(format_mermaid(input), expected);
    }

    #[test]
    fn test_direction_statements() {
        let input = "graph TD\nsubgraph One\ndirection LR\nA-->B\nend\n";
        let expected = "graph TD\nsubgraph One\n    direction LR\n    A-->B\nend\n";
        assert_eq!(format_mermaid(input), expected);
    }

    #[test]
    fn test_class_assignments() {
        let input = "graph TD\nA-->B:::classB\nC:::classC-->D\n";
        let expected = "graph TD\n    A-->B\n    C-->D\n\nA-->B:::classB\nC:::classC-->D\n";
        assert_eq!(format_mermaid(input), expected);
    }
}
