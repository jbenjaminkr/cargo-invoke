use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn main() {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <dir_A> <dir_B>", args[0]);
        std::process::exit(1);
    }

    let dir_a = &args[1];
    let dir_b = &args[2];

    // Construct paths for architecture directories
    let path_a = Path::new(dir_a).join("architecture");
    let path_b = Path::new(dir_b).join("architecture");

    // Verify both directories exist
    if !path_a.exists() {
        eprintln!("Error: {}/architecture directory does not exist", dir_a);
        std::process::exit(1);
    }
    if !path_b.exists() {
        eprintln!("Error: {}/architecture directory does not exist", dir_b);
        std::process::exit(1);
    }

    // Change to the parent directory of the first path to make diff output cleaner
    env::set_current_dir(Path::new(dir_a).parent().unwrap_or(Path::new(".")))
        .expect("Failed to change directory");

    // Generate diff using system diff command
    // Using relative paths and -Nr to avoid subdirectory listings
    let output = Command::new("diff")
        .arg("-Nr") // -N: treat absent files as empty, -r: recursive
        .arg(path_a)
        .arg(path_b)
        .output()
        .expect("Failed to execute diff command");

    // Create output filename
    let diff_filename = format!("{}.{}.diff", dir_a, dir_b);

    // Write diff output to file
    let mut file = fs::File::create(&diff_filename).expect("Failed to create output file");

    file.write_all(&output.stdout)
        .expect("Failed to write to output file");

    if output.status.success() {
        println!("No differences found");
    } else if !output.stdout.is_empty() {
        println!("Diff generated successfully: {}", diff_filename);
    } else {
        // Write stderr to file as well if there were any errors
        file.write_all(&output.stderr)
            .expect("Failed to write errors to output file");

        println!("Error generating diff. Check {} for details", diff_filename);
    }
}
