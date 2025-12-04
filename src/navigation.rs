use std::env;
use std::io::{self, Write};

pub fn change_directory() {
    print!("Enter directory to change into: ");
    io::stdout().flush().unwrap();
    let mut dir = String::new();
    io::stdin().read_line(&mut dir).unwrap();
    let dir = dir.trim();

    match env::set_current_dir(dir) {
        Ok(_) => println!("Changed directory to {}", dir),
        Err(e) if e.kind() == io::ErrorKind::NotFound => println!("Error: Directory not found."),
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => println!("Error: Permission denied."),
        Err(e) => println!("Unexpected error: {}", e),
    }
}
