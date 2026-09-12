use crate::error::AppResult;
use std::process::Command;

const MAX: usize = 16 * 1024;

pub fn read() -> AppResult<String> {
    let attempts: &[(&str, &[&str])] = &[
        ("wl-paste", &["--no-newline"]),
        ("xclip", &["-selection", "clipboard", "-o"]),
        ("xsel", &["-b"]),
    ];
    let raw = run_first(attempts)?;
    Ok(truncate(&raw, MAX))
}

pub fn write(text: &str) -> AppResult<String> {
    if text.is_empty() {
        return Err("texto vacío".into());
    }
    if text.len() > MAX {
        return Err(format!("el portapapeles admite como máximo {MAX} bytes").into());
    }
    let attempts: &[(&str, &[&str])] = &[
        ("wl-copy", &[]),
        ("xclip", &["-selection", "clipboard"]),
        ("xsel", &["-ib"]),
    ];
    stdin_first(attempts, text)?;
    Ok(format!("copiados {} caracteres al portapapeles", text.chars().count()))
}

fn run_first(attempts: &[(&str, &[&str])]) -> AppResult<String> {
    let mut last = String::from("instala wl-clipboard o xclip");
    for (bin, args) in attempts {
        if crate::tools::which(bin).is_none() {
            continue;
        }
        match Command::new(bin).args(*args).output() {
            Ok(out) if out.status.success() => {
                return Ok(String::from_utf8_lossy(&out.stdout).into_owned());
            }
            Ok(out) => {
                last = format!("{bin}: {}", String::from_utf8_lossy(&out.stderr).trim());
            }
            Err(e) => last = format!("{bin}: {e}"),
        }
    }
    Err(last.into())
}

fn stdin_first(attempts: &[(&str, &[&str])], text: &str) -> AppResult<()> {
    let mut last = String::from("instala wl-clipboard o xclip");
    for (bin, args) in attempts {
        if crate::tools::which(bin).is_none() {
            continue;
        }
        let mut child = Command::new(bin)
            .args(*args)
            .stdin(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| format!("{bin}: {e}"))?;
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            stdin.write_all(text.as_bytes())?;
        }
        match child.wait() {
            Ok(status) if status.success() => return Ok(()),
            Ok(status) => last = format!("{bin} exit={status}"),
            Err(e) => last = format!("{bin}: {e}"),
        }
    }
    Err(last.into())
}

fn truncate(text: &str, max: usize) -> String {
    if text.len() <= max {
        text.to_string()
    } else {
        format!("{}\n…[truncado]", &text[..max])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_write_is_rejected() {
        let err = write("").unwrap_err().to_string();
        assert!(err.contains("vacío"));
    }

    #[test]
    fn truncate_marks_overflow() {
        let out = truncate(&"a".repeat(20), 8);
        assert!(out.contains("truncado"));
        assert!(out.starts_with("aaaaaaaa"));
    }
}
