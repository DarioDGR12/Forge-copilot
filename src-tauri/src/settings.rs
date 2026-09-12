use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub provider: String,
    pub model: String,
    pub base_url: String,
    pub allowed_roots: Vec<String>,
    pub shortcut: String,
}

impl Default for Settings {
    fn default() -> Self {
        let home = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("/"))
            .to_string_lossy()
            .to_string();
        Self {
            provider: "demo".into(),
            model: default_model_for("demo").into(),
            base_url: String::new(),
            allowed_roots: vec![home],
            shortcut: "ctrl+shift+space".into(),
        }
    }
}

pub fn default_model_for(provider: &str) -> &'static str {
    match provider {
        "openai" => "gpt-4o-mini",
        "anthropic" => "claude-sonnet-4-5",
        "openrouter" => "openai/gpt-4o-mini",
        "ollama" => "llama3.2",
        _ => "forge-demo",
    }
}

pub fn effective_base_url(settings: &Settings) -> String {
    let custom = settings.base_url.trim().trim_end_matches('/');
    if !custom.is_empty() {
        return custom.to_string();
    }
    match settings.provider.as_str() {
        "openai" => "https://api.openai.com/v1".into(),
        "openrouter" => "https://openrouter.ai/api/v1".into(),
        "ollama" => "http://127.0.0.1:11434/v1".into(),
        "anthropic" => "https://api.anthropic.com".into(),
        _ => String::new(),
    }
}

pub fn effective_model(settings: &Settings) -> String {
    let model = settings.model.trim();
    if model.is_empty() {
        default_model_for(&settings.provider).to_string()
    } else {
        model.to_string()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyStatus {
    pub provider: String,
    pub configured: bool,
    pub hint: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_urls() {
        let mut s = Settings::default();
        s.provider = "openai".into();
        assert_eq!(effective_base_url(&s), "https://api.openai.com/v1");
        s.base_url = "https://example.com/v1/".into();
        assert_eq!(effective_base_url(&s), "https://example.com/v1");
    }
}
