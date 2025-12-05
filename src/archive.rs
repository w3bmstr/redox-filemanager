use std::process::Command;
use std::path::Path;

/// Small wrapper around the 7-Zip CLI (`7z`) to provide basic archive operations.
/// This keeps the implementation simple and calls the external `7z` binary when available.
pub fn is_7z_available() -> bool {
    Command::new("7z").arg("--help").output().is_ok()
}

pub fn list_archive(path: &str) -> Result<String, String> {
    if !Path::new(path).exists() {
        return Err(format!("Archive not found: {}", path));
    }
    let output = Command::new("7z").arg("l").arg(path).output().map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

pub fn extract_archive(path: &str, dest: &str, password: Option<&str>) -> Result<String, String> {
    if !Path::new(path).exists() {
        return Err(format!("Archive not found: {}", path));
    }
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
}

pub fn create_archive(sources: &[&str], output: &str, format: Option<&str>, password: Option<&str>) -> Result<String, String> {
    if sources.is_empty() {
        return Err("No sources provided".to_string());
    }
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
}
