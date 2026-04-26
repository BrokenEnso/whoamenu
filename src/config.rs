use std::env;
use std::fs;
use std::path::PathBuf;

pub fn read_config_args() -> Vec<String> {
    let path = match config_path() {
        Some(path) => path,
        None => return Vec::new(),
    };

    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };

    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .flat_map(tokenize_config_line)
        .collect()
}

pub fn config_path() -> Option<PathBuf> {
    if let Ok(xdg_config_home) = env::var("XDG_CONFIG_HOME") {
        if !xdg_config_home.trim().is_empty() {
            return Some(
                PathBuf::from(xdg_config_home)
                    .join("whoamenu")
                    .join("config"),
            );
        }
    }

    let home = env::var("HOME").ok()?;
    Some(
        PathBuf::from(home)
            .join(".config")
            .join("whoamenu")
            .join("config"),
    )
}

pub fn tokenize_config_line(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in line.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            c if c.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            c => current.push(c),
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}
