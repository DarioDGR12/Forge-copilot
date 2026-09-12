use serde_json::Value;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Risk {
    AutoAllow,
    NeedsApproval { reason: String, sensitive: bool },
}

const SENSITIVE_MARKERS: &[&str] = &[
    "/.ssh",
    "/.gnupg",
    "/.aws",
    "/.password-store",
    "/.local/share/keyrings",
    "/.config/forge-copilot/secrets",
    "/etc/shadow",
    "/etc/sudoers",
    "id_rsa",
    "id_ed25519",
];

const DANGEROUS_CMD: &[&str] = &[
    "rm -rf",
    "mkfs",
    "dd if=",
    "sudo ",
    ":(){",
    "chmod 777",
    "curl | sh",
    "wget | sh",
    "curl|sh",
    "wget|sh",
];

pub fn assess(tool: &str, args: &Value, allowed_roots: &[String]) -> Risk {
    match tool {
        "list_apps"
        | "list_processes"
        | "screenshot"
        | "host_info"
        |         "clipboard_read"
        | "list_windows"
        | "notify"
        | "pointer_info" => Risk::AutoAllow,
        "list_dir" | "read_file" => assess_read(tool, args_path(args), allowed_roots),
        "open_path" => assess_open(args_path(args), allowed_roots),
        "launch_app" => {
            let name = args
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim();
            Risk::NeedsApproval {
                reason: format!("Lanzar aplicación: {name}"),
                sensitive: false,
            }
        }
        "clipboard_write" => {
            let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("");
            Risk::NeedsApproval {
                reason: format!(
                    "Escribir al portapapeles ({} caracteres)",
                    text.chars().count()
                ),
                sensitive: false,
            }
        }
        "focus_window" => {
            let query = args
                .get("query")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim();
            Risk::NeedsApproval {
                reason: format!("Enfocar ventana: {query}"),
                sensitive: false,
            }
        }
        "type_text" => {
            let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("");
            let preview: String = text.chars().take(80).collect();
            Risk::NeedsApproval {
                reason: format!(
                    "Teclear en la ventana activa (Forge se ocultará): «{preview}»"
                ),
                sensitive: true,
            }
        }
        "press_keys" => {
            let keys = args.get("keys").and_then(|v| v.as_str()).unwrap_or("");
            Risk::NeedsApproval {
                reason: format!("Pulsar teclas en la ventana activa: {keys}"),
                sensitive: true,
            }
        }
        "mouse_click" => {
            let x = args.get("x").and_then(|v| v.as_i64());
            let y = args.get("y").and_then(|v| v.as_i64());
            let button = args.get("button").and_then(|v| v.as_str()).unwrap_or("left");
            let where_ = match (x, y) {
                (Some(x), Some(y)) => format!("{x},{y}"),
                _ => "posición actual".into(),
            };
            Risk::NeedsApproval {
                reason: format!("Clic {button} en {where_} (Forge se ocultará)"),
                sensitive: true,
            }
        }
        "write_file" => assess_write(args_path(args), allowed_roots),
        "run_terminal" => {
            let command = args
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim();
            if command.is_empty() {
                return Risk::NeedsApproval {
                    reason: "Comando vacío".into(),
                    sensitive: true,
                };
            }
            Risk::NeedsApproval {
                reason: format!("Comando: {command}"),
                sensitive: command_looks_sensitive(command),
            }
        }
        other => Risk::NeedsApproval {
            reason: format!("Herramienta desconocida: {other}"),
            sensitive: true,
        },
    }
}

pub fn expand_path(raw: &str) -> PathBuf {
    let raw = raw.trim();
    if raw.is_empty() {
        return dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    }
    if raw == "~" {
        return dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
    }
    if let Some(rest) = raw.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(raw)
}

pub fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                out.pop();
            }
            Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

pub fn is_sensitive(path: &Path) -> bool {
    let rendered = path.to_string_lossy();
    SENSITIVE_MARKERS.iter().any(|marker| rendered.contains(marker))
}

pub fn in_allowed_roots(path: &Path, roots: &[String]) -> bool {
    if roots.is_empty() {
        return false;
    }
    let canon = normalize(path);
    roots.iter().any(|root| {
        let root = normalize(&expand_path(root));
        canon.starts_with(&root)
    })
}

pub fn command_looks_sensitive(command: &str) -> bool {
    let lower = command.to_lowercase();
    DANGEROUS_CMD.iter().any(|needle| lower.contains(needle))
}

fn args_path(args: &Value) -> PathBuf {
    let raw = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
    expand_path(raw)
}

fn assess_read(tool: &str, path: PathBuf, roots: &[String]) -> Risk {
    if path.as_os_str().is_empty() {
        return Risk::NeedsApproval {
            reason: "Ruta vacía".into(),
            sensitive: true,
        };
    }
    if is_sensitive(&path) {
        return Risk::NeedsApproval {
            reason: format!("Ruta sensible: {}", path.display()),
            sensitive: true,
        };
    }
    if in_allowed_roots(&path, roots) {
        return Risk::AutoAllow;
    }
    Risk::NeedsApproval {
        reason: format!("{tool} fuera de las carpetas permitidas: {}", path.display()),
        sensitive: false,
    }
}

fn assess_open(path: PathBuf, roots: &[String]) -> Risk {
    if is_sensitive(&path) {
        return Risk::NeedsApproval {
            reason: format!("Abrir ruta sensible: {}", path.display()),
            sensitive: true,
        };
    }
    if in_allowed_roots(&path, roots) {
        return Risk::AutoAllow;
    }
    Risk::NeedsApproval {
        reason: format!("Abrir fuera de las carpetas permitidas: {}", path.display()),
        sensitive: false,
    }
}

fn assess_write(path: PathBuf, roots: &[String]) -> Risk {
    let sensitive = is_sensitive(&path) || !in_allowed_roots(&path, roots);
    Risk::NeedsApproval {
        reason: format!("Escribir: {}", path.display()),
        sensitive,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn home_roots() -> Vec<String> {
        vec!["/home/demo".into()]
    }

    #[test]
    fn lists_and_screenshot_are_auto() {
        assert_eq!(assess("host_info", &json!({}), &home_roots()), Risk::AutoAllow);
        assert_eq!(
            assess("list_apps", &json!({}), &home_roots()),
            Risk::AutoAllow
        );
        assert_eq!(
            assess("screenshot", &json!({}), &home_roots()),
            Risk::AutoAllow
        );
        assert_eq!(
            assess("list_processes", &json!({}), &home_roots()),
            Risk::AutoAllow
        );
    }

    #[test]
    fn read_inside_home_is_auto() {
        let risk = assess(
            "read_file",
            &json!({"path": "/home/demo/notes.txt"}),
            &home_roots(),
        );
        assert_eq!(risk, Risk::AutoAllow);
    }

    #[test]
    fn read_outside_home_needs_approval() {
        match assess(
            "read_file",
            &json!({"path": "/etc/os-release"}),
            &home_roots(),
        ) {
            Risk::NeedsApproval { sensitive, .. } => assert!(!sensitive),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn ssh_key_is_sensitive() {
        match assess(
            "read_file",
            &json!({"path": "/home/demo/.ssh/id_ed25519"}),
            &home_roots(),
        ) {
            Risk::NeedsApproval { sensitive, .. } => assert!(sensitive),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn write_always_needs_approval() {
        match assess(
            "write_file",
            &json!({"path": "/home/demo/out.txt", "content": "hi"}),
            &home_roots(),
        ) {
            Risk::NeedsApproval { sensitive, .. } => assert!(!sensitive),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn shell_always_needs_approval() {
        match assess(
            "run_terminal",
            &json!({"command": "ls -la"}),
            &home_roots(),
        ) {
            Risk::NeedsApproval { sensitive, reason } => {
                assert!(!sensitive);
                assert!(reason.contains("ls -la"));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn destructive_shell_is_sensitive() {
        match assess(
            "run_terminal",
            &json!({"command": "sudo rm -rf /tmp/demo"}),
            &home_roots(),
        ) {
            Risk::NeedsApproval { sensitive, .. } => assert!(sensitive),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn parent_dir_escape_is_detected() {
        let escaped = normalize(Path::new("/home/demo/../etc/shadow"));
        assert!(is_sensitive(&escaped));
        assert!(!in_allowed_roots(Path::new("/home/demo/../etc"), &home_roots()));
    }

    #[test]
    fn launch_app_needs_approval() {
        match assess("launch_app", &json!({"name": "Firefox"}), &home_roots()) {
            Risk::NeedsApproval { sensitive, reason } => {
                assert!(!sensitive);
                assert!(reason.contains("Firefox"));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn open_home_file_is_auto() {
        assert_eq!(
            assess(
                "open_path",
                &json!({"path": "/home/demo/Documents"}),
                &home_roots()
            ),
            Risk::AutoAllow
        );
    }

    #[test]
    fn tilde_expands_to_home() {
        let expanded = expand_path("~/Documents");
        assert!(expanded.ends_with("Documents"));
    }

    #[test]
    fn desktop_read_tools_are_auto() {
        assert_eq!(
            assess("clipboard_read", &json!({}), &home_roots()),
            Risk::AutoAllow
        );
        assert_eq!(
            assess("list_windows", &json!({}), &home_roots()),
            Risk::AutoAllow
        );
        assert_eq!(
            assess(
                "notify",
                &json!({"title": "Forge", "body": "hola"}),
                &home_roots()
            ),
            Risk::AutoAllow
        );
    }

    #[test]
    fn clipboard_write_needs_approval() {
        match assess(
            "clipboard_write",
            &json!({"text": "hola"}),
            &home_roots(),
        ) {
            Risk::NeedsApproval { sensitive, reason } => {
                assert!(!sensitive);
                assert!(reason.contains("portapapeles"));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn focus_window_needs_approval() {
        match assess(
            "focus_window",
            &json!({"query": "Firefox"}),
            &home_roots(),
        ) {
            Risk::NeedsApproval { sensitive, reason } => {
                assert!(!sensitive);
                assert!(reason.contains("Firefox"));
            }
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn pointer_info_is_auto() {
        assert_eq!(
            assess("pointer_info", &json!({}), &home_roots()),
            Risk::AutoAllow
        );
    }

    #[test]
    fn input_tools_are_sensitive() {
        match assess("type_text", &json!({"text": "hola"}), &home_roots()) {
            Risk::NeedsApproval { sensitive, reason } => {
                assert!(sensitive);
                assert!(reason.contains("hola"));
            }
            other => panic!("unexpected {other:?}"),
        }
        match assess("press_keys", &json!({"keys": "ctrl+c"}), &home_roots()) {
            Risk::NeedsApproval { sensitive, .. } => assert!(sensitive),
            other => panic!("unexpected {other:?}"),
        }
        match assess(
            "mouse_click",
            &json!({"x": 10, "y": 20, "button": "left"}),
            &home_roots(),
        ) {
            Risk::NeedsApproval { sensitive, reason } => {
                assert!(sensitive);
                assert!(reason.contains("10,20"));
            }
            other => panic!("unexpected {other:?}"),
        }
    }
}
