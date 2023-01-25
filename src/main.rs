use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::exit;

const VERSION: &str = "0.1.0";
const PROGRAM_DIR_NAME: &str = ".virt-manager";

fn main() {
    init_program_directory();
    start_interactive_mode();
}

fn start_interactive_mode() {
    loop {
        let input = get_user_input();
        parse_user_input(&input);
    }
}

fn parse_user_input(input: &str) {
    match input {
        "help" => {
            print_usage();
        }
        "version" => {
            println!("Version: {VERSION}");
        }
        "quit" => {
            exit(0);
        }
        "" => {}
        _ => {
            println!("Command '{input}' is not supported!");
        }
    }
}

fn init_program_directory() {
    let program_directory_abs_path = get_program_directory_abs_path();

    let program_directory = Path::new(&program_directory_abs_path);
    if !program_directory.exists() {
        create_program_directory(&program_directory_abs_path);
    }
}

fn get_program_directory_abs_path() -> String {
    let home_dir = match env::var("HOME") {
        Ok(val) => val,
        Err(e) => panic!("Failed to get HOME directory: {e}"),
    };

    let mut prog_path = PathBuf::new();
    prog_path.push(home_dir);
    prog_path.push(PROGRAM_DIR_NAME);

    if let Some(path) = prog_path.to_str() {
        String::from(path)
    } else {
        panic!("Failed to get program directory!");
    }
}

fn create_program_directory(path: &str) {
    if let Err(e) = fs::create_dir(path) {
        eprintln!("Error creating directory: {e}");
    }

    println!("Created program directory");
}

fn get_user_input() -> String {
    let mut input = String::new();
    print!("[virt-manager]\x1b[32m#\x1b[0m ");

    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    input.trim().to_string()
}

fn print_usage() {
    println!("virt-manager-cli {VERSION}");
    println!("Virtual machines manager command line utility");
    println!();
    println!("USAGE:");
    println!("  virt-manager-cli [FLAGS] [OPTIONS] ARGUMENTS");
    println!();
    println!("FLAGS:");
    println!("  -h, --help       Prints help information");
    println!("  -v, --version    Prints version information");
    println!();
    println!("OPTIONS:");
    println!("  -i, --interactive    Starts the interactive mode");
}
