use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::exit;

const VERSION: &str = "0.1.0";
const PROGRAM_DIR_NAME: &str = ".virt-manager";

#[derive(Serialize, Deserialize)]
struct VmDetails {
    name: String,
    cpu: u8,
    ram: u8,
    kvm: bool,
}

impl std::fmt::Display for VmDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Name: {} | Cpu: {} cores | Ram: {}GB | Kvm: {}",
            self.name, self.cpu, self.ram, self.kvm
        )
    }
}

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
        "list" => {
            list_vms();
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

fn list_vms() {
    let program_directory = get_program_directory_abs_path();
    if let Ok(entries) = fs::read_dir(&program_directory) {
        for entry in entries.flatten() {
            if entry.file_type().unwrap().is_dir() {
                if let Some(file_name) = entry.file_name().to_str() {
                    let vm_path = program_directory.clone() + "/" + file_name;
                    let vm = read_vm_details(&vm_path);
                    println!("{vm}");
                }
            }
        }
    }
}

fn read_vm_details(path: &str) -> VmDetails {
    let config_path = path.to_owned() + "/config.json";
    let mut file =
        File::open(config_path).expect("Failed to open configuration file for {config_path}");

    let mut data = String::new();
    file.read_to_string(&mut data)
        .expect("Unable to read the file");

    let details: VmDetails = serde_json::from_str(&data).expect("Failed to parse json file");

    VmDetails {
        name: details.name,
        cpu: details.cpu,
        ram: details.ram,
        kvm: details.kvm,
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
