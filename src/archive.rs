use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::process::Command;

// Added pure-Rust archive support as a fallback when `7z` is not available.
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use tar::Archive as TarArchive;
use tar::Builder as TarBuilder;
use zip::ZipWriter;
use zip::read::ZipArchive;
use zip::write::FileOptions;

/// Small wrapper around the 7-Zip CLI (`7z`) to provide basic archive operations.
/// This keeps the implementation simple and calls the external `7z` binary when available.
pub fn is_7z_available() -> bool {
    Command::new("7z").arg("--help").output().is_ok()
}

pub fn list_archive(path: &str) -> Result<String, String> {
    if !Path::new(path).exists() {
        return Err(format!("Archive not found: {}", path));
    }
    if is_7z_available() {
        let output = Command::new("7z")
            .arg("l")
            .arg(path)
            .output()
            .map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    } else {
        // Fallback to pure-Rust listing for zip and tar formats
        let ext = Path::new(path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        if ext == "zip" {
            let file = File::open(path).map_err(|e| e.to_string())?;
            let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;
            let mut out = String::new();
            for i in 0..archive.len() {
                let f = archive.by_index(i).map_err(|e| e.to_string())?;
                out.push_str(&format!("{}\n", f.name()));
            }
            Ok(out)
        } else if ext == "tar" || ext == "gz" || path.ends_with(".tar.gz") || path.ends_with(".tgz")
        {
            let file = File::open(path).map_err(|e| e.to_string())?;
            let reader: Box<dyn Read> =
                if path.ends_with(".gz") || path.ends_with(".tgz") || path.ends_with(".tar.gz") {
                    Box::new(GzDecoder::new(file))
                } else {
                    Box::new(file)
                };
            let mut archive = TarArchive::new(reader);
            let mut out = String::new();
            for entry in archive.entries().map_err(|e| e.to_string())? {
                let e = entry.map_err(|e| e.to_string())?;
                if let Ok(path) = e.path() {
                    out.push_str(&format!("{}\n", path.display()));
                }
            }
            Ok(out)
        } else {
            Err("Unsupported archive format and 7z not available".to_string())
        }
    }
}

pub fn extract_archive(path: &str, dest: &str, password: Option<&str>) -> Result<String, String> {
    if !Path::new(path).exists() {
        return Err(format!("Archive not found: {}", path));
    }
    if is_7z_available() {
        let mut cmd = Command::new("7z");
        cmd.arg("x").arg(path).arg(format!("-o{}", dest)).arg("-y");
        if let Some(p) = password {
            if !p.is_empty() {
                cmd.arg(format!("-p{}", p));
            }
        }
        let output = cmd.output().map_err(|e| e.to_string())?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    } else {
        // Fallback: support zip and tar.gz extraction
        let ext = Path::new(path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        if ext == "zip" {
            let file = File::open(path).map_err(|e| e.to_string())?;
            let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;
            for i in 0..archive.len() {
                let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
                let outpath = Path::new(dest).join(file.mangled_name());
                if file.name().ends_with('/') {
                    std::fs::create_dir_all(&outpath).map_err(|e| e.to_string())?;
                } else {
                    if let Some(parent) = outpath.parent() {
                        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                    }
                    let mut outfile = File::create(&outpath).map_err(|e| e.to_string())?;
                    io::copy(&mut file, &mut outfile).map_err(|e| e.to_string())?;
                }
            }
            Ok("Extraction complete".to_string())
        } else if ext == "tar" || ext == "gz" || path.ends_with(".tar.gz") || path.ends_with(".tgz")
        {
            let file = File::open(path).map_err(|e| e.to_string())?;
            let reader: Box<dyn Read> =
                if path.ends_with(".gz") || path.ends_with(".tgz") || path.ends_with(".tar.gz") {
                    Box::new(GzDecoder::new(file))
                } else {
                    Box::new(file)
                };
            let mut archive = TarArchive::new(reader);
            archive.unpack(dest).map_err(|e| e.to_string())?;
            Ok("Extraction complete".to_string())
        } else {
            Err("Unsupported archive format and 7z not available".to_string())
        }
    }
}

pub fn create_archive(
    sources: &[&str],
    output: &str,
    format: Option<&str>,
    password: Option<&str>,
) -> Result<String, String> {
    if sources.is_empty() {
        return Err("No sources provided".to_string());
    }
    if is_7z_available() {
        let mut cmd = Command::new("7z");
        cmd.arg("a").arg(output);
        if let Some(t) = format {
            if !t.is_empty() {
                cmd.arg(format!("-t{}", t));
            }
        }
        if let Some(p) = password {
            if !p.is_empty() {
                cmd.arg(format!("-p{}", p));
                cmd.arg("-mhe=on"); // enable header encryption when supported
            }
        }
        for s in sources.iter() {
            cmd.arg(s);
        }
        let output_res = cmd.output().map_err(|e| e.to_string())?;
        if output_res.status.success() {
            Ok(String::from_utf8_lossy(&output_res.stdout).to_string())
        } else {
            Err(String::from_utf8_lossy(&output_res.stderr).to_string())
        }
    } else {
        // Fallback: support zip and tar.gz creation
        let out_ext = Path::new(output)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        if out_ext == "zip" {
            let file = File::create(output).map_err(|e| e.to_string())?;
            let mut zip = ZipWriter::new(file);
            let options = FileOptions::default();
            for s in sources.iter() {
                let p = Path::new(s);
                if p.is_file() {
                    let mut f = File::open(p).map_err(|e| e.to_string())?;
                    zip.start_file(
                        p.file_name().and_then(|n| n.to_str()).unwrap_or("file"),
                        options,
                    )
                    .map_err(|e| e.to_string())?;
                    io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
                } else if p.is_dir() {
                    // recursively add files
                    for entry in walkdir::WalkDir::new(p) {
                        let entry = entry.map_err(|e| e.to_string())?;
                        let path = entry.path();
                        if path.is_file() {
                            let name = path
                                .strip_prefix(p)
                                .map_err(|e| e.to_string())?
                                .to_string_lossy()
                                .to_string();
                            let mut f = File::open(path).map_err(|e| e.to_string())?;
                            zip.start_file(name, options).map_err(|e| e.to_string())?;
                            io::copy(&mut f, &mut zip).map_err(|e| e.to_string())?;
                        }
                    }
                }
            }
            zip.finish().map_err(|e| e.to_string())?;
            Ok("Archive created".to_string())
        } else if out_ext == "gz"
            || output.ends_with(".tar.gz")
            || output.ends_with(".tgz")
            || out_ext == "tar"
        {
            let tar_gz = File::create(output).map_err(|e| e.to_string())?;
            let enc = GzEncoder::new(tar_gz, Compression::default());
            let mut tar = TarBuilder::new(enc);
            for s in sources.iter() {
                tar.append_path(s).map_err(|e| e.to_string())?;
            }
            // finish
            let _ = tar
                .into_inner()
                .map_err(|e| e.to_string())?
                .finish()
                .map_err(|e| e.to_string())?;
            Ok("Archive created".to_string())
        } else {
            Err("Unsupported output format and 7z not available".to_string())
        }
    }
}
