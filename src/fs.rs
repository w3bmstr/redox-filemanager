use std::fs;
use std::io::{self, Write};
use std::time::UNIX_EPOCH;

#[cfg(target_os = "windows")]
use std::os::windows::fs::MetadataExt;

pub fn list_files() {
    println!("Listing files in current directory:");

    match fs::read_dir(".") {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let metadata = match entry.metadata() {
                        Ok(m) => m,
                        Err(e) => {
                            println!("Could not read metadata: {}", e);
                            continue;
                        }
                    };

                    let file_name = entry.file_name();
                    let file_name_str = file_name.to_string_lossy();

                    // Hidden detection
                    let mut is_hidden = file_name_str.starts_with('.'); // dotfiles
                    #[cfg(target_os = "windows")]
                    {
                        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
                        if metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0 {
                            is_hidden = true;
                        }
                    }

                    let file_type = if metadata.is_dir() { "Directory" } else { "File" };
                    let size = metadata.len();
                    let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
                    let readonly = metadata.permissions().readonly();

                    if is_hidden {
                        println!(
                            "{} (hidden) | {} | {} bytes | modified {:?} | readonly: {}",
                            file_name_str, file_type, size, modified, readonly
                        );
                    } else {
                        println!(
                            "{} | {} | {} bytes | modified {:?} | readonly: {}",
                            file_name_str, file_type, size, modified, readonly
                        );
                    }
                }
            }
        }
        Err(e) => println!("Error reading directory: {}", e),
    }
}

pub fn search_files() {
    print!("Enter search term: ");
    io::stdout().flush().unwrap();
    let mut term = String::new();
    io::stdin().read_line(&mut term).unwrap();
    let term = term.trim().to_lowercase();

    match fs::read_dir(".") {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let file_name = entry.file_name();
                    let file_name_str = file_name.to_string_lossy();

                    if file_name_str.to_lowercase().contains(&term) {
                        let metadata = entry.metadata().unwrap();
                        let file_type = if metadata.is_dir() { "Directory" } else { "File" };
                        let size = metadata.len();
                        let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
                        let readonly = metadata.permissions().readonly();

                        // Hidden detection
                        let mut is_hidden = file_name_str.starts_with('.');
                        #[cfg(target_os = "windows")]
                        {
                            const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
                            if metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0 {
                                is_hidden = true;
                            }
                        }

                        if is_hidden {
                            println!(
                                "{} (hidden) | {} | {} bytes | modified {:?} | readonly: {}",
                                file_name_str, file_type, size, modified, readonly
                            );
                        } else {
                            println!(
                                "{} | {} | {} bytes | modified {:?} | readonly: {}",
                                file_name_str, file_type, size, modified, readonly
                            );
                        }
                    }
                }
            }
        }
        Err(e) => println!("Error searching directory: {}", e),
    }
}
