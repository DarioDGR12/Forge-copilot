use crate::error::AppResult;
use crate::policy::expand_path;
use std::time::Duration;
use tokio::process::Command;

const MAX_OUTPUT: usize = 32 * 1024;
const TIMEOUT_SECS: u64 = 30;

pub async fn run_terminal(command: &str, cwd: Option<&str>) -> AppResult<String> {
    let command = command.trim();
    if command.is_empty() {
        return Err("comando vacío".into());
    }

    let mut child = Command::new("bash");
    child.arg("-lc").arg(command).kill_on_drop(true);
    child.env("TERM", "xterm-256color");
    if let Some(cwd) = cwd.map(str::trim).filter(|s| !s.is_empty()) {
        child.current_dir(expand_path(cwd));
    }

    let output = tokio::time::timeout(Duration::from_secs(TIMEOUT_SECS), child.output())
        .await
        .map_err(|_| AppErrorTimeout)?;

    let output = output?;
    let stdout = truncate(&String::from_utf8_lossy(&output.stdout), MAX_OUTPUT);
    let stderr = truncate(&String::from_utf8_lossy(&output.stderr), MAX_OUTPUT);
    let code = output.status.code().unwrap_or(-1);
    Ok(format!(
        "exit={code}\n--- stdout ---\n{stdout}\n--- stderr ---\n{stderr}"
    ))
}

struct AppErrorTimeout;

impl From<AppErrorTimeout> for crate::error::AppError {
    fn from(_: AppErrorTimeout) -> Self {
        crate::error::AppError::Msg(format!(
            "el comando superó el tiempo límite de {TIMEOUT_SECS}s"
        ))
    }
}

fn truncate(text: &str, max: usize) -> String {
    if text.len() <= max {
        text.to_string()
    } else {
        format!("{}\n…[truncado]", &text[..max])
    }
}
