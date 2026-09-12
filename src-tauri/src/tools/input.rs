use crate::error::AppResult;
use std::io::Write;
use std::process::{Command, Stdio};

const MAX_TYPE: usize = 2000;
const MISSING: &str = "No hay herramienta de entrada. En X11: sudo apt install xdotool. \
En Wayland/COSMIC: ydotool + el daemon ydotoold (usuario en el grupo input). \
wtype solo teclea en algunos compositors. No simulo clics ni teclas.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Combo {
    pub parts: Vec<String>,
}

pub fn type_text(text: &str) -> AppResult<String> {
    if text.is_empty() {
        return Err("texto vacío".into());
    }
    if text.chars().count() > MAX_TYPE {
        return Err(format!("el texto supera {MAX_TYPE} caracteres").into());
    }
    let mut last = String::from(MISSING);
    if crate::tools::which("ydotool").is_some() {
        match ydotool_type(text) {
            Ok(()) => return Ok(format!("tecleados {} caracteres", text.chars().count())),
            Err(e) => last = e,
        }
    }
    if crate::tools::which("xdotool").is_some() {
        match run_ok("xdotool", &["type", "--clearmodifiers", "--", text]) {
            Ok(()) => return Ok(format!("tecleados {} caracteres", text.chars().count())),
            Err(e) => last = e,
        }
    }
    if crate::tools::which("wtype").is_some() {
        match run_ok("wtype", &["--", text]) {
            Ok(()) => return Ok(format!("tecleados {} caracteres", text.chars().count())),
            Err(e) => last = e,
        }
    }
    Err(last.into())
}

pub fn press_keys(raw: &str) -> AppResult<String> {
    let combo = parse_combo(raw)?;
    let rendered = combo.xdotool();
    let mut last = String::from(MISSING);
    if crate::tools::which("ydotool").is_some() {
        match ydotool_key(&combo) {
            Ok(()) => return Ok(format!("pulsado {rendered}")),
            Err(e) => last = e,
        }
    }
    if crate::tools::which("xdotool").is_some() {
        match run_ok("xdotool", &["key", "--clearmodifiers", &rendered]) {
            Ok(()) => return Ok(format!("pulsado {rendered}")),
            Err(e) => last = e,
        }
    }
    if crate::tools::which("wtype").is_some() {
        match wtype_key(&combo) {
            Ok(()) => return Ok(format!("pulsado {rendered}")),
            Err(e) => last = e,
        }
    }
    Err(last.into())
}

pub fn mouse_click(x: Option<i64>, y: Option<i64>, button: &str) -> AppResult<String> {
    let button = parse_button(button)?;
    let x = coord(x)?;
    let y = coord(y)?;
    if x.is_some() != y.is_some() {
        return Err("indica x e y juntos, o ninguno para clic en la posición actual".into());
    }
    let mut last = String::from(MISSING);
    if crate::tools::which("ydotool").is_some() {
        match ydotool_click(x, y, button) {
            Ok(()) => return Ok(click_ok(x, y, button)),
            Err(e) => last = e,
        }
    }
    if crate::tools::which("xdotool").is_some() {
        match xdotool_click(x, y, button) {
            Ok(()) => return Ok(click_ok(x, y, button)),
            Err(e) => last = e,
        }
    }
    Err(last.into())
}

pub fn pointer_info() -> AppResult<String> {
    if crate::tools::which("xdotool").is_some() {
        let out = Command::new("xdotool")
            .args(["getmouselocation", "--shell"])
            .output()
            .map_err(|e| format!("xdotool: {e}"))?;
        if out.status.success() {
            return Ok(String::from_utf8_lossy(&out.stdout).trim().to_string());
        }
    }
    Err(
        "no pude leer el puntero (hace falta xdotool; en Wayland puro suele fallar)".into(),
    )
}

pub fn parse_combo(raw: &str) -> Result<Combo, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("atajo vacío".into());
    }
    if raw.len() > 64 {
        return Err("atajo demasiado largo".into());
    }
    let parts: Vec<&str> = raw
        .split('+')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();
    if parts.is_empty() || parts.len() > 4 {
        return Err("usa como máximo 4 teclas unidas por +".into());
    }
    let mut canon = Vec::with_capacity(parts.len());
    for part in parts {
        canon.push(canonicalize(part)?);
    }
    Ok(Combo { parts: canon })
}

impl Combo {
    pub fn xdotool(&self) -> String {
        self.parts
            .iter()
            .map(|p| match p.as_str() {
                "ctrl" => "ctrl",
                "alt" => "alt",
                "shift" => "shift",
                "super" => "super",
                other => other,
            })
            .collect::<Vec<_>>()
            .join("+")
    }
}

pub fn hides_overlay(name: &str) -> bool {
    matches!(name, "type_text" | "press_keys" | "mouse_click")
}

fn canonicalize(part: &str) -> Result<String, String> {
    let lower = part.to_lowercase();
    let mapped = match lower.as_str() {
        "ctrl" | "control" => "ctrl",
        "alt" => "alt",
        "shift" => "shift",
        "super" | "meta" | "win" | "windows" => "super",
        "return" | "enter" | "intro" => "Return",
        "escape" | "esc" => "Escape",
        "tab" => "Tab",
        "space" | "espacio" => "space",
        "backspace" => "BackSpace",
        "delete" | "del" | "suprimir" => "Delete",
        "up" | "arriba" => "Up",
        "down" | "abajo" => "Down",
        "left" | "izquierda" => "Left",
        "right" | "derecha" => "Right",
        "home" => "Home",
        "end" => "End",
        "pageup" | "prior" => "Page_Up",
        "pagedown" | "next" => "Page_Down",
        "plus" => "plus",
        "minus" | "guion" => "minus",
        "f1" => "F1",
        "f2" => "F2",
        "f3" => "F3",
        "f4" => "F4",
        "f5" => "F5",
        "f6" => "F6",
        "f7" => "F7",
        "f8" => "F8",
        "f9" => "F9",
        "f10" => "F10",
        "f11" => "F11",
        "f12" => "F12",
        other if other.len() == 1 => {
            let ch = other.chars().next().unwrap();
            if ch.is_ascii_alphanumeric() {
                other
            } else {
                return Err(format!("tecla no permitida: {part}"));
            }
        }
        _ => return Err(format!("tecla no permitida: {part}")),
    };
    Ok(mapped.to_string())
}

fn parse_button(raw: &str) -> Result<u8, String> {
    match raw.trim().to_lowercase().as_str() {
        "" | "left" | "izquierdo" | "1" => Ok(1),
        "middle" | "centro" | "2" => Ok(2),
        "right" | "derecho" | "3" => Ok(3),
        other => Err(format!("botón no permitido: {other}")),
    }
}

fn coord(value: Option<i64>) -> Result<Option<i32>, String> {
    match value {
        None => Ok(None),
        Some(n) if (0..=16_000).contains(&n) => Ok(Some(n as i32)),
        Some(_) => Err("coordenada fuera de rango (0–16000)".into()),
    }
}

fn click_ok(x: Option<i32>, y: Option<i32>, button: u8) -> String {
    match (x, y) {
        (Some(x), Some(y)) => format!("clic botón {button} en {x},{y}"),
        _ => format!("clic botón {button} en la posición actual"),
    }
}

fn run_ok(bin: &str, args: &[&str]) -> Result<(), String> {
    let out = Command::new(bin)
        .args(args)
        .output()
        .map_err(|e| format!("{bin}: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{bin}: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

fn ydotool_type(text: &str) -> Result<(), String> {
    if run_ok("ydotool", &["type", "--", text]).is_ok() {
        return Ok(());
    }
    let mut child = Command::new("ydotool")
        .args(["type", "--file", "-"])
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("ydotool: {e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(text.as_bytes())
            .map_err(|e| format!("ydotool stdin: {e}"))?;
    }
    let status = child.wait().map_err(|e| format!("ydotool: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("ydotool type falló (¿está ydotoold en marcha?)".into())
    }
}

fn ydotool_key(combo: &Combo) -> Result<(), String> {
    let named = combo.xdotool();
    if run_ok("ydotool", &["key", &named]).is_ok() {
        return Ok(());
    }
    let seq = evdev_sequence(combo)?;
    let args: Vec<String> = std::iter::once("key".into()).chain(seq).collect();
    let arg_refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run_ok("ydotool", &arg_refs)
}

fn evdev_sequence(combo: &Combo) -> Result<Vec<String>, String> {
    let codes: Result<Vec<u16>, String> = combo.parts.iter().map(|p| evdev_code(p)).collect();
    let codes = codes?;
    let mut seq = Vec::new();
    for code in &codes {
        seq.push(format!("{code}:1"));
    }
    for code in codes.iter().rev() {
        seq.push(format!("{code}:0"));
    }
    Ok(seq)
}

fn evdev_code(name: &str) -> Result<u16, String> {
    let code = match name {
        "ctrl" => 29,
        "alt" => 56,
        "shift" => 42,
        "super" => 125,
        "Return" => 28,
        "Escape" => 1,
        "Tab" => 15,
        "space" => 57,
        "BackSpace" => 14,
        "Delete" => 111,
        "Up" => 103,
        "Down" => 108,
        "Left" => 105,
        "Right" => 106,
        "Home" => 102,
        "End" => 107,
        "Page_Up" => 104,
        "Page_Down" => 109,
        "plus" => 13,
        "minus" => 12,
        "F1" => 59,
        "F2" => 60,
        "F3" => 61,
        "F4" => 62,
        "F5" => 63,
        "F6" => 64,
        "F7" => 65,
        "F8" => 66,
        "F9" => 67,
        "F10" => 68,
        "F11" => 87,
        "F12" => 88,
        "0" => 11,
        "1" => 2,
        "2" => 3,
        "3" => 4,
        "4" => 5,
        "5" => 6,
        "6" => 7,
        "7" => 8,
        "8" => 9,
        "9" => 10,
        other if other.len() == 1 => {
            const LETTERS: [u16; 26] = [
                30, 48, 46, 32, 18, 33, 34, 35, 23, 36, 37, 38, 50, 49, 24, 25, 16, 19, 31, 20,
                22, 47, 17, 45, 21, 44,
            ];
            let ch = other.chars().next().unwrap();
            if ch.is_ascii_lowercase() {
                LETTERS[(ch as u8 - b'a') as usize]
            } else {
                return Err(format!("sin keycode evdev para {name}"));
            }
        }
        _ => return Err(format!("sin keycode evdev para {name}")),
    };
    Ok(code)
}

fn wtype_key(combo: &Combo) -> Result<(), String> {
    let mut args: Vec<String> = Vec::new();
    let mut mods = Vec::new();
    let mut key = None;
    for part in &combo.parts {
        match part.as_str() {
            "ctrl" | "alt" | "shift" | "super" => {
                mods.push(part.clone());
            }
            other => key = Some(other.to_string()),
        }
    }
    for m in &mods {
        args.push("-M".into());
        args.push(m.clone());
    }
    if let Some(k) = key {
        if k.len() == 1 {
            args.push(k);
        } else {
            args.push("-k".into());
            args.push(k);
        }
    }
    for m in mods.iter().rev() {
        args.push("-m".into());
        args.push(m.clone());
    }
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    run_ok("wtype", &refs)
}

fn ydotool_click(x: Option<i32>, y: Option<i32>, button: u8) -> Result<(), String> {
    if let (Some(x), Some(y)) = (x, y) {
        let sx = x.to_string();
        let sy = y.to_string();
        if run_ok("ydotool", &["mousemove", "--absolute", &sx, &sy]).is_err() {
            run_ok("ydotool", &["mousemove", &sx, &sy])?;
        }
    }
    let named = match button {
        2 => "middle",
        3 => "right",
        _ => "left",
    };
    let zero = (button - 1).to_string();
    if run_ok("ydotool", &["click", named]).is_ok() || run_ok("ydotool", &["click", &zero]).is_ok()
    {
        return Ok(());
    }
    let mask = match button {
        2 => "0xC1",
        3 => "0xC2",
        _ => "0xC0",
    };
    run_ok("ydotool", &["click", mask])
}

fn xdotool_click(x: Option<i32>, y: Option<i32>, button: u8) -> Result<(), String> {
    if let (Some(x), Some(y)) = (x, y) {
        run_ok("xdotool", &["mousemove", "--", &x.to_string(), &y.to_string()])?;
    }
    run_ok("xdotool", &["click", "--clearmodifiers", &button.to_string()])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ctrl_c() {
        let combo = parse_combo("ctrl+c").unwrap();
        assert_eq!(combo.parts, vec!["ctrl", "c"]);
        assert_eq!(combo.xdotool(), "ctrl+c");
    }

    #[test]
    fn parses_spanish_enter() {
        assert_eq!(parse_combo("intro").unwrap().xdotool(), "Return");
        assert_eq!(parse_combo("Enter").unwrap().xdotool(), "Return");
    }

    #[test]
    fn parses_ctrl_shift_t() {
        assert_eq!(parse_combo("ctrl+shift+t").unwrap().xdotool(), "ctrl+shift+t");
    }

    #[test]
    fn rejects_shell_payload() {
        assert!(parse_combo("ctrl+c; rm -rf /").is_err());
        assert!(parse_combo("$(reboot)").is_err());
        assert!(parse_combo("").is_err());
    }

    #[test]
    fn rejects_unknown_key() {
        assert!(parse_combo("ctrl+foobar").is_err());
    }

    #[test]
    fn evdev_ctrl_c_sequence() {
        let combo = parse_combo("ctrl+c").unwrap();
        let seq = evdev_sequence(&combo).unwrap();
        assert_eq!(seq, vec!["29:1", "46:1", "46:0", "29:0"]);
    }

    #[test]
    fn empty_type_is_rejected() {
        assert!(type_text("").is_err());
    }

    #[test]
    fn button_and_coord_validation() {
        assert_eq!(parse_button("left").unwrap(), 1);
        assert_eq!(parse_button("derecho").unwrap(), 3);
        assert!(parse_button("side").is_err());
        assert!(coord(Some(-1)).is_err());
        assert_eq!(coord(Some(12)).unwrap(), Some(12));
    }
}
