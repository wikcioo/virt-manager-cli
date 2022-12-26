use std::io::{self, Write};

fn main() {
    loop {
        let input = get_user_input();
        parse_user_input(input);
    }
}

fn parse_user_input(input: String) {
    println!("You typed: {input}");
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
