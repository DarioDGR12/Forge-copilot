use std::fs;

#[derive(Debug, Clone)]
pub struct HostInfo {
    pub user: String,
    pub hostname: String,
    pub os: String,
    pub desktop: String,
    pub home: String,
}

pub fn collect() -> HostInfo {
    HostInfo {
        user: std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "desconocido".into()),
        hostname: hostname(),
        os: pretty_os(),
        desktop: std::env::var("XDG_CURRENT_DESKTOP")
            .or_else(|_| std::env::var("DESKTOP_SESSION"))
            .unwrap_or_else(|_| "linux".into()),
        home: dirs::home_dir()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| "/".into()),
    }
}

pub fn render() -> String {
    let info = collect();
    format!(
        "usuario={}\nhostname={}\nos={}\ndesktop={}\nhome={}",
        info.user, info.hostname, info.os, info.desktop, info.home
    )
}

pub fn system_prompt() -> String {
    format!(
        "{}\n\nContexto de esta máquina:\n{}",
        crate::llm::types::SYSTEM_PROMPT,
        render()
    )
}

fn hostname() -> String {
    if let Ok(name) = fs::read_to_string("/etc/hostname") {
        let name = name.trim();
        if !name.is_empty() {
            return name.to_string();
        }
    }
    std::env::var("HOSTNAME").unwrap_or_else(|_| "localhost".into())
}

fn pretty_os() -> String {
    if let Ok(raw) = fs::read_to_string("/etc/os-release") {
        for line in raw.lines() {
            if let Some(value) = line.strip_prefix("PRETTY_NAME=") {
                return value.trim_matches('"').to_string();
            }
        }
    }
    "Linux".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_fields() {
        let text = render();
        assert!(text.contains("usuario="));
        assert!(text.contains("os="));
        assert!(text.contains("home="));
    }
}
