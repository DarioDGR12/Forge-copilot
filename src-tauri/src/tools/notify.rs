use crate::error::AppResult;
use std::process::Command;

pub fn send(title: &str, body: &str) -> AppResult<String> {
    let title = title.trim();
    let body = body.trim();
    if title.is_empty() {
        return Err("título vacío".into());
    }
    if crate::tools::which("notify-send").is_none() {
        return Err("notify-send no está instalado (paquete libnotify-bin)".into());
    }
    let status = Command::new("notify-send")
        .args(["--app-name=Forge Copilot", title, body])
        .status()?;
    if !status.success() {
        return Err("notify-send falló".into());
    }
    Ok(format!("notificación: {title}"))
}
