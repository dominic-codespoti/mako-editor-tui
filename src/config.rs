use std::{fs, io, path::PathBuf, process::Command};

use home::home_dir;

/// Representation of one config line (key = value).
#[derive(Clone, Debug)]
pub struct Param {
    pub key: String,
    pub value: String,
}

impl Param {
    pub fn new<K: Into<String>, V: Into<String>>(key: K, value: V) -> Self {
        Param {
            key: key.into(),
            value: value.into(),
        }
    }

    pub fn formatted_value(&self) -> String {
        self.value.trim().to_string()
    }
}

/// Representation of the whole config file.
#[derive(Clone, Debug)]
pub struct Config {
    pub params: Vec<Param>,
}

impl Config {
    pub fn config_path() -> PathBuf {
        let mut p = home_dir().expect("Could not find home directory");
        p.push(".config/mako");
        p.push("config");
        p
    }

    pub fn load() -> io::Result<Self> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Config { params: Vec::new() });
        }
        let s = fs::read_to_string(&path)?;
        let mut params = Vec::new();
        for line in s.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(idx) = line.find('=') {
                let key = line[..idx].trim().to_string();
                let mut value = line[idx + 1..].trim().to_string();
                if (value.starts_with('"') && value.ends_with('"')) || (value.starts_with('\'') && value.ends_with('\'')) {
                    value = value[1..value.len() - 1].to_string();
                    value = value.replace("\\\"", "\"");
                }
                params.push(Param::new(key, value));
            } else {
                // line with no '=' — we'll ignore for now
            }
        }
        Ok(Config { params })
    }

    pub fn save(&self) -> io::Result<PathBuf> {
        let mut path = home_dir().expect("Could not find home directory");
        path.push(".config/mako");
        fs::create_dir_all(&path)?;
        path.push("config");

        let mut contents = String::new();
        for p in &self.params {
            contents.push_str(&format!("{}={}\n", p.key, p.formatted_value()));
        }

        fs::write(&path, contents)?;
        Ok(path)
    }

    pub fn add_param(&mut self, key: String, value: String) {
        self.params.push(Param::new(key, value));
    }

    pub fn remove_param(&mut self, idx: usize) {
        if idx < self.params.len() {
            self.params.remove(idx);
        }
    }

    pub fn reload(&self) -> Result<String, String> {
        match Command::new("makoctl").arg("reload").output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
                } else {
                    Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
                }
            }
            Err(e) => Err(format!("Failed to execute makoctl: {}", e)),
        }
    }

    pub fn notify(&self, key: &str, value: &str) {
        let _ = Command::new("notify-send")
            .arg("Mako Config Updated")
            .arg(format!("{} = {}", key, value))
            .spawn();
    }
}
