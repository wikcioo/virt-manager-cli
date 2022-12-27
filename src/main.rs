use std::io::{self, Write};
use std::process::exit;

const VERSION: &str = "0.1.0";

fn main() {
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
            println!("Version: {}", VERSION);
        }
        "quit" => {
            exit(0);
        }
        "" => {
            return;
        }
        _ => {
            println!("Command '{input}' is not supported!");
            return;
        }
    }
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
