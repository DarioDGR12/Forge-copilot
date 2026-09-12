use crate::error::AppResult;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct Win {
    pub id: String,
    pub title: String,
}

pub fn list_windows() -> AppResult<String> {
    let wins = collect()?;
    if wins.is_empty() {
        return Ok("no hay ventanas visibles (¿falta wmctrl en Wayland?)".into());
    }
    let mut lines = vec![format!("{} ventanas", wins.len())];
    for win in wins {
        lines.push(format!("{}  {}", win.id, win.title));
    }
    Ok(lines.join("\n"))
}

pub fn focus_window(query: &str) -> AppResult<String> {
    let query = query.trim();
    if query.is_empty() {
        return Err("indica el título o el id de la ventana".into());
    }
    if crate::tools::which("wmctrl").is_none() {
        return Err("wmctrl no está instalado; en COSMIC/Wayland el listado de ventanas es limitado".into());
    }
    let wins = collect()?;
    let found = wins.iter().find(|w| {
        w.id.eq_ignore_ascii_case(query) || w.title.to_lowercase().contains(&query.to_lowercase())
    });
    let Some(win) = found else {
        return Err(format!("no encontré una ventana que coincida con «{query}»").into());
    };
    let status = if win.id.starts_with("0x") {
        Command::new("wmctrl").args(["-i", "-a", &win.id]).status()?
    } else {
        Command::new("wmctrl").args(["-a", &win.title]).status()?
    };
    if !status.success() {
        return Err("wmctrl no pudo enfocar la ventana".into());
    }
    Ok(format!("enfocada: {}", win.title))
}

pub fn parse_wmctrl(raw: &str) -> Vec<Win> {
    raw.lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let id = parts.next()?.to_string();
            let _desktop = parts.next()?;
            let _host = parts.next()?;
            let title = parts.collect::<Vec<_>>().join(" ");
            if title.is_empty() {
                return None;
            }
            Some(Win { id, title })
        })
        .collect()
}

fn collect() -> AppResult<Vec<Win>> {
    if crate::tools::which("wmctrl").is_none() {
        return Err(
            "wmctrl no está instalado. En X11: sudo apt install wmctrl. En Wayland/COSMIC el compositor no siempre lo expone."
                .into(),
        );
    }
    let out = Command::new("wmctrl").arg("-l").output()?;
    if !out.status.success() {
        return Err(format!(
            "wmctrl: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )
        .into());
    }
    Ok(parse_wmctrl(&String::from_utf8_lossy(&out.stdout)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_wmctrl_line() {
        let raw = "0x03c00007  0 pop-os Firefox\n0x02a00001  0 pop-os Forge Copilot\n";
        let wins = parse_wmctrl(raw);
        assert_eq!(wins.len(), 2);
        assert_eq!(wins[0].title, "Firefox");
        assert_eq!(wins[1].id, "0x02a00001");
    }
}
