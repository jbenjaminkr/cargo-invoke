use std::env;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Usage: {} <SOL_DIR1> <SOL_DIR2>", args[0]);
        return;
    }

    let dir1 = &args[1];
    let dir2 = &args[2];

    Command::new("cargo")
        .args(["invoke", "architecture", dir1])
        .status()
        .expect("Failed to execute first command");

    Command::new("cargo")
        .args(["invoke", "architecture", dir2])
        .status()
        .expect("Failed to execute second command");

    Command::new("diff")
        .args([
            &format!("{}/architecture", dir1),
            &format!("{}/architecture", dir2),
        ])
        .status()
        .expect("Failed to execute diff");
}
