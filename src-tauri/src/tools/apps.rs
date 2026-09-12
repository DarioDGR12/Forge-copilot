use crate::error::AppResult;
use serde::Serialize;
use std::fs;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DesktopApp {
    pub name: String,
    pub exec: String,
    pub desktop_file: String,
}

pub fn collect_apps() -> Vec<DesktopApp> {
    let mut dirs = vec![
        std::path::PathBuf::from("/usr/share/applications"),
        std::path::PathBuf::from("/usr/local/share/applications"),
    ];
    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".local/share/applications"));
    }

    let mut apps = Vec::new();
    for dir in dirs {
        if !dir.is_dir() {
            continue;
        }
        let Ok(read) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in read.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("desktop") {
                continue;
            }
            if let Ok(contents) = fs::read_to_string(&path) {
                if let Some(mut app) = parse_desktop(&contents) {
                    app.desktop_file = path.to_string_lossy().into_owned();
                    apps.push(app);
                }
            }
        }
    }

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps.dedup_by(|a, b| a.name.eq_ignore_ascii_case(&b.name));
    apps
}

pub fn list_apps() -> AppResult<String> {
    let apps = collect_apps();
    let mut lines = vec![format!("{} aplicaciones", apps.len())];
    for app in apps.into_iter().take(250) {
        lines.push(format!("{} — {}", app.name, app.exec));
    }
    Ok(lines.join("\n"))
}

pub fn clean_exec(exec: &str) -> Option<String> {
    if exec.contains('\n') || exec.contains("..") {
        return None;
    }
    let bin = exec
        .split_whitespace()
        .find(|part| !part.starts_with('%'))
        .map(|part| part.to_string())?;
    if bin.contains('/') && !std::path::Path::new(&bin).exists() {
        return None;
    }
    Some(bin)
}

pub fn launch_app(name: &str) -> AppResult<String> {
    let needle = name.trim();
    if needle.is_empty() {
        return Err("nombre de aplicación vacío".into());
    }
    let apps = collect_apps();
    let app = apps
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case(needle))
        .or_else(|| {
            apps.iter().find(|a| a.name.to_lowercase().contains(&needle.to_lowercase()))
        })
        .ok_or_else(|| format!("no encontré una app .desktop llamada «{needle}»"))?;

    let stem = std::path::Path::new(&app.desktop_file)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    if !stem.is_empty() && which("gtk-launch").is_some() {
        match std::process::Command::new("gtk-launch").arg(stem).spawn() {
            Ok(_) => return Ok(format!("lanzada {} ({stem})", app.name)),
            Err(err) => {
                eprintln!("gtk-launch: {err}");
            }
        }
    }

    let bin = clean_exec(&app.exec)
        .ok_or_else(|| format!("Exec no seguro en {}", app.desktop_file))?;
    std::process::Command::new(bin)
        .spawn()
        .map_err(|e| format!("no se pudo lanzar {}: {e}", app.name))?;
    Ok(format!("lanzada {}", app.name))
}

fn which(bin: &str) -> Option<std::path::PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let candidate = dir.join(bin);
            candidate.is_file().then_some(candidate)
        })
    })
}

pub fn parse_desktop(contents: &str) -> Option<DesktopApp> {
    let mut in_entry = false;
    let mut name = None;
    let mut exec = None;
    let mut hidden = false;
    let mut no_display = false;
    let mut typ: Option<String> = None;

    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            in_entry = line.eq_ignore_ascii_case("[Desktop Entry]");
            continue;
        }
        if !in_entry {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key {
            "Name" => name = Some(value.to_string()),
            "Exec" => exec = Some(value.to_string()),
            "Hidden" if value.eq_ignore_ascii_case("true") => hidden = true,
            "NoDisplay" if value.eq_ignore_ascii_case("true") => no_display = true,
            "Type" => typ = Some(value.to_string()),
            _ => {}
        }
    }

    if hidden || no_display {
        return None;
    }
    if typ.as_deref().is_some_and(|t| t != "Application") {
        return None;
    }
    Some(DesktopApp {
        name: name?,
        exec: exec?,
        desktop_file: String::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_firefox() {
        let raw = r#"
[Desktop Entry]
Name=Firefox
Exec=firefox %u
Type=Application
"#;
        let app = parse_desktop(raw).unwrap();
        assert_eq!(app.name, "Firefox");
        assert_eq!(app.exec, "firefox %u");
    }

    #[test]
    fn hides_nodisplay() {
        let raw = "[Desktop Entry]\nName=Hidden\nExec=x\nNoDisplay=true\n";
        assert!(parse_desktop(raw).is_none());
    }

    #[test]
    fn skips_non_application() {
        let raw = "[Desktop Entry]\nName=Link\nExec=x\nType=Link\n";
        assert!(parse_desktop(raw).is_none());
    }

    #[test]
    fn strips_field_codes() {
        assert_eq!(clean_exec("firefox %u").as_deref(), Some("firefox"));
        assert_eq!(clean_exec("bad\nbin"), None);
    }
}
