use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum AppError {
    Msg(String),
    Sqlite(rusqlite::Error),
    Http(reqwest::Error),
    Json(serde_json::Error),
    Io(std::io::Error),
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Msg(m) => write!(f, "{m}"),
            Self::Sqlite(e) => write!(f, "base de datos: {e}"),
            Self::Http(e) => write!(f, "red: {e}"),
            Self::Json(e) => write!(f, "json: {e}"),
            Self::Io(e) => write!(f, "io: {e}"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<String> for AppError {
    fn from(value: String) -> Self {
        Self::Msg(value)
    }
}

impl From<&str> for AppError {
    fn from(value: &str) -> Self {
        Self::Msg(value.to_string())
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Sqlite(value)
    }
}

impl From<reqwest::Error> for AppError {
    fn from(value: reqwest::Error) -> Self {
        Self::Http(value)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<AppError> for String {
    fn from(value: AppError) -> Self {
        value.to_string()
    }
}

pub type AppResult<T> = Result<T, AppError>;
