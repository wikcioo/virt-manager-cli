use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

const VERSION: &str = "0.1.0";
const PROGRAM_DIR_NAME: &str = ".virt-manager";

#[derive(Serialize, Deserialize)]
struct VmDetails {
    name: String,
    smp: u8,
    ram: u8,
    kvm: bool,
}

impl std::fmt::Display for VmDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Name: {} | Smp: {} vcpus | Ram: {}GB | Kvm: {}",
            self.name, self.smp, self.ram, self.kvm
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
    let args: Vec<&str> = input.split_whitespace().collect();

    match args[0] {
        "help" => {
            print_usage();
        }
        "version" => {
            println!("Version: {VERSION}");
        }
        "dusage" => {
            let size_bytes = get_dir_size(&get_program_directory_abs_path());
            println!(
                "Disk usage: {:.2}GB",
                size_bytes as f64 / 1024_f64.powf(3.0)
            );
        }
        "list" => {
            let vms = get_vm_details();
            for vm in vms {
                println!("{vm}");
            }
        }
        "start" => {
            if args.len() > 1 {
                start_vm(Some(args[1]));
            } else {
                start_vm(None);
            }
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

fn get_dir_size(path: &str) -> u64 {
    let mut total_size: u64 = 0;

    for entry in fs::read_dir(path).unwrap() {
        let entry = entry.unwrap();

        if entry.file_type().unwrap().is_file() {
            total_size += entry.metadata().unwrap().len();
        } else if entry.file_type().unwrap().is_dir() {
            total_size += get_dir_size(entry.path().to_str().unwrap());
        }
    }

    total_size
}

fn start_vm(vm_name: Option<&str>) {
    let name;

    if let Some(n) = vm_name {
        name = n.to_string();
    } else {
        print!("Enter the virtual machine name: ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read the name");

        name = input.trim().to_lowercase();
    }

    let vms = get_vm_details();

    if let Some(vm) = vms.iter().find(|vm| vm.name == name) {
        let mut vm_args: Vec<&str> = vec![];

        if vm.kvm {
            vm_args.push("-enable-kvm");
        }

        let ram_str = vm.ram.to_string() + "G";
        vm_args.extend(["-m", &ram_str].iter());

        let smp_str = vm.smp.to_string();
        vm_args.extend(["-smp", &smp_str].iter());

        vm_args.extend(["-boot", "menu=on"].iter());

        let drive =
            "file=".to_owned() + &get_program_directory_abs_path() + "/" + &name + "/image.img";
        vm_args.extend(["-drive", &drive].iter());

        vm_args.extend(["-cpu", "host"].iter());
        vm_args.extend(["-device", "virtio-vga-gl"].iter());
        vm_args.extend(["-display", "sdl,gl=on"].iter());

        let mut child_process = Command::new("qemu-system-x86_64")
            .args(vm_args)
            .spawn()
            .expect("Failed to execute the process.");

        let exit_status = child_process
            .wait()
            .expect("Failed to wait for the child process");

        println!("{exit_status}");
    } else {
        println!("{name} not found!");
    }
}

fn get_vm_details() -> Vec<VmDetails> {
    let mut vm_details: Vec<VmDetails> = vec![];
    let program_directory = get_program_directory_abs_path();

    if let Ok(entries) = fs::read_dir(&program_directory) {
        for entry in entries.flatten() {
            if entry.file_type().unwrap().is_dir() {
                if let Some(file_name) = entry.file_name().to_str() {
                    let vm_path = program_directory.clone() + "/" + file_name;
                    vm_details.push(read_vm_details(&vm_path));
                }
            }
        }
    }

    vm_details
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
        smp: details.smp,
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
