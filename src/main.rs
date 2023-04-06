use serde::{Deserialize, Serialize};
use serde_json::json;
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
    os_installed: bool,
}

impl std::fmt::Display for VmDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Name: {} | Smp: {} vcpus | Ram: {}GB | Kvm: {} | Os installed: {}",
            self.name, self.smp, self.ram, self.kvm, self.os_installed
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
        "create" => {
            create_vm();
        }
        "delete" => {
            if args.len() > 1 {
                delete_vm(Some(args[1]));
            } else {
                delete_vm(None);
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

fn create_vm() {
    let name = get_user_input_as_str("Name: ").to_lowercase();

    let smp: u8 = get_user_input_until_valid("Virtual CPUs: ");

    let ram: u8 = get_user_input_until_valid("Ram size in GB: ");

    let input: String = get_user_input_until_valid("Enable kvm [Y/n]: ");
    let kvm = input.is_empty() || input == "Y" || input == "y";

    let image_size: String = get_user_input_until_valid("Image size in GB: ");

    let config = json!({
        "name": name,
        "smp": smp,
        "ram": ram,
        "kvm": kvm,
        "os_installed": false
    });

    let vm_dir = get_program_directory_abs_path() + "/" + &name;
    if let Err(e) = fs::create_dir(&vm_dir) {
        eprintln!("Error creating directory: {e}");
        return;
    }

    println!("Created directory for {name}");

    let mut file = match File::create(vm_dir.clone() + "/config.json") {
        Ok(val) => val,
        Err(e) => {
            eprintln!("Error creating {name}/config.json: {e}");
            return;
        }
    };

    let bytes_to_write = [config.to_string().as_bytes(), b"\n"].concat();
    if let Err(e) = file.write(&bytes_to_write) {
        eprintln!("Failed to write to {name}/config.json: {e}");
    }

    let image_path = vm_dir + "/image.img";
    let image_size = image_size + "G";

    let mut child_process = Command::new("qemu-img")
        .args(["create", "-f", "qcow2", &image_path, &image_size].iter())
        .spawn()
        .expect("Failed to spawn qemu-img");

    let exit_status = child_process
        .wait()
        .expect("Failed to wait for the child process");

    println!("{exit_status}");
}

fn get_user_input_until_valid<T: std::str::FromStr>(prompt: &str) -> T
where
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let mut input: T;
    'l: loop {
        let failed;

        input = match get_user_input_as_str(prompt).parse() {
            Ok(val) => {
                failed = false;
                val
            }
            Err(e) => {
                eprintln!("Error: {e}");
                continue 'l;
            }
        };

        if !failed {
            break;
        }
    }

    input
}

fn get_user_input_as_str(prompt: &str) -> String {
    let mut input = String::new();

    print!("{prompt}");
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");

    input.trim().to_owned()
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

fn get_file_size(path_to_file: &str) -> Result<u64, std::io::Error> {
    match fs::metadata(path_to_file) {
        Ok(val) => Ok(val.len()),
        Err(e) => Err(e),
    }
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
    let vm_path = get_program_directory_abs_path() + "/" + &name;

    if let Some(vm) = vms.iter().find(|vm| vm.name == name) {
        let mut vm_args: Vec<&str> = vec![];

        if vm.kvm {
            vm_args.push("-enable-kvm");
        }

        let iso_path = vm_path.clone() + "/" + &name + ".iso";
        let image_size = get_file_size(&(vm_path.clone() + "/image.img")).unwrap();
        if !vm.os_installed {
            if !Path::new(&iso_path).exists() {
                eprintln!("Missing {iso_path} file!");
                return;
            }
            vm_args.extend(["-cdrom", &iso_path].iter());
        }

        let ram_str = vm.ram.to_string() + "G";
        vm_args.extend(["-m", &ram_str].iter());

        let smp_str = vm.smp.to_string();
        vm_args.extend(["-smp", &smp_str].iter());

        vm_args.extend(["-boot", "menu=on"].iter());

        let drive = "file=".to_owned() + &vm_path + "/image.img";
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

        let image_size_after_run = get_file_size(&(vm_path.clone() + "/image.img")).unwrap();
        if image_size_after_run != image_size && !vm.os_installed {
            println!("Marking {} as vm with os installed", vm.name);
            vm_mark_os_installed(&(vm_path + "/config.json"));
        }

        println!("{exit_status}");
    } else {
        println!("{name} not found!");
    }
}

fn delete_vm(vm_name: Option<&str>) {
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

    let vm_dir = get_program_directory_abs_path() + "/" + &name;
    if Path::new(&vm_dir).exists() {
        fs::remove_dir_all(vm_dir).unwrap();
        println!("Deleted '{name}' virtual machine from disk");
    } else {
        eprintln!("Virtual machine '{name}' does not exist!");
    }
}

fn vm_mark_os_installed(file_path: &str) {
    let mut file = File::open(file_path).unwrap();
    let mut file_content = String::new();
    file.read_to_string(&mut file_content).unwrap();

    let mut vm: VmDetails = serde_json::from_str(&file_content).unwrap();
    vm.os_installed = true;

    let binding = serde_json::to_string(&vm).unwrap();
    let new_file_content = [binding.as_bytes(), b"\n"].concat();

    let mut file = File::create(file_path).unwrap();
    file.write_all(&new_file_content).unwrap();
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
        os_installed: details.os_installed,
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
