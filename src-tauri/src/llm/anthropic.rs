use crate::error::AppResult;
use crate::llm::types::{
    anthropic_tools, AssistantTurn, ChatMessage, Role, ToolCall, SYSTEM_PROMPT,
};
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
    let key = api_key.filter(|k| !k.is_empty()).ok_or("falta la API key de Anthropic")?;
    let url = format!("{}/v1/messages", effective_base_url(settings));
    let body = json!({
        "model": effective_model(settings),
        "max_tokens": 4096,
        "system": SYSTEM_PROMPT,
        "messages": to_anthropic_messages(messages),
        "tools": anthropic_tools(),
        "stream": true,
    });

    let response = http
        .post(url)
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        return Err(format!("Anthropic {status}: {text}").into());
    }

    let mut acc = AnthropicAcc::default();
    let mut bytes = response.bytes_stream();
    let mut buf = String::new();
    while let Some(chunk) = bytes.next().await {
        buf.push_str(&String::from_utf8_lossy(&chunk?));
        while let Some(pos) = buf.find('\n') {
            let line = buf[..pos].trim().to_string();
            buf = buf[pos + 1..].to_string();
            if let Some(data) = line.strip_prefix("data: ") {
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
    let key = api_key.filter(|k| !k.is_empty()).ok_or("falta la API key de Anthropic")?;
    let url = format!("{}/v1/models", effective_base_url(settings));
    let response = http
        .get(url)
        .header("x-api-key", key)
        .header("anthropic-version", "2023-06-01")
        .send()
        .await?;
    let status = response.status();
    if status.is_success() {
        Ok(format!("Conexión correcta ({status})"))
    } else {
        let text = response.text().await.unwrap_or_default();
        Err(format!("falló {status}: {text}").into())
    }
}

fn to_anthropic_messages(messages: &[ChatMessage]) -> Value {
    let mut out: Vec<Value> = Vec::new();
    for msg in messages {
        match msg.role {
            Role::User => out.push(json!({"role":"user","content": msg.content})),
            Role::Assistant => {
                let mut content = Vec::new();
                if !msg.content.is_empty() {
                    content.push(json!({"type":"text","text": msg.content}));
                }
                if let Some(calls) = &msg.tool_calls {
                    for call in calls {
                        let input: Value =
                            serde_json::from_str(&call.arguments).unwrap_or(json!({}));
                        content.push(json!({
                            "type": "tool_use",
                            "id": call.id,
                            "name": call.name,
                            "input": input
                        }));
                    }
                }
                if !content.is_empty() {
                    out.push(json!({"role":"assistant","content": content}));
                }
            }
            Role::Tool => {
                out.push(json!({
                    "role": "user",
                    "content": [{
                        "type": "tool_result",
                        "tool_use_id": msg.tool_call_id,
                        "content": msg.content
                    }]
                }));
            }
            Role::System => {}
        }
    }
    Value::Array(out)
}

#[derive(Default)]
struct AnthropicAcc {
    text: String,
    tools: Vec<ToolCall>,
}

impl AnthropicAcc {
    fn ingest(&mut self, value: &Value, on_token: &mut (dyn FnMut(&str) + Send)) {
        let kind = value.get("type").and_then(|v| v.as_str()).unwrap_or("");
        match kind {
            "content_block_start" => {
                if let Some(block) = value.get("content_block") {
                    if block.get("type").and_then(|v| v.as_str()) == Some("tool_use") {
                        self.tools.push(ToolCall {
                            id: block
                                .get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            name: block
                                .get("name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            arguments: String::new(),
                        });
                    }
                }
            }
            "content_block_delta" => {
                if let Some(delta) = value.get("delta") {
                    if let Some(text) = delta.get("text").and_then(|v| v.as_str()) {
                        self.text.push_str(text);
                        on_token(text);
                    }
                    if let Some(partial) = delta.get("partial_json").and_then(|v| v.as_str()) {
                        if let Some(last) = self.tools.last_mut() {
                            last.arguments.push_str(partial);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn into_turn(self) -> AssistantTurn {
        AssistantTurn {
            text: self.text,
            tool_calls: self.tools.into_iter().filter(|t| !t.name.is_empty()).collect(),
        }
    }
}
