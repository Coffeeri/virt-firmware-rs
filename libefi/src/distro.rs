extern crate std;
use std::collections::HashMap;
use std::format;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::string::{String, ToString};

use log::{debug, error};

pub fn envfile_parse_string(data: &str) -> HashMap<String, String> {
    let mut values = HashMap::new();
    for line in data.split('\n') {
        let line = line.trim();
        if line.is_empty() || line.starts_with("#") || line.starts_with(";") {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"');
            values.insert(key.to_string(), value.to_string());
        } else {
            debug!("parse error: no '=' in line ({})", line);
        }
    }
    values
}

pub fn envfile_parse_file(filename: &str) -> Option<HashMap<String, String>> {
    match fs::read_to_string(filename) {
        Err(e) => {
            error!("open {}: {}", filename, e);
            None
        }
        Ok(s) => {
            debug!("reading {}", filename);
            Some(envfile_parse_string(&s))
        }
    }
}

pub fn efi_arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "x64",
        "aarch64" => "aa64",
        a => a,
    }
}

pub fn find_esp_path() -> Option<String> {
    // try use bootctl first
    if let Ok(output) = Command::new("bootctl").arg("--print-esp-path").output() {
        let s = String::from_utf8_lossy(&output.stdout);
        return Some(s.trim().to_string());
    }

    // fallback
    for candidate in ["/efi", "/boot/efi", "/boot"] {
        let bootdir = format!("{}/EFI/BOOT", candidate);
        if Path::new(&bootdir).exists() {
            return Some(candidate.to_string());
        }
    }

    None
}

pub struct LinuxOsInfo {
    values: HashMap<String, String>,
    esp_path: Option<String>,
}

impl LinuxOsInfo {
    pub fn new() -> LinuxOsInfo {
        let values =
            envfile_parse_file("/etc/os-release").expect("can not read /etc/os-release file");
        let esp_path = find_esp_path();
        LinuxOsInfo { values, esp_path }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        match self.values.get(key) {
            Some(v) => Some(v.as_str()),
            None => None,
        }
    }

    pub fn esp_distro_dir(&self) -> &str {
        match self.get("ID") {
            Some("rhel") => "redhat",
            Some(x) => x,
            None => "unknown",
        }
    }

    pub fn shim_path(&self) -> Option<String> {
        let Some(esp) = &self.esp_path else {
            return None;
        };
        let path = format!(
            "{}/EFI/{}/shim{}.efi",
            esp,
            self.esp_distro_dir(),
            efi_arch()
        );
        Some(path)
    }
}

impl Default for LinuxOsInfo {
    fn default() -> Self {
        Self::new()
    }
}
