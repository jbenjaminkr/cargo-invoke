use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Run 'cargo invoke architecture'
    Command::new("cargo")
        .arg("invoke")
        .arg("architecture")
        .status()?;

    // Run 'cargo invoke class_diagram'
    Command::new("cargo")
        .arg("invoke")
        .arg("class_diagram")
        .status()?;

    // Run 'cargo invoke visualize' with parameter class_diagram
    Command::new("cargo")
        .arg("invoke")
        .arg("view")
        .arg("class_diagram")
        .status()?;

    Ok(())
}
