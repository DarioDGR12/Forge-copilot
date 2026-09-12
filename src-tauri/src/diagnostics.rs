use crate::tools;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticItem {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostics {
    pub host: String,
    pub items: Vec<DiagnosticItem>,
}

pub fn collect() -> Diagnostics {
    Diagnostics {
        host: crate::host::render(),
        items: vec![
            any_of(
                "Captura",
                &["grim", "gnome-screenshot", "spectacle", "import"],
                "sudo apt install grim",
            ),
            any_of(
                "Portapapeles",
                &["wl-paste", "xclip", "xsel"],
                "sudo apt install wl-clipboard o xclip",
            ),
            one("Notificaciones", "notify-send", "sudo apt install libnotify-bin"),
            one("Ventanas", "wmctrl", "sudo apt install wmctrl (limitado en Wayland)"),
            any_of(
                "Teclado / ratón",
                &["ydotool", "xdotool", "wtype"],
                "X11: xdotool · Wayland: ydotool + ydotoold",
            ),
            one("Abrir rutas", "xdg-open", "paquete xdg-utils"),
            one("Lanzar apps", "gtk-launch", "opcional; si falta se usa Exec del .desktop"),
            one("Shell", "bash", "necesario para run_terminal"),
        ],
    }
}

fn one(name: &str, bin: &str, hint: &str) -> DiagnosticItem {
    match tools::which(bin) {
        Some(path) => ok(name, path),
        None => miss(name, hint),
    }
}

fn any_of(name: &str, bins: &[&str], hint: &str) -> DiagnosticItem {
    for bin in bins {
        if let Some(path) = tools::which(bin) {
            return ok(name, path);
        }
    }
    miss(name, hint)
}

fn ok(name: &str, path: PathBuf) -> DiagnosticItem {
    DiagnosticItem {
        name: name.into(),
        ok: true,
        detail: path.display().to_string(),
    }
}

fn miss(name: &str, hint: &str) -> DiagnosticItem {
    DiagnosticItem {
        name: name.into(),
        ok: false,
        detail: hint.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_shell() {
        let diag = collect();
        let shell = diag.items.iter().find(|i| i.name == "Shell").unwrap();
        assert!(shell.ok);
        assert!(diag.host.contains("os="));
    }

    #[test]
    fn lists_expected_checks() {
        let diag = collect();
        let names: Vec<_> = diag.items.iter().map(|i| i.name.as_str()).collect();
        assert!(names.contains(&"Captura"));
        assert!(names.contains(&"Teclado / ratón"));
        assert!(names.contains(&"Portapapeles"));
    }
}
