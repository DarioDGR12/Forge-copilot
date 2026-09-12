use crate::error::AppResult;
use crate::llm::types::{AssistantTurn, ChatMessage, Role, ToolCall};
use crate::settings::{effective_base_url, effective_model, Settings};
use futures_util::StreamExt;
use serde_json::{json, Value};

pub async fn complete(
    http: &reqwest::Client,
    settings: &Settings,
    api_key: Option<&str>,
    messages: &[ChatMessage],
    on_token: &mut (dyn FnMut(&str) + Send),
) -> AppResult<AssistantTurn> {
    let url = format!("{}/chat/completions", effective_base_url(settings));
    let body = json!({
        "model": effective_model(settings),
        "messages": to_openai_messages(messages),
        "tools": crate::llm::types::tool_specs(),
        "stream": true,
    });

    let mut req = http.post(&url).header("Content-Type", "application/json");
    if let Some(key) = api_key.filter(|k| !k.is_empty()) {
        req = req.bearer_auth(key);
    }
    if settings.provider == "openrouter" {
        req = req
            .header("HTTP-Referer", "https://github.com/DarioDGR12/Forge-copilot")
            .header("X-Title", "Forge Copilot");
    }

    let response = req.json(&body).send().await?;
    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("OpenAI-compat {status}: {text}").into());
    }

    let mut acc = StreamAcc::default();
    let mut bytes = response.bytes_stream();
    let mut buf = String::new();
    while let Some(chunk) = bytes.next().await {
        buf.push_str(&String::from_utf8_lossy(&chunk?));
        while let Some(pos) = buf.find('\n') {
            let line = buf[..pos].trim().to_string();
            buf = buf[pos + 1..].to_string();
            if let Some(data) = line.strip_prefix("data: ") {
                if data.trim() == "[DONE]" {
                    return Ok(acc.into_turn());
                }
                if let Ok(value) = serde_json::from_str::<Value>(data) {
                    acc.ingest(&value, on_token);
                }
            }
        }
    }
    Ok(acc.into_turn())
}

pub async fn test_connection(
    http: &reqwest::Client,
    settings: &Settings,
    api_key: Option<&str>,
) -> AppResult<String> {
    let url = format!("{}/models", effective_base_url(settings));
    let mut req = http.get(&url);
    if let Some(key) = api_key.filter(|k| !k.is_empty()) {
        req = req.bearer_auth(key);
    }
    let response = req.send().await?;
    let status = response.status();
    if status.is_success() {
        Ok(format!("Conexión correcta ({status})"))
    } else {
        let text = response.text().await.unwrap_or_default();
        Err(format!("falló {status}: {text}").into())
    }
}

pub(crate) fn to_openai_messages(messages: &[ChatMessage]) -> Value {
    let images = crate::llm::vision::recent_image_ids(messages);
    let mut out = vec![json!({"role":"system","content": crate::host::system_prompt()})];
    for msg in messages {
        match msg.role {
            Role::User => out.push(json!({"role":"user","content": msg.content})),
            Role::Assistant => {
                let mut obj = json!({"role":"assistant","content": msg.content});
                if let Some(calls) = &msg.tool_calls {
                    obj["tool_calls"] = json!(calls
                        .iter()
                        .map(|c| json!({
                            "id": c.id,
                            "type": "function",
                            "function": { "name": c.name, "arguments": c.arguments }
                        }))
                        .collect::<Vec<_>>());
                }
                out.push(obj);
            }
            Role::Tool => {
                out.push(json!({
                    "role": "tool",
                    "tool_call_id": msg.tool_call_id,
                    "content": msg.content
                }));
                if images.contains(&msg.id) {
                    if let Some(b64) = crate::llm::vision::image_data(msg) {
                        out.push(crate::llm::vision::openai_image_followup(b64));
                    }
                }
            }
            Role::System => {}
        }
    }
    Value::Array(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attaches_recent_screenshot() {
        let msg = ChatMessage {
            id: "shot1".into(),
            conversation_id: "c".into(),
            role: Role::Tool,
            content: "captura".into(),
            tool_name: Some("screenshot".into()),
            tool_call_id: Some("call1".into()),
            tool_calls: None,
            created_at: 1,
            status: None,
            image_base64: Some("abc123".into()),
        };
        let value = to_openai_messages(&[msg]);
        let arr = value.as_array().unwrap();
        assert!(arr.iter().any(|m| m["role"] == "tool"));
        assert!(arr.iter().any(|m| {
            m["content"]
                .as_array()
                .is_some_and(|c| c.iter().any(|p| p["type"] == "image_url"))
        }));
    }
}

#[derive(Default)]
struct StreamAcc {
    text: String,
    tools: Vec<ToolCall>,
}

impl StreamAcc {
    fn ingest(&mut self, value: &Value, on_token: &mut (dyn FnMut(&str) + Send)) {
        let Some(choices) = value.get("choices").and_then(|c| c.as_array()) else {
            return;
        };
        for choice in choices {
            if let Some(delta) = choice.get("delta") {
                if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                    if !content.is_empty() {
                        self.text.push_str(content);
                        on_token(content);
                    }
                }
                if let Some(tool_calls) = delta.get("tool_calls").and_then(|t| t.as_array()) {
                    for call in tool_calls {
                        let index = call.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as usize;
                        while self.tools.len() <= index {
                            self.tools.push(ToolCall {
                                id: String::new(),
                                name: String::new(),
                                arguments: String::new(),
                            });
                        }
                        if let Some(id) = call.get("id").and_then(|v| v.as_str()) {
                            self.tools[index].id = id.to_string();
                        }
                        if let Some(func) = call.get("function") {
                            if let Some(name) = func.get("name").and_then(|v| v.as_str()) {
                                self.tools[index].name.push_str(name);
                            }
                            if let Some(args) = func.get("arguments").and_then(|v| v.as_str()) {
                                self.tools[index].arguments.push_str(args);
                            }
                        }
                    }
                }
            }
        }
    }

    fn into_turn(self) -> AssistantTurn {
        let tools = self
            .tools
            .into_iter()
            .filter(|t| !t.name.is_empty())
            .map(|mut t| {
                if t.id.is_empty() {
                    t.id = format!("call_{}", uuid::Uuid::new_v4().simple());
                }
                t
            })
            .collect();
        AssistantTurn {
            text: self.text,
            tool_calls: tools,
        }
    }
}
