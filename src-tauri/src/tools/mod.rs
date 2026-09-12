pub mod apps;
pub mod fs;
pub mod processes;
pub mod screenshot;
pub mod terminal;

use crate::error::AppResult;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolOutcome {
    pub text: String,
    pub image_base64: Option<String>,
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
        other => Err(format!("herramienta desconocida: {other}").into()),
    }
}
