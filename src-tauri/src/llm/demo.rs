use crate::error::AppResult;
use crate::llm::types::{AssistantTurn, ChatMessage, Role, ToolCall};
use std::time::Duration;

const HELP: &str = "Soy Forge Copilot. En este modo demostración simulo las herramientas; en la app Tauri de Pop!_OS las ejecuto de verdad.

Prueba:
- «qué sistema tengo»
- «lista los archivos de mi home»
- «abre Documentos»
- «abre Firefox»
- «ejecuta `uname -a`»

En Ajustes pones tu propia clave (BYOK).";

pub async fn complete(
    messages: &[ChatMessage],
    on_token: &mut (dyn FnMut(&str) + Send),
) -> AppResult<AssistantTurn> {
    let last_user = messages.iter().rposition(|m| m.role == Role::User);
    let last_tool = messages.iter().rposition(|m| m.role == Role::Tool);
    if let (Some(tool_idx), Some(user_idx)) = (last_tool, last_user) {
        if tool_idx > user_idx {
            let tool = &messages[tool_idx];
            let reply = format!(
                "Listo. Esto es lo que devolvió **{}**:\n\n```\n{}\n```",
                tool.tool_name.as_deref().unwrap_or("herramienta"),
                truncate(&tool.content, 4000)
            );
            stream_text(&reply, on_token).await;
            return Ok(AssistantTurn {
                text: reply,
                tool_calls: vec![],
            });
        }
    }

    let user = messages
        .iter()
        .rev()
        .find(|m| m.role == Role::User)
        .map(|m| m.content.clone())
        .unwrap_or_default();

    if let Some(call) = infer_tool(&user) {
        let intro = format!(
            "Voy a usar `{}` para responderte con datos del sistema.",
            call.name
        );
        stream_text(&intro, on_token).await;
        return Ok(AssistantTurn {
            text: intro,
            tool_calls: vec![call],
        });
    }

    stream_text(HELP, on_token).await;
    Ok(AssistantTurn {
        text: HELP.to_string(),
        tool_calls: vec![],
    })
}

pub fn infer_tool(text: &str) -> Option<ToolCall> {
    let t = text.to_lowercase();
    if contains_any(&t, &["screenshot", "captura", "capturar pantalla", "pantalla"])
        && contains_any(&t, &["screenshot", "captura", "capturar", "foto", "pantalla"])
    {
        if t.contains("pantalla") || t.contains("screenshot") || t.contains("captura") {
            return Some(call("screenshot", "{}"));
        }
    }
    if contains_any(&t, &["qué sistema", "que sistema", "distro", "host_info", "qué os", "que os"]) {
        return Some(call("host_info", "{}"));
    }
    if contains_any(&t, &["proceso", "process", "cpu"]) {
        return Some(call("list_processes", r#"{"limit":25}"#));
    }
    if contains_any(&t, &["abre ", "abrir ", "lanza ", "lanzar "]) {
        if let Some(path) = extract_path(text) {
            let args = serde_json::json!({ "path": path });
            return Some(call("open_path", &args.to_string()));
        }
        if contains_any(&t, &["documento", "descarga", "home", "carpeta"]) {
            let path = if t.contains("descarga") {
                "~/Downloads"
            } else if t.contains("home") {
                "~"
            } else {
                "~/Documents"
            };
            let args = serde_json::json!({ "path": path });
            return Some(call("open_path", &args.to_string()));
        }
        let name = extract_app_name(text).unwrap_or_else(|| "Firefox".into());
        let args = serde_json::json!({ "name": name });
        return Some(call("launch_app", &args.to_string()));
    }
    if contains_any(&t, &["aplicación", "aplicacion", "apps instal", "aplicaciones"]) {
        return Some(call("list_apps", "{}"));
    }
    if contains_any(&t, &["ejecuta", "correr ", "corre ", "run ", "terminal", "comando"]) {
        let command = extract_command(text)
            .unwrap_or_else(|| "uname -a && (lsb_release -d || cat /etc/os-release)".into());
        let args = serde_json::json!({ "command": command });
        return Some(call("run_terminal", &args.to_string()));
    }
    if (t.contains("lista") || t.contains("listar") || t.contains("muéstrame") || t.contains("muestrame"))
        && contains_any(&t, &["archivo", "directorio", "carpeta", "home", "home"])
    {
        return Some(call("list_dir", r#"{"path":"~"}"#));
    }
    if t.contains("lee ") || t.contains("leer ") || t.starts_with("cat ") {
        if let Some(path) = extract_path(text) {
            let args = serde_json::json!({ "path": path });
            return Some(call("read_file", &args.to_string()));
        }
    }
    None
}

fn call(name: &str, arguments: &str) -> ToolCall {
    ToolCall {
        id: format!("call_{}", uuid::Uuid::new_v4().simple()),
        name: name.into(),
        arguments: arguments.into(),
    }
}

fn extract_command(text: &str) -> Option<String> {
    if let Some(start) = text.find('`') {
        if let Some(end_rel) = text[start + 1..].find('`') {
            let cmd = text[start + 1..start + 1 + end_rel].trim();
            if !cmd.is_empty() {
                return Some(cmd.to_string());
            }
        }
    }
    for prefix in ["ejecuta ", "ejecutar ", "corre ", "correr ", "run "] {
        if let Some(idx) = text.to_lowercase().find(prefix) {
            let rest = text[idx + prefix.len()..].trim();
            if !rest.is_empty() {
                return Some(rest.trim_matches('"').to_string());
            }
        }
    }
    None
}

fn extract_app_name(text: &str) -> Option<String> {
    for prefix in ["abre ", "abrir ", "lanza ", "lanzar "] {
        if let Some(idx) = text.to_lowercase().find(prefix) {
            let rest = text[idx + prefix.len()..].trim();
            if !rest.is_empty() {
                return Some(rest.trim_matches(|c| c == '"' || c == '.' || c == '!').to_string());
            }
        }
    }
    None
}

fn extract_path(text: &str) -> Option<String> {
    text.split_whitespace()
        .find(|w| w.starts_with('/') || w.starts_with("~/") || w.starts_with('.'))
        .map(|s| s.trim_matches(|c| c == '"' || c == '\'').to_string())
}

fn contains_any(hay: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| hay.contains(n))
}

fn truncate(text: &str, max: usize) -> String {
    if text.len() <= max {
        text.to_string()
    } else {
        format!("{}…", &text[..max])
    }
}

async fn stream_text(text: &str, on_token: &mut (dyn FnMut(&str) + Send)) {
    for chunk in text.chars().collect::<Vec<_>>().chunks(4) {
        let piece: String = chunk.iter().collect();
        on_token(&piece);
        tokio::time::sleep(Duration::from_millis(8)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infers_terminal_from_backticks() {
        let call = infer_tool("ejecuta `uname -a`").unwrap();
        assert_eq!(call.name, "run_terminal");
        assert!(call.arguments.contains("uname -a"));
    }

    #[test]
    fn infers_list_dir() {
        let call = infer_tool("lista los archivos de mi home").unwrap();
        assert_eq!(call.name, "list_dir");
    }

    #[test]
    fn chat_without_tools() {
        assert!(infer_tool("hola, ¿qué puedes hacer?").is_none());
    }

    #[test]
    fn new_user_turn_is_not_old_tool_followup() {
        assert!(infer_tool("ejecuta `uname -a`").is_some());
    }

    #[test]
    fn infers_host_info() {
        assert_eq!(infer_tool("qué sistema tengo").unwrap().name, "host_info");
    }

    #[test]
    fn infers_open_documents() {
        let call = infer_tool("abre Documentos").unwrap();
        assert_eq!(call.name, "open_path");
        assert!(call.arguments.contains("Documents"));
    }

    #[test]
    fn infers_launch_firefox() {
        let call = infer_tool("abre Firefox").unwrap();
        assert_eq!(call.name, "launch_app");
        assert!(call.arguments.contains("Firefox"));
    }
}
