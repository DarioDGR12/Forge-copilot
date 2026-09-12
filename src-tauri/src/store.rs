use crate::error::AppResult;
use crate::llm::types::{ChatMessage, Conversation, Role, ToolCall};
use crate::settings::Settings;
use rusqlite::{params, Connection};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn db_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
        .join("forge-copilot")
        .join("forge.db")
}

pub fn open() -> AppResult<Connection> {
    let path = db_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            tool_name TEXT,
            tool_call_id TEXT,
            tool_calls_json TEXT,
            created_at INTEGER NOT NULL,
            FOREIGN KEY(conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
        );
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        ",
    )?;
    Ok(conn)
}

pub fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn load_settings(conn: &Connection) -> AppResult<Settings> {
    let raw: Result<String, rusqlite::Error> = conn.query_row(
        "SELECT value FROM settings WHERE key = 'app'",
        [],
        |row| row.get(0),
    );
    match raw {
        Ok(json) => Ok(serde_json::from_str(&json).unwrap_or_default()),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(Settings::default()),
        Err(e) => Err(e.into()),
    }
}

pub fn save_settings(conn: &Connection, settings: &Settings) -> AppResult<()> {
    let json = serde_json::to_string(settings)?;
    conn.execute(
        "INSERT INTO settings(key, value) VALUES('app', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![json],
    )?;
    Ok(())
}

pub fn get_conversation(conn: &Connection, id: &str) -> AppResult<Option<Conversation>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, created_at, updated_at FROM conversations WHERE id = ?1",
    )?;
    let mut rows = stmt.query(params![id])?;
    if let Some(row) = rows.next()? {
        Ok(Some(Conversation {
            id: row.get(0)?,
            title: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
        }))
    } else {
        Ok(None)
    }
}

pub fn list_conversations(conn: &Connection) -> AppResult<Vec<Conversation>> {
    let mut stmt = conn.prepare(
        "SELECT id, title, created_at, updated_at FROM conversations ORDER BY updated_at DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Conversation {
            id: row.get(0)?,
            title: row.get(1)?,
            created_at: row.get(2)?,
            updated_at: row.get(3)?,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn create_conversation(conn: &Connection, title: &str) -> AppResult<Conversation> {
    let now = now_secs();
    let conv = Conversation {
        id: uuid::Uuid::new_v4().to_string(),
        title: title_from(title),
        created_at: now,
        updated_at: now,
    };
    conn.execute(
        "INSERT INTO conversations(id, title, created_at, updated_at) VALUES(?1, ?2, ?3, ?4)",
        params![conv.id, conv.title, conv.created_at, conv.updated_at],
    )?;
    Ok(conv)
}

pub fn delete_conversation(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM conversations WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn touch_conversation(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE conversations SET updated_at = ?1 WHERE id = ?2",
        params![now_secs(), id],
    )?;
    Ok(())
}

pub fn load_messages(conn: &Connection, conversation_id: &str) -> AppResult<Vec<ChatMessage>> {
    let mut stmt = conn.prepare(
        "SELECT id, conversation_id, role, content, tool_name, tool_call_id, tool_calls_json, created_at
         FROM messages WHERE conversation_id = ?1 ORDER BY created_at ASC, id ASC",
    )?;
    let rows = stmt.query_map(params![conversation_id], |row| {
        let role_raw: String = row.get(2)?;
        let tool_calls_json: Option<String> = row.get(6)?;
        Ok(ChatMessage {
            id: row.get(0)?,
            conversation_id: row.get(1)?,
            role: Role::from_str(&role_raw),
            content: row.get(3)?,
            tool_name: row.get(4)?,
            tool_call_id: row.get(5)?,
            tool_calls: tool_calls_json
                .as_deref()
                .and_then(|raw| serde_json::from_str::<Vec<ToolCall>>(raw).ok()),
            created_at: row.get(7)?,
            status: None,
            image_base64: None,
        })
    })?;
    rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
}

pub fn insert_message(conn: &Connection, message: &ChatMessage) -> AppResult<()> {
    let tool_calls_json = message
        .tool_calls
        .as_ref()
        .map(serde_json::to_string)
        .transpose()?;
    conn.execute(
        "INSERT INTO messages(id, conversation_id, role, content, tool_name, tool_call_id, tool_calls_json, created_at)
         VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            message.id,
            message.conversation_id,
            message.role.as_str(),
            message.content,
            message.tool_name,
            message.tool_call_id,
            tool_calls_json,
            message.created_at
        ],
    )?;
    touch_conversation(conn, &message.conversation_id)?;
    Ok(())
}

pub fn title_from(text: &str) -> String {
    let compact: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() <= 42 {
        if compact.is_empty() {
            "Nueva conversación".into()
        } else {
            compact
        }
    } else {
        let trimmed: String = compact.chars().take(41).collect();
        format!("{trimmed}…")
    }
}

pub fn new_message(
    conversation_id: &str,
    role: Role,
    content: impl Into<String>,
) -> ChatMessage {
    ChatMessage {
        id: uuid::Uuid::new_v4().to_string(),
        conversation_id: conversation_id.to_string(),
        role,
        content: content.into(),
        tool_name: None,
        tool_call_id: None,
        tool_calls: None,
        created_at: now_secs(),
        status: None,
        image_base64: None,
    }
}
