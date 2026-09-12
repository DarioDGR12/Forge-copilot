use crate::error::AppResult;
use crate::policy::expand_path;
use std::fs;

const MAX_READ: usize = 256 * 1024;
const MAX_WRITE: usize = 1024 * 1024;
const MAX_DIR: usize = 200;

pub fn read_file(path: &str) -> AppResult<String> {
    let path = expand_path(path);
    let bytes = fs::read(&path)?;
    if bytes.len() > MAX_READ {
        return Err(format!(
            "el archivo pesa {} bytes; el límite de lectura es {MAX_READ}",
            bytes.len()
        )
        .into());
    }
    if is_binary(&bytes) {
        return Err(format!("el archivo parece binario: {}", path.display()).into());
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

pub fn write_file(path: &str, content: &str) -> AppResult<String> {
    if content.len() > MAX_WRITE {
        return Err(format!("el contenido supera el límite de {MAX_WRITE} bytes").into());
    }
    let path = expand_path(path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, content)?;
    Ok(format!("escrito {} bytes en {}", content.len(), path.display()))
}

pub fn list_dir(path: &str) -> AppResult<String> {
    let path = expand_path(path);
    let mut entries: Vec<_> = fs::read_dir(&path)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|e| e.file_name());
    let total = entries.len();
    let mut lines = vec![format!("{} ({total} entradas)", path.display())];
    for entry in entries.into_iter().take(MAX_DIR) {
        let meta = entry.metadata()?;
        let kind = if meta.is_dir() { "dir" } else { "file" };
        lines.push(format!(
            "{kind:4} {:>10} {}",
            meta.len(),
            entry.file_name().to_string_lossy()
        ));
    }
    if total > MAX_DIR {
        lines.push(format!("… y {} más", total - MAX_DIR));
    }
    Ok(lines.join("\n"))
}

fn is_binary(bytes: &[u8]) -> bool {
    bytes.iter().take(512).any(|b| *b == 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn roundtrip_temp_file() {
        let dir = std::env::temp_dir().join(format!("forge-fs-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("note.txt");
        write_file(file.to_str().unwrap(), "hola forge").unwrap();
        let read = read_file(file.to_str().unwrap()).unwrap();
        assert_eq!(read, "hola forge");
        let listing = list_dir(dir.to_str().unwrap()).unwrap();
        assert!(listing.contains("note.txt"));
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn rejects_binary() {
        let dir = std::env::temp_dir().join(format!("forge-bin-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("blob.bin");
        let mut f = fs::File::create(&file).unwrap();
        f.write_all(&[0, 1, 2, 3]).unwrap();
        let err = read_file(file.to_str().unwrap()).unwrap_err().to_string();
        assert!(err.contains("binario"));
        fs::remove_dir_all(dir).unwrap();
    }
}
