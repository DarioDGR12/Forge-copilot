mod agent;
mod diagnostics;
mod error;
mod host;
mod llm;
mod policy;
mod secrets;
mod settings;
mod state;
mod store;
mod tools;
mod windows;

use crate::llm::types::{ChatMessage, Conversation};
use crate::settings::{KeyStatus, Settings};
use crate::state::AppState;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

#[tauri::command]
fn list_conversations(state: tauri::State<Arc<AppState>>) -> Result<Vec<Conversation>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    store::list_conversations(&db).map_err(Into::into)
}

#[tauri::command]
fn get_messages(
    state: tauri::State<Arc<AppState>>,
    conversation_id: String,
) -> Result<Vec<ChatMessage>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    store::load_messages(&db, &conversation_id).map_err(Into::into)
}

#[tauri::command]
fn create_conversation(state: tauri::State<Arc<AppState>>) -> Result<Conversation, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    store::create_conversation(&db, "Nueva conversación").map_err(Into::into)
}

#[tauri::command]
fn delete_conversation(state: tauri::State<Arc<AppState>>, id: String) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    store::delete_conversation(&db, &id).map_err(Into::into)
}

#[tauri::command]
fn send_message(
    app: tauri::AppHandle,
    state: tauri::State<Arc<AppState>>,
    conversation_id: Option<String>,
    content: String,
    allow_duplicate: Option<bool>,
) -> Result<Conversation, String> {
    let _ = allow_duplicate;
    let content = content.trim().to_string();
    if content.is_empty() {
        return Err("escribe un mensaje".into());
    }

    let conv = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let conv = match conversation_id {
            Some(id) if !id.is_empty() => store::get_conversation(&db, &id)?
                .ok_or_else(|| "conversación no encontrada".to_string())?,
            _ => store::create_conversation(&db, &content)?,
        };
        let msg = store::new_message(&conv.id, llm::types::Role::User, &content);
        store::insert_message(&db, &msg)?;
        conv
    };

    let cancel = Arc::new(AtomicBool::new(false));
    {
        let mut runs = state.runs.lock().map_err(|e| e.to_string())?;
        if let Some(prev) = runs.get(&conv.id) {
            if !prev.load(Ordering::Relaxed) {
                return Err("ya hay una respuesta en curso".into());
            }
        }
        runs.insert(conv.id.clone(), cancel.clone());
    }

    let app_handle = app.clone();
    let state_arc = Arc::clone(&state);
    let cid = conv.id.clone();
    tauri::async_runtime::spawn(async move {
        agent::run(app_handle, state_arc, cid, cancel).await;
    });

    Ok(conv)
}

#[tauri::command]
fn cancel_run(state: tauri::State<Arc<AppState>>, conversation_id: String) -> Result<(), String> {
    let runs = state.runs.lock().map_err(|e| e.to_string())?;
    if let Some(flag) = runs.get(&conversation_id) {
        flag.store(true, Ordering::Relaxed);
    }
    Ok(())
}

#[tauri::command]
async fn resolve_approval(
    state: tauri::State<'_, Arc<AppState>>,
    request_id: String,
    allowed: bool,
) -> Result<(), String> {
    let mut pending = state.pending.lock().await;
    if let Some(tx) = pending.remove(&request_id) {
        let _ = tx.send(allowed);
    }
    Ok(())
}

#[tauri::command]
fn get_settings(state: tauri::State<Arc<AppState>>) -> Result<Settings, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    store::load_settings(&db).map_err(Into::into)
}

#[tauri::command]
fn save_settings(state: tauri::State<Arc<AppState>>, settings: Settings) -> Result<Settings, String> {
    let mut cleaned = settings;
    cleaned.allowed_roots = cleaned
        .allowed_roots
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if cleaned.allowed_roots.is_empty() {
        cleaned = Settings {
            allowed_roots: Settings::default().allowed_roots,
            ..cleaned
        };
    }
    let db = state.db.lock().map_err(|e| e.to_string())?;
    store::save_settings(&db, &cleaned)?;
    Ok(cleaned)
}

#[tauri::command]
fn set_api_key(provider: String, key: String) -> Result<KeyStatus, String> {
    secrets::set_key(&provider, &key)?;
    secrets::status_for(&provider).map_err(Into::into)
}

#[tauri::command]
fn clear_api_key(provider: String) -> Result<KeyStatus, String> {
    secrets::clear_key(&provider)?;
    secrets::status_for(&provider).map_err(Into::into)
}

#[tauri::command]
fn key_status(provider: String) -> Result<KeyStatus, String> {
    secrets::status_for(&provider).map_err(Into::into)
}

#[tauri::command]
async fn test_connection(state: tauri::State<'_, Arc<AppState>>) -> Result<String, String> {
    let settings = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        store::load_settings(&db)?
    };
    let key = secrets::get_key(&settings.provider)?;
    llm::test_connection(&state.http, &settings, key.as_deref())
        .await
        .map_err(Into::into)
}

#[tauri::command]
fn system_diagnostics() -> crate::diagnostics::Diagnostics {
    crate::diagnostics::collect()
}

#[tauri::command]
fn window_hide(app: tauri::AppHandle) {
    windows::hide(&app);
}

#[tauri::command]
fn window_toggle(app: tauri::AppHandle) {
    windows::toggle(&app);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db = store::open().expect("no se pudo abrir la base de datos");
    let state = Arc::new(AppState::new(db).expect("no se pudo iniciar el estado"));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            list_conversations,
            get_messages,
            create_conversation,
            delete_conversation,
            send_message,
            cancel_run,
            resolve_approval,
            get_settings,
            save_settings,
            set_api_key,
            clear_api_key,
            key_status,
            test_connection,
            system_diagnostics,
            window_hide,
            window_toggle
        ])
        .setup(|app| {
            windows::dock_right(app.handle());
            windows::prevent_close(app.handle());
            if let Err(err) = windows::setup_tray(app.handle()) {
                eprintln!("bandeja: {err}");
            }
            let shortcut = {
                let state = app.state::<Arc<AppState>>();
                let db = state.db.lock().expect("db");
                store::load_settings(&db)
                    .map(|s| s.shortcut)
                    .unwrap_or_else(|_| "ctrl+shift+space".into())
            };
            if let Err(err) = app.global_shortcut().on_shortcut(shortcut.as_str(), |app, _, e| {
                if e.state == ShortcutState::Pressed {
                    windows::toggle(app);
                }
            }) {
                eprintln!("atajo global ({shortcut}): {err}");
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error al iniciar Forge Copilot");
}
