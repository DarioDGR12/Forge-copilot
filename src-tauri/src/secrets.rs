use crate::error::AppResult;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

const SERVICE: &str = "forge-copilot";

fn fallback_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join("forge-copilot")
        .join("secrets.json")
}

fn read_fallback() -> serde_json::Map<String, serde_json::Value> {
    let path = fallback_path();
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn write_fallback(map: &serde_json::Map<String, serde_json::Value>) -> AppResult<()> {
    let path = fallback_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, serde_json::to_vec_pretty(map)?)?;
    let mut perms = fs::metadata(&path)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(&path, perms)?;
    Ok(())
}

pub fn set_key(provider: &str, key: &str) -> AppResult<()> {
    let key = key.trim();
    if key.is_empty() {
        return clear_key(provider);
    }
    if let Ok(entry) = keyring::Entry::new(SERVICE, provider) {
        if entry.set_password(key).is_ok() {
            return Ok(());
        }
    }
    let mut map = read_fallback();
    map.insert(provider.to_string(), serde_json::Value::String(key.to_string()));
    write_fallback(&map)
}

pub fn get_key(provider: &str) -> AppResult<Option<String>> {
    if let Ok(entry) = keyring::Entry::new(SERVICE, provider) {
        if let Ok(password) = entry.get_password() {
            if !password.is_empty() {
                return Ok(Some(password));
            }
        }
    }
    Ok(read_fallback()
        .get(provider)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string()))
}

pub fn clear_key(provider: &str) -> AppResult<()> {
    if let Ok(entry) = keyring::Entry::new(SERVICE, provider) {
        let _ = entry.delete_password();
    }
    let mut map = read_fallback();
    if map.remove(provider).is_some() {
        write_fallback(&map)?;
    }
    Ok(())
}

pub fn hint_for(key: &str) -> String {
    let trimmed = key.trim();
    if trimmed.len() <= 4 {
        return "••••".into();
    }
    format!("…{}", &trimmed[trimmed.len() - 4..])
}

pub fn status_for(provider: &str) -> AppResult<crate::settings::KeyStatus> {
    let key = get_key(provider)?;
    Ok(crate::settings::KeyStatus {
        provider: provider.to_string(),
        configured: key.as_ref().is_some_and(|k| !k.is_empty()),
        hint: key.as_deref().map(hint_for),
    })
}
