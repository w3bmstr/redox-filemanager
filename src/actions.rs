use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use crate::error::handle_error;

pub fn copy_file() {
    print!("Enter the file to copy: ");
    io::stdout().flush().unwrap();
    let mut src = String::new();
    io::stdin().read_line(&mut src).unwrap();
    let src = src.trim();

    print!("Enter the new file name: ");
    io::stdout().flush().unwrap();
    let mut dst = String::new();
    io::stdin().read_line(&mut dst).unwrap();
    let dst = dst.trim();

    match fs::copy(src, dst) {
        Ok(_) => println!("File copied successfully."),
        Err(e) => handle_error(e, src),
    }
}

pub fn delete_file() {
    print!("Enter the file to delete: ");
    io::stdout().flush().unwrap();
    let mut target = String::new();
    io::stdin().read_line(&mut target).unwrap();
    let target = target.trim();

    match fs::remove_file(target) {
        Ok(_) => println!("File deleted successfully."),
        Err(e) => handle_error(e, target),
    }
}

pub fn rename_file() {
    print!("Enter the current file name: ");
    io::stdout().flush().unwrap();
    let mut old = String::new();
    io::stdin().read_line(&mut old).unwrap();
    let old = old.trim();

    print!("Enter the new file name: ");
    io::stdout().flush().unwrap();
    let mut new = String::new();
    io::stdin().read_line(&mut new).unwrap();
    let new = new.trim();

    match fs::rename(old, new) {
        Ok(_) => println!("Renamed {} to {}", old, new),
        Err(e) => handle_error(e, old),
    }
}

pub fn move_file() {
    print!("Enter the file to move: ");
    io::stdout().flush().unwrap();
    let mut src = String::new();
    io::stdin().read_line(&mut src).unwrap();
    let src = src.trim();

    print!("Enter the destination path: ");
    io::stdout().flush().unwrap();
    let mut dst = String::new();
    io::stdin().read_line(&mut dst).unwrap();
    let dst = dst.trim();

    match fs::copy(src, dst) {
        Ok(_) => match fs::remove_file(src) {
            Ok(_) => println!("Moved {} to {}", src, dst),
            Err(e) => handle_error(e, src),
        },
        Err(e) => handle_error(e, src),
    }
}

pub fn batch_delete() {
    println!("Enter files to delete (type 'done' when finished):");
    let mut files = Vec::new();

    loop {
        print!("File: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let file = input.trim();
        if file.eq_ignore_ascii_case("done") {
            break;
        }
        files.push(file.to_string());
    }

    for file in files {
        match fs::remove_file(&file) {
            Ok(_) => println!("Deleted {}", file),
            Err(e) => handle_error(e, &file),
        }
    }
}

pub fn batch_copy() {
    println!("Enter files to copy (type 'done' when finished):");
    let mut files = Vec::new();

    loop {
        print!("File: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let file = input.trim();
        if file.eq_ignore_ascii_case("done") {
            break;
        }
        files.push(file.to_string());
    }

    print!("Enter destination directory: ");
    io::stdout().flush().unwrap();
    let mut dst_dir = String::new();
    io::stdin().read_line(&mut dst_dir).unwrap();
    let dst_dir = dst_dir.trim();

    for file in files {
        let mut dst_path = PathBuf::from(dst_dir);
        dst_path.push(&file);
        match fs::copy(&file, &dst_path) {
            Ok(_) => println!("Copied {} to {:?}", file, dst_path),
            Err(e) => handle_error(e, &file),
        }
    }
}

pub fn batch_rename() {
    println!("Enter old filenames (type 'done' when finished):");
    let mut old_files = Vec::new();
    loop {
        print!("Old file: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let file = input.trim();
        if file.eq_ignore_ascii_case("done") {
            break;
        }
        old_files.push(file.to_string());
    }

    println!("Enter new filenames (same count, type 'done' when finished):");
    let mut new_files = Vec::new();
    loop {
        print!("New file: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let file = input.trim();
        if file.eq_ignore_ascii_case("done") {
            break;
        }
        new_files.push(file.to_string());
    }

    if old_files.len() != new_files.len() {
        println!("Error: number of old and new filenames must match.");
        return;
    }

    for (old, new) in old_files.iter().zip(new_files.iter()) {
        match fs::rename(old, new) {
            Ok(_) => println!("Renamed {} to {}", old, new),
            Err(e) => handle_error(e, old),
        }
    }
}

pub fn create_file() {
    use std::fs::File;

    print!("Enter new file name: ");
    io::stdout().flush().unwrap();
    let mut name = String::new();
    io::stdin().read_line(&mut name).unwrap();
    let name = name.trim();

    match File::create(name) {
        Ok(_) => println!("Created file {}", name),
        Err(e) => handle_error(e, name),
    }
}

pub fn create_directory() {
    print!("Enter new directory name: ");
    io::stdout().flush().unwrap();
    let mut name = String::new();
    io::stdin().read_line(&mut name).unwrap();
    let name = name.trim();

    match fs::create_dir(name) {
        Ok(_) => println!("Created directory {}", name),
        Err(e) => handle_error(e, name),
    }
}

pub fn delete_directory() {
    print!("Enter directory name to delete: ");
    io::stdout().flush().unwrap();
    let mut name = String::new();
    io::stdin().read_line(&mut name).unwrap();
    let name = name.trim();

    match fs::remove_dir(name) {
        Ok(_) => println!("Deleted directory {}", name),
        Err(e) => handle_error(e, name),
    }
}
