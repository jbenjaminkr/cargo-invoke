use std::process::{exit, Command};

fn main() {
    let commands: Vec<(&str, Vec<&str>)> = vec![
        ("cargo", vec!["invoke", "architecture"]),
        ("cargo", vec!["invoke", "structure", "structs"]),
        ("cargo", vec!["invoke", "visualize", "structs"]),
    ];

    for (command, args) in commands {
        println!("Running: {} {}", command, args.join(" "));
        let status = Command::new(command)
            .args(&args)
            .status()
            .expect("Failed to execute command");

        if !status.success() {
            eprintln!(
                "Command '{}' failed with exit code: {}",
                format!("{} {}", command, args.join(" ")),
                status.code().unwrap_or(-1)
            );
            exit(status.code().unwrap_or(1));
        }
    }

    println!("All commands completed successfully!");
}
