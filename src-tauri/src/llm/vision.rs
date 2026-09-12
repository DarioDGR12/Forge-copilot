use crate::llm::types::{ChatMessage, Role};
use serde_json::{json, Value};
use std::collections::HashSet;

const MAX_IMAGES: usize = 2;

pub fn recent_image_ids(messages: &[ChatMessage]) -> HashSet<String> {
    messages
        .iter()
        .rev()
        .filter(|m| m.role == Role::Tool && image_data(m).is_some())
        .take(MAX_IMAGES)
        .map(|m| m.id.clone())
        .collect()
}

pub fn image_data(message: &ChatMessage) -> Option<&str> {
    message
        .image_base64
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

pub fn openai_image_followup(b64: &str) -> Value {
    json!({
        "role": "user",
        "content": [
            {
                "type": "text",
                "text": "Imagen capturada por una herramienta. Úsala para responder; no inventes lo que no se ve."
            },
            {
                "type": "image_url",
                "image_url": { "url": format!("data:image/png;base64,{b64}") }
            }
        ]
    })
}

pub fn anthropic_tool_content(message: &ChatMessage, attach_image: bool) -> Value {
    let mut blocks = vec![json!({"type": "text", "text": message.content})];
    if attach_image {
        if let Some(b64) = image_data(message) {
            blocks.push(json!({
                "type": "image",
                "source": {
                    "type": "base64",
                    "media_type": "image/png",
                    "data": b64
                }
            }));
        }
    }
    json!(blocks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::types::ChatMessage;

    fn tool(id: &str, image: Option<&str>) -> ChatMessage {
        ChatMessage {
            id: id.into(),
            conversation_id: "c".into(),
            role: Role::Tool,
            content: "captura".into(),
            tool_name: Some("screenshot".into()),
            tool_call_id: Some(id.into()),
            tool_calls: None,
            created_at: 1,
            status: None,
            image_base64: image.map(str::to_string),
        }
    }

    #[test]
    fn keeps_only_last_two_images() {
        let msgs = vec![
            tool("a", Some("aaa")),
            tool("b", Some("bbb")),
            tool("c", Some("ccc")),
            tool("d", None),
        ];
        let ids = recent_image_ids(&msgs);
        assert!(!ids.contains("a"));
        assert!(ids.contains("b"));
        assert!(ids.contains("c"));
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn openai_followup_uses_data_url() {
        let v = openai_image_followup("abc");
        let url = v["content"][1]["image_url"]["url"].as_str().unwrap();
        assert!(url.starts_with("data:image/png;base64,abc"));
    }

    #[test]
    fn anthropic_includes_image_block() {
        let msg = tool("x", Some("pngdata"));
        let content = anthropic_tool_content(&msg, true);
        assert_eq!(content.as_array().unwrap().len(), 2);
        assert_eq!(content[1]["type"], "image");
    }
}
