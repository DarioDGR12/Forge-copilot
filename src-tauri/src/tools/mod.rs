pub mod apps;
pub mod clipboard;
pub mod fs;
pub mod notify;
pub mod processes;
pub mod screenshot;
pub mod terminal;
pub mod wm;

use crate::error::AppResult;
use serde::Serialize;
use serde_json::Value;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolOutcome {
    pub text: String,
    pub image_base64: Option<String>,
}

pub fn which(bin: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let candidate = dir.join(bin);
            candidate.is_file().then_some(candidate)
        })
    })
}

pub async fn execute(name: &str, args: &Value) -> AppResult<ToolOutcome> {
    match name {
        "run_terminal" => {
            let command = args.get("command").and_then(|v| v.as_str()).unwrap_or("");
            let cwd = args.get("cwd").and_then(|v| v.as_str());
            Ok(ToolOutcome {
                text: terminal::run_terminal(command, cwd).await?,
                image_base64: None,
            })
        }
        "read_file" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
            Ok(ToolOutcome {
                text: fs::read_file(path)?,
                image_base64: None,
            })
        }
        "write_file" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
            let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("");
            Ok(ToolOutcome {
                text: fs::write_file(path, content)?,
                image_base64: None,
            })
        }
        "list_dir" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("~");
            Ok(ToolOutcome {
                text: fs::list_dir(path)?,
                image_base64: None,
            })
        }
        "list_apps" => Ok(ToolOutcome {
            text: apps::list_apps()?,
            image_base64: None,
        }),
        "list_processes" => {
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(25) as usize;
            Ok(ToolOutcome {
                text: processes::list_processes(limit)?,
                image_base64: None,
            })
        }
        "screenshot" => {
            let shot = screenshot::capture().await?;
            Ok(ToolOutcome {
                text: format!("Captura guardada en {}", shot.path.display()),
                image_base64: shot.image_base64,
            })
        }
        "host_info" => Ok(ToolOutcome {
            text: crate::host::render(),
            image_base64: None,
        }),
        "launch_app" => {
            let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
            Ok(ToolOutcome {
                text: apps::launch_app(name)?,
                image_base64: None,
            })
        }
        "open_path" => {
            let path = args.get("path").and_then(|v| v.as_str()).unwrap_or("");
            Ok(ToolOutcome {
                text: fs::open_path(path)?,
                image_base64: None,
            })
        }
        "clipboard_read" => Ok(ToolOutcome {
            text: clipboard::read()?,
            image_base64: None,
        }),
        "clipboard_write" => {
            let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("");
            Ok(ToolOutcome {
                text: clipboard::write(text)?,
                image_base64: None,
            })
        }
        "notify" => {
            let title = args
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Forge Copilot");
            let body = args.get("body").and_then(|v| v.as_str()).unwrap_or("");
            Ok(ToolOutcome {
                text: notify::send(title, body)?,
                image_base64: None,
            })
        }
        "list_windows" => Ok(ToolOutcome {
            text: wm::list_windows()?,
            image_base64: None,
        }),
        "focus_window" => {
            let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
            Ok(ToolOutcome {
                text: wm::focus_window(query)?,
                image_base64: None,
            })
        }
        other => Err(format!("herramienta desconocida: {other}").into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn which_finds_a_shell() {
        assert!(which("sh").is_some() || which("bash").is_some());
    }
}
