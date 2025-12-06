use crate::error;
use rand::RngCore;
use std::fs;
use std::io;
use std::io::Write;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc::Sender,
};

/// Copy a file
pub fn copy_file() {
    println!("Enter source file:");
    let mut src = String::new();
    io::stdin().read_line(&mut src).unwrap();
    let src = src.trim();

    println!("Enter destination file:");
    let mut dst = String::new();
    io::stdin().read_line(&mut dst).unwrap();
    let dst = dst.trim();

    match fs::copy(src, dst) {
        Ok(_) => println!("File copied successfully."),
        Err(e) => println!("Error copying file: {}", e),
    }
}

/// Delete a single file with confirmation
pub fn delete_file() {
    println!("Enter the file name to delete:");
    let mut filename = String::new();
    io::stdin().read_line(&mut filename).unwrap();
    let filename = filename.trim();

    println!("Are you sure you want to delete '{}'? (y/n)", filename);

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    match input.trim().to_lowercase().as_str() {
        "y" | "yes" => match fs::remove_file(filename) {
            Ok(_) => println!("File '{}' deleted successfully.", filename),
            Err(e) => error::handle_error(e, filename),
        },
        _ => println!("Delete cancelled."),
    }
}

/// Delete a directory with confirmation
pub fn delete_directory() {
    println!("Enter the directory name to delete:");
    let mut dirname = String::new();
    io::stdin().read_line(&mut dirname).unwrap();
    let dirname = dirname.trim();

    println!(
        "Are you sure you want to delete directory '{}'? (y/n)",
        dirname
    );

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");

    match input.trim().to_lowercase().as_str() {
        "y" | "yes" => match fs::remove_dir_all(dirname) {
            Ok(_) => println!("Directory '{}' deleted successfully.", dirname),
            Err(e) => error::handle_error(e, dirname),
        },
        _ => println!("Delete cancelled."),
    }
}

/// Batch delete files with one confirmation prompt
pub fn batch_delete() {
    println!("Enter file names to delete (comma separated):");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let files: Vec<&str> = input.trim().split(',').map(|s| s.trim()).collect();

    println!("You are about to delete {} files: {:?}", files.len(), files);
    println!("Are you sure? (y/n)");

    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm).unwrap();

    match confirm.trim().to_lowercase().as_str() {
        "y" | "yes" => {
            for file in files {
                match fs::remove_file(file) {
                    Ok(_) => println!("Deleted '{}'", file),
                    Err(e) => println!("Error deleting '{}': {}", file, e),
                }
            }
        }
        _ => println!("Batch delete cancelled."),
    }
}

/// Rename a file
pub fn rename_file() {
    println!("Enter current file name:");
    let mut old = String::new();
    io::stdin().read_line(&mut old).unwrap();
    let old = old.trim();

    println!("Enter new file name:");
    let mut new = String::new();
    io::stdin().read_line(&mut new).unwrap();
    let new = new.trim();

    match fs::rename(old, new) {
        Ok(_) => println!("File renamed successfully."),
        Err(e) => println!("Error renaming file: {}", e),
    }
}

/// Move a file (same as rename but with path change)
pub fn move_file() {
    println!("Enter file to move:");
    let mut src = String::new();
    io::stdin().read_line(&mut src).unwrap();
    let src = src.trim();

    println!("Enter new path:");
    let mut dst = String::new();
    io::stdin().read_line(&mut dst).unwrap();
    let dst = dst.trim();

    match fs::rename(src, dst) {
        Ok(_) => println!("File moved successfully."),
        Err(e) => println!("Error moving file: {}", e),
    }
}

/// Batch copy files
pub fn batch_copy() {
    println!("Enter file names to copy (comma separated):");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let files: Vec<&str> = input
        .trim()
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    println!("Enter destination directory:");
    let mut dst_dir = String::new();
    io::stdin().read_line(&mut dst_dir).unwrap();
    let dst_dir = dst_dir.trim();

    println!(
        "You are about to copy {} files to {}: {:?}",
        files.len(),
        dst_dir,
        files
    );
    println!("Are you sure? (y/n)");

    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm).unwrap();

    match confirm.trim().to_lowercase().as_str() {
        "y" | "yes" => {
            for file in files {
                let filename = std::path::Path::new(file)
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| file.to_string());
                let dest = std::path::Path::new(dst_dir).join(&filename);
                match fs::copy(file, &dest) {
                    Ok(_) => println!("Copied '{}' -> '{}'", file, dest.to_string_lossy()),
                    Err(e) => println!("Error copying '{}': {}", file, e),
                }
            }
        }
        _ => println!("Batch copy cancelled."),
    }
}

/// Batch rename files
pub fn batch_rename() {
    println!("Enter pairs old:new separated by commas (e.g., a.txt:b.txt,c.txt:d.txt):");
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    let pairs: Vec<&str> = input
        .trim()
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    println!("You are about to rename {} files: {:?}", pairs.len(), pairs);
    println!("Are you sure? (y/n)");

    let mut confirm = String::new();
    io::stdin().read_line(&mut confirm).unwrap();

    match confirm.trim().to_lowercase().as_str() {
        "y" | "yes" => {
            for pair in pairs {
                if let Some((old, new)) = pair.split_once(':') {
                    match fs::rename(old.trim(), new.trim()) {
                        Ok(_) => println!("Renamed '{}' -> '{}'", old.trim(), new.trim()),
                        Err(e) => println!("Error renaming '{}': {}", old.trim(), e),
                    }
                } else {
                    println!("Invalid pair '{}'", pair);
                }
            }
        }
        _ => println!("Batch rename cancelled."),
    }
}

/// Create a file
pub fn create_file() {
    println!("Enter file name to create:");
    let mut filename = String::new();
    io::stdin().read_line(&mut filename).unwrap();
    let filename = filename.trim();

    match fs::File::create(filename) {
        Ok(_) => println!("File '{}' created successfully.", filename),
        Err(e) => println!("Error creating file: {}", e),
    }
}

/// Create a directory
pub fn create_directory() {
    println!("Enter directory name to create:");
    let mut dirname = String::new();
    io::stdin().read_line(&mut dirname).unwrap();
    let dirname = dirname.trim();

    match fs::create_dir(dirname) {
        Ok(_) => println!("Directory '{}' created successfully.", dirname),
        Err(e) => println!("Error creating directory: {}", e),
    }
}

/// List archive contents via 7z CLI wrapper
pub fn archive_list_cli() {
    println!("Enter archive file path:");
    let mut path = String::new();
    io::stdin().read_line(&mut path).unwrap();
    let path = path.trim();

    match crate::archive::list_archive(path) {
        Ok(contents) => println!("Archive contents:\n{}", contents),
        Err(e) => println!("Error listing archive: {}", e),
    }
}

/// Extract archive via 7z CLI wrapper
pub fn archive_extract_cli() {
    println!("Enter archive file path:");
    let mut path = String::new();
    io::stdin().read_line(&mut path).unwrap();
    let path = path.trim();

    println!("Enter extraction destination:");
    let mut dest = String::new();
    io::stdin().read_line(&mut dest).unwrap();
    let dest = dest.trim();

    println!("Enter password (or press Enter for none):");
    let mut password = String::new();
    io::stdin().read_line(&mut password).unwrap();
    let password = if password.trim().is_empty() {
        None
    } else {
        Some(password.trim().to_string())
    };

    match crate::archive::extract_archive(path, dest, password.as_deref()) {
        Ok(msg) => println!("Archive extracted successfully:\n{}", msg),
        Err(e) => println!("Error extracting archive: {}", e),
    }
}

/// Create archive via 7z CLI wrapper
pub fn archive_create_cli() {
    println!("Enter source file or directory (or comma-separated list):");
    let mut sources_input = String::new();
    io::stdin().read_line(&mut sources_input).unwrap();
    let sources: Vec<&str> = sources_input
        .trim()
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    println!("Enter output archive path (e.g., archive.zip):");
    let mut output = String::new();
    io::stdin().read_line(&mut output).unwrap();
    let output = output.trim();

    println!("Enter format (zip, 7z, tar, gz, bz2, or press Enter for zip):");
    let mut format = String::new();
    io::stdin().read_line(&mut format).unwrap();
    let format = if format.trim().is_empty() {
        Some("zip")
    } else {
        Some(format.trim())
    };

    println!("Enter password (or press Enter for none):");
    let mut password = String::new();
    io::stdin().read_line(&mut password).unwrap();
    let password = if password.trim().is_empty() {
        None
    } else {
        Some(password.trim())
    };

    match crate::archive::create_archive(&sources, output, format, password) {
        Ok(msg) => println!("Archive created successfully:\n{}", msg),
        Err(e) => println!("Error creating archive: {}", e),
    }
}

/// Calculate hash of a file (SHA256 or BLAKE3)
pub fn calculate_hash() {
    println!("Enter file path:");
    let mut filepath = String::new();
    io::stdin().read_line(&mut filepath).unwrap();
    let filepath = filepath.trim();

    println!("Enter hash algorithm (sha256 or blake3, default sha256):");
    let mut algo = String::new();
    io::stdin().read_line(&mut algo).unwrap();
    let algo = algo.trim();

    match std::fs::read(filepath) {
        Ok(data) => {
            let hash_str = if algo.eq_ignore_ascii_case("blake3") {
                format!("BLAKE3: {}", blake3::hash(&data))
            } else {
                use sha2::Digest;
                let mut hasher = sha2::Sha256::new();
                hasher.update(&data);
                format!("SHA256: {:x}", hasher.finalize())
            };
            println!("{}\n File: {}", hash_str, filepath);
        }
        Err(e) => println!("Error reading file: {}", e),
    }
}

/// Find duplicate files in a directory by hash
pub fn find_duplicates() {
    println!("Enter directory path:");
    let mut dir = String::new();
    io::stdin().read_line(&mut dir).unwrap();
    let dir = dir.trim();

    use std::collections::HashMap;
    let mut hashes: HashMap<String, Vec<String>> = HashMap::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(data) = std::fs::read(&path) {
                    let hash = blake3::hash(&data).to_hex().to_string();
                    hashes
                        .entry(hash)
                        .or_insert_with(Vec::new)
                        .push(path.to_string_lossy().to_string());
                }
            }
        }
    }

    let mut duplicates_found = false;
    for (hash, files) in hashes.iter() {
        if files.len() > 1 {
            duplicates_found = true;
            println!("\n=== Hash: {} ===", &hash[0..16]);
            for file in files {
                println!("  {}", file);
            }
        }
    }

    if !duplicates_found {
        println!("No duplicate files found.");
    }
}

/// Secure delete a file (overwrite with random data before deletion)
pub fn secure_delete() {
    println!("Enter file path to securely delete:");
    let mut filepath = String::new();
    io::stdin().read_line(&mut filepath).unwrap();
    let filepath = filepath.trim();

    if !std::path::Path::new(filepath).exists() {
        println!("File not found: {}", filepath);
        return;
    }

    // Overwrite with random data
    use rand::RngCore;
    if let Ok(metadata) = std::fs::metadata(filepath) {
        let size = metadata.len() as usize;
        let mut random_data = vec![0u8; size];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut random_data);

        if let Err(e) = std::fs::write(filepath, random_data) {
            println!("Error overwriting file: {}", e);
            return;
        }
    }

    // Delete the file
    match std::fs::remove_file(filepath) {
        Ok(_) => println!("File '{}' securely deleted.", filepath),
        Err(e) => println!("Error deleting file: {}", e),
    }
}

/// Split a file into chunks
pub fn split_file() {
    println!("Enter file path to split:");
    let mut filepath = String::new();
    io::stdin().read_line(&mut filepath).unwrap();
    let filepath = filepath.trim();

    println!("Enter chunk size in MB (default 100):");
    let mut size_input = String::new();
    io::stdin().read_line(&mut size_input).unwrap();
    let chunk_size = size_input.trim().parse::<u64>().unwrap_or(100) * 1024 * 1024;

    match std::fs::read(filepath) {
        Ok(data) => {
            let num_chunks = (data.len() as u64 + chunk_size - 1) / chunk_size;
            for (i, chunk) in data.chunks(chunk_size as usize).enumerate() {
                let out_path = format!("{}.{:03}", filepath, i);
                match std::fs::write(&out_path, chunk) {
                    Ok(_) => println!("Created: {} ({} bytes)", out_path, chunk.len()),
                    Err(e) => println!("Error creating chunk: {}", e),
                }
            }
            println!("Split complete: {} chunks", num_chunks);
        }
        Err(e) => println!("Error reading file: {}", e),
    }
}

/// Join split file chunks back together
pub fn join_files() {
    println!("Enter base file path (without .001, .002, etc):");
    let mut base = String::new();
    io::stdin().read_line(&mut base).unwrap();
    let base = base.trim();

    println!("Enter output file path:");
    let mut output = String::new();
    io::stdin().read_line(&mut output).unwrap();
    let output = output.trim();

    use std::fs::File;
    use std::io::Write;

    match File::create(output) {
        Ok(mut out_file) => {
            let mut part_num = 0;
            loop {
                let part_path = format!("{}.{:03}", base, part_num);
                match std::fs::read(&part_path) {
                    Ok(data) => {
                        if let Err(e) = out_file.write_all(&data) {
                            println!("Error writing to output file: {}", e);
                            return;
                        }
                        part_num += 1;
                    }
                    Err(_) => break,
                }
            }
            println!("Joined {} parts into: {}", part_num, output);
        }
        Err(e) => println!("Error creating output file: {}", e),
    }
}

// --- Progress-capable worker helpers ---
/// Archive extract with progress messages. Sends status updates to `tx`. Observes `cancel` flag.
pub fn archive_extract_progress(
    path: &str,
    dest: &str,
    password: Option<&str>,
    tx: Sender<String>,
    cancel: Arc<AtomicBool>,
) {
    let _ = tx.send(format!("Starting extract: {} -> {}", path, dest));
    if cancel.load(Ordering::SeqCst) {
        let _ = tx.send("Canceled before start".to_string());
        return;
    }
    // prefer crate::archive which has fallbacks
    match crate::archive::extract_archive(path, dest, password) {
        Ok(msg) => {
            let _ = tx.send(format!("Finished: {}", msg));
        }
        Err(e) => {
            let _ = tx.send(format!("Error: {}", e));
        }
    }
}

/// Archive create with progress messages (simple starts/finishes).
pub fn archive_create_progress(
    sources: &[String],
    output: &str,
    format: Option<&str>,
    password: Option<&str>,
    tx: Sender<String>,
    cancel: Arc<AtomicBool>,
) {
    let _ = tx.send(format!("Starting archive create -> {}", output));
    if cancel.load(Ordering::SeqCst) {
        let _ = tx.send("Canceled before start".to_string());
        return;
    }
    let src_refs: Vec<&str> = sources.iter().map(|s| s.as_str()).collect();
    match crate::archive::create_archive(&src_refs, output, format, password) {
        Ok(msg) => {
            let _ = tx.send(format!("Finished: {}", msg));
        }
        Err(e) => {
            let _ = tx.send(format!("Error: {}", e));
        }
    }
}

/// Find duplicates with incremental progress messages.
pub fn find_duplicates_progress(dir: &str, tx: Sender<String>, cancel: Arc<AtomicBool>) {
    use std::collections::HashMap;
    let _ = tx.send(format!("Scanning directory: {}", dir));
    if cancel.load(Ordering::SeqCst) {
        let _ = tx.send("Canceled before scan".to_string());
        return;
    }
    let mut hashes: HashMap<String, Vec<String>> = HashMap::new();
    let mut total = 0usize;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                total += 1;
            }
        }
    }
    let _ = tx.send(format!("Found {} files, hashing...", total));
    let mut processed = 0usize;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if cancel.load(Ordering::SeqCst) {
                let _ = tx.send("Canceled during hashing".to_string());
                return;
            }
            let path = entry.path();
            if path.is_file() {
                processed += 1;
                let pstr = path.to_string_lossy().to_string();
                if let Ok(data) = std::fs::read(&path) {
                    let hash = blake3::hash(&data).to_hex().to_string();
                    hashes
                        .entry(hash)
                        .or_insert_with(Vec::new)
                        .push(pstr.clone());
                }
                let _ = tx.send(format!(
                    "Hashed {}/{}: {}",
                    processed,
                    total,
                    path.display()
                ));
            }
        }
    }
    let mut groups = 0usize;
    for (_h, files) in hashes.iter() {
        if files.len() > 1 {
            groups += 1;
            let _ = tx.send(format!("Duplicate group ({} files)", files.len()));
        }
    }
    if groups == 0 {
        let _ = tx.send("No duplicates found.".to_string());
    } else {
        let _ = tx.send(format!("Found {} duplicate groups", groups));
    }
}

/// Secure delete with progress updates: overwrites file in chunks, then deletes.
pub fn secure_delete_progress(filepath: &str, tx: Sender<String>, cancel: Arc<AtomicBool>) {
    let _ = tx.send(format!("Secure deleting: {}", filepath));
    if cancel.load(Ordering::SeqCst) {
        let _ = tx.send("Canceled before start".to_string());
        return;
    }
    if !std::path::Path::new(filepath).exists() {
        let _ = tx.send(format!("File not found: {}", filepath));
        return;
    }
    if let Ok(metadata) = std::fs::metadata(filepath) {
        let size = metadata.len() as usize;
        let mut remaining = size;
        let mut rng = rand::thread_rng();
        let mut file = match std::fs::OpenOptions::new().write(true).open(filepath) {
            Ok(f) => f,
            Err(e) => {
                let _ = tx.send(format!("Error opening file: {}", e));
                return;
            }
        };
        let chunk = 1024 * 1024; // 1MB
        let mut written = 0usize;
        while remaining > 0 {
            if cancel.load(Ordering::SeqCst) {
                let _ = tx.send("Canceled during overwrite".to_string());
                return;
            }
            let write_size = std::cmp::min(chunk, remaining);
            let mut buf = vec![0u8; write_size];
            rng.fill_bytes(&mut buf);
            if let Err(e) = file.write_all(&buf) {
                let _ = tx.send(format!("Error writing: {}", e));
                return;
            }
            remaining -= write_size;
            written += write_size;
            let pct = (written as f64 / size as f64) * 100.0;
            let _ = tx.send(format!("Overwrite progress: {:.1}%", pct));
        }
        // ensure flush
        let _ = file.flush();
        // delete
        match std::fs::remove_file(filepath) {
            Ok(_) => {
                let _ = tx.send("Secure delete complete".to_string());
            }
            Err(e) => {
                let _ = tx.send(format!("Error deleting file: {}", e));
            }
        }
    } else {
        let _ = tx.send(format!("Unable to read metadata for {}", filepath));
    }
}

/// Archive helpers (non-interactive) - return Ok(String) on success or Err(String) on error
pub fn archive_list_noninteractive(path: &str) -> Result<String, String> {
    match crate::archive::list_archive(path) {
        Ok(contents) => Ok(contents),
        Err(e) => Err(format!("{}", e)),
    }
}

#[allow(dead_code)]
pub fn archive_extract_noninteractive(
    path: &str,
    dest: &str,
    password: Option<&str>,
) -> Result<String, String> {
    match crate::archive::extract_archive(path, dest, password) {
        Ok(msg) => Ok(msg),
        Err(e) => Err(format!("{}", e)),
    }
}

#[allow(dead_code)]
pub fn archive_create_noninteractive(
    sources: &[String],
    output: &str,
    format: Option<&str>,
    password: Option<&str>,
) -> Result<String, String> {
    let src_refs: Vec<&str> = sources.iter().map(|s| s.as_str()).collect();
    match crate::archive::create_archive(&src_refs, output, format, password) {
        Ok(msg) => Ok(msg),
        Err(e) => Err(format!("{}", e)),
    }
}

/// Calculate file hash (non-interactive)
pub fn calculate_hash_noninteractive(filepath: &str, algo: &str) -> Result<String, String> {
    match std::fs::read(filepath) {
        Ok(data) => {
            let hash_str = if algo.eq_ignore_ascii_case("blake3") {
                format!("BLAKE3: {}", blake3::hash(&data))
            } else {
                use sha2::Digest;
                let mut hasher = sha2::Sha256::new();
                hasher.update(&data);
                format!("SHA256: {:x}", hasher.finalize())
            };
            Ok(format!("{}\n File: {}", hash_str, filepath))
        }
        Err(e) => Err(format!("Error reading file: {}", e)),
    }
}

/// Find duplicates (non-interactive) - returns vector of duplicate groups
#[allow(dead_code)]
pub fn find_duplicates_noninteractive(dir: &str) -> Result<Vec<Vec<String>>, String> {
    use std::collections::HashMap;
    let mut hashes: HashMap<String, Vec<String>> = HashMap::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Ok(data) = std::fs::read(&path) {
                    let hash = blake3::hash(&data).to_hex().to_string();
                    hashes
                        .entry(hash)
                        .or_insert_with(Vec::new)
                        .push(path.to_string_lossy().to_string());
                }
            }
        }
    } else {
        return Err(format!("Unable to read directory: {}", dir));
    }

    let mut groups: Vec<Vec<String>> = Vec::new();
    for (_hash, files) in hashes.into_iter() {
        if files.len() > 1 {
            groups.push(files);
        }
    }
    Ok(groups)
}

/// Secure delete (non-interactive)
#[allow(dead_code)]
pub fn secure_delete_noninteractive(filepath: &str) -> Result<(), String> {
    if !std::path::Path::new(filepath).exists() {
        return Err(format!("File not found: {}", filepath));
    }

    use rand::RngCore;
    if let Ok(metadata) = std::fs::metadata(filepath) {
        let size = metadata.len() as usize;
        let mut random_data = vec![0u8; size];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut random_data);

        if let Err(e) = std::fs::write(filepath, random_data) {
            return Err(format!("Error overwriting file: {}", e));
        }
    }

    match std::fs::remove_file(filepath) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Error deleting file: {}", e)),
    }
}

/// Split file (non-interactive)
pub fn split_file_noninteractive(filepath: &str, chunk_size_mb: u64) -> Result<usize, String> {
    let chunk_size = chunk_size_mb * 1024 * 1024;
    match std::fs::read(filepath) {
        Ok(data) => {
            let num_chunks = (data.len() as u64 + chunk_size - 1) / chunk_size;
            for (i, chunk) in data.chunks(chunk_size as usize).enumerate() {
                let out_path = format!("{}.{}", filepath, i);
                match std::fs::write(&out_path, chunk) {
                    Ok(_) => (),
                    Err(e) => return Err(format!("Error creating chunk: {}", e)),
                }
            }
            Ok(num_chunks as usize)
        }
        Err(e) => Err(format!("Error reading file: {}", e)),
    }
}

/// Join files (non-interactive)
pub fn join_files_noninteractive(base: &str, output: &str) -> Result<usize, String> {
    use std::fs::File;
    use std::io::Write;

    match File::create(output) {
        Ok(mut out_file) => {
            let mut part_num = 0;
            loop {
                let part_path = format!("{}.{}", base, part_num);
                match std::fs::read(&part_path) {
                    Ok(data) => {
                        if let Err(e) = out_file.write_all(&data) {
                            return Err(format!("Error writing to output file: {}", e));
                        }
                        part_num += 1;
                    }
                    Err(_) => break,
                }
            }
            Ok(part_num)
        }
        Err(e) => Err(format!("Error creating output file: {}", e)),
    }
}

// Non-interactive helper functions for GUI/backend integration
pub fn copy_file_noninteractive(src: &str, dst: &str) -> Result<(), std::io::Error> {
    fs::copy(src, dst).map(|_| ())
}

pub fn delete_file_noninteractive(path: &str) -> Result<(), std::io::Error> {
    fs::remove_file(path)
}

pub fn delete_directory_noninteractive(path: &str) -> Result<(), std::io::Error> {
    fs::remove_dir_all(path)
}

pub fn rename_file_noninteractive(old: &str, new: &str) -> Result<(), std::io::Error> {
    fs::rename(old, new)
}

pub fn move_file_noninteractive(src: &str, dst: &str) -> Result<(), std::io::Error> {
    fs::rename(src, dst)
}

pub fn create_file_noninteractive(path: &str) -> Result<(), std::io::Error> {
    match fs::File::create(path) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn create_directory_noninteractive(path: &str) -> Result<(), std::io::Error> {
    fs::create_dir(path)
}

pub fn batch_delete_noninteractive(paths: &[String]) -> Vec<Result<(), std::io::Error>> {
    paths.iter().map(|p| fs::remove_file(p)).collect()
}

/// Batch copy helper for GUI: copy each path into destination directory.
pub fn batch_copy_noninteractive(
    paths: &[String],
    dst_dir: &str,
) -> Vec<Result<(), std::io::Error>> {
    let mut results = Vec::new();
    for p in paths.iter() {
        let filename = std::path::Path::new(p)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| p.clone());
        let dest = std::path::Path::new(dst_dir).join(&filename);
        results.push(fs::copy(p, &dest).map(|_| ()));
    }
    results
}

/// Batch rename helper for GUI: accepts pairs `old -> new` as Vec<(old,new)>
pub fn batch_rename_noninteractive(pairs: &[(String, String)]) -> Vec<Result<(), std::io::Error>> {
    pairs
        .iter()
        .map(|(old, new)| fs::rename(old, new))
        .collect()
}
