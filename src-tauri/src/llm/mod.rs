pub mod anthropic;
pub mod demo;
pub mod openai;
pub mod types;
pub mod vision;

use crate::error::AppResult;
use crate::llm::types::{AssistantTurn, ChatMessage};
use crate::settings::Settings;

pub async fn complete(
    http: &reqwest::Client,
    settings: &Settings,
    api_key: Option<&str>,
    messages: &[ChatMessage],
    on_token: &mut (dyn FnMut(&str) + Send),
) -> AppResult<AssistantTurn> {
    match settings.provider.as_str() {
        "anthropic" => anthropic::complete(http, settings, api_key, messages, on_token).await,
        "openai" | "openrouter" | "ollama" => {
            openai::complete(http, settings, api_key, messages, on_token).await
        }
        _ => demo::complete(messages, on_token).await,
    }
}

pub async fn test_connection(
    http: &reqwest::Client,
    settings: &Settings,
    api_key: Option<&str>,
) -> AppResult<String> {
    match settings.provider.as_str() {
        "demo" => Ok("Modo demostración listo (no requiere clave)".into()),
        "anthropic" => anthropic::test_connection(http, settings, api_key).await,
        _ => openai::test_connection(http, settings, api_key).await,
    }
}
