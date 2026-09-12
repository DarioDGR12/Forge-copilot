use crate::error::AppResult;
use std::path::PathBuf;
use tokio::process::Command;

#[derive(Debug, Clone)]
pub struct ScreenshotResult {
    pub path: PathBuf,
    pub image_base64: Option<String>,
}

pub async fn capture() -> AppResult<ScreenshotResult> {
    let path = std::env::temp_dir().join(format!("forge-copilot-{}.png", uuid::Uuid::new_v4()));
    let path_str = path.to_string_lossy().to_string();

    let attempts: &[(&str, Vec<String>)] = &[
        ("grim", vec!["-t".into(), "png".into(), path_str.clone()]),
        ("gnome-screenshot", vec!["-f".into(), path_str.clone()]),
        (
            "spectacle",
            vec!["-b".into(), "-n".into(), "-o".into(), path_str.clone()],
        ),
        (
            "import",
            vec!["-window".into(), "root".into(), path_str.clone()],
        ),
    ];

    let mut last_err = String::from("ninguna herramienta de captura está disponible");
    for (bin, args) in attempts {
        if which(bin).is_none() {
            continue;
        }
        match Command::new(bin).args(args).output().await {
            Ok(out) if out.status.success() && path.exists() => {
                return Ok(pack(path));
            }
            Ok(out) => {
                last_err = format!(
                    "{bin}: {}",
                    String::from_utf8_lossy(&out.stderr).trim()
                );
            }
            Err(e) => last_err = format!("{bin}: {e}"),
        }
    }

    Err(format!(
        "{last_err}. En Pop!_OS / GNOME / COSMIC instala grim o usa el portal de captura (xdg-desktop-portal)."
    )
    .into())
}

fn pack(path: PathBuf) -> ScreenshotResult {
    let image_base64 = std::fs::read(&path).ok().and_then(|bytes| {
        if bytes.len() < 2_000_000 {
            Some(base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                bytes,
            ))
        } else {
            None
        }
    });
    ScreenshotResult { path, image_base64 }
}

fn which(bin: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            let candidate = dir.join(bin);
            candidate.is_file().then_some(candidate)
        })
    })
}
