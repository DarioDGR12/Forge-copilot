use crate::error::AppResult;
use crate::llm::types::{Role, ToolCall};
use crate::policy::{self, Risk};
use crate::settings::Settings;
use crate::state::AppState;
use crate::store;
use crate::tools;
use serde::Serialize;
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPayload {
    pub conversation_id: String,
    pub text: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolPayload {
    pub conversation_id: String,
    pub id: String,
    pub name: String,
    pub arguments: String,
    pub result: Option<String>,
    pub image_base64: Option<String>,
    pub status: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalPayload {
    pub conversation_id: String,
    pub request_id: String,
    pub name: String,
    pub arguments: String,
    pub reason: String,
    pub sensitive: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DonePayload {
    pub conversation_id: String,
    pub status: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorPayload {
    pub conversation_id: String,
    pub message: String,
}

pub async fn run(
    app: AppHandle,
    state: Arc<AppState>,
    conversation_id: String,
    cancel: Arc<AtomicBool>,
) {
    if let Err(err) = run_inner(&app, &state, &conversation_id, &cancel).await {
        let _ = app.emit(
            "agent://error",
            ErrorPayload {
                conversation_id: conversation_id.clone(),
                message: err.to_string(),
            },
        );
    }
    if let Ok(mut runs) = state.runs.lock() {
        runs.remove(&conversation_id);
    }
    let _ = app.emit(
        "agent://done",
        DonePayload {
            conversation_id,
            status: if cancel.load(Ordering::Relaxed) {
                "cancelled".into()
            } else {
                "ok".into()
            },
        },
    );
}

async fn run_inner(
    app: &AppHandle,
    state: &Arc<AppState>,
    conversation_id: &str,
    cancel: &Arc<AtomicBool>,
) -> AppResult<()> {
    let settings = {
        let db = state.db.lock().map_err(|_| "db bloqueada")?;
        store::load_settings(&db)?
    };
    let api_key = crate::secrets::get_key(&settings.provider)?;

    for _ in 0..8 {
        if cancel.load(Ordering::Relaxed) {
            return Ok(());
        }

        let messages = {
            let db = state.db.lock().map_err(|_| "db bloqueada")?;
            store::load_messages(&db, conversation_id)?
        };

        let cid = conversation_id.to_string();
        let app_tokens = app.clone();
        let mut on_token = move |text: &str| {
            let _ = app_tokens.emit(
                "agent://token",
                TokenPayload {
                    conversation_id: cid.clone(),
                    text: text.to_string(),
                },
            );
        };

        let turn = crate::llm::complete(
            &state.http,
            &settings,
            api_key.as_deref(),
            &messages,
            &mut on_token,
        )
        .await?;

        let mut assistant = store::new_message(conversation_id, Role::Assistant, turn.text);
        if !turn.tool_calls.is_empty() {
            assistant.tool_calls = Some(turn.tool_calls.clone());
        }
        {
            let db = state.db.lock().map_err(|_| "db bloqueada")?;
            store::insert_message(&db, &assistant)?;
        }

        if turn.tool_calls.is_empty() {
            return Ok(());
        }

        for call in turn.tool_calls {
            if cancel.load(Ordering::Relaxed) {
                return Ok(());
            }
            let args: Value = serde_json::from_str(&call.arguments).unwrap_or(serde_json::json!({}));
            let (allowed, denied_reason) =
                decide(app, state, conversation_id, &call, &args, &settings).await?;

            let mut tool_msg = store::new_message(conversation_id, Role::Tool, String::new());
            tool_msg.tool_name = Some(call.name.clone());
            tool_msg.tool_call_id = Some(call.id.clone());

            if !allowed {
                tool_msg.content = denied_reason;
                let _ = app.emit(
                    "agent://tool_result",
                    ToolPayload {
                        conversation_id: conversation_id.to_string(),
                        id: call.id.clone(),
                        name: call.name.clone(),
                        arguments: call.arguments.clone(),
                        result: Some(tool_msg.content.clone()),
                        image_base64: None,
                        status: "denied".into(),
                    },
                );
            } else {
                let _ = app.emit(
                    "agent://tool_start",
                    ToolPayload {
                        conversation_id: conversation_id.to_string(),
                        id: call.id.clone(),
                        name: call.name.clone(),
                        arguments: call.arguments.clone(),
                        result: None,
                        image_base64: None,
                        status: "running".into(),
                    },
                );
                match tools::execute(&call.name, &args).await {
                    Ok(outcome) => {
                        tool_msg.content = outcome.text.clone();
                        tool_msg.image_base64 = outcome.image_base64.clone();
                        let _ = app.emit(
                            "agent://tool_result",
                            ToolPayload {
                                conversation_id: conversation_id.to_string(),
                                id: call.id.clone(),
                                name: call.name.clone(),
                                arguments: call.arguments.clone(),
                                result: Some(outcome.text),
                                image_base64: outcome.image_base64,
                                status: "done".into(),
                            },
                        );
                    }
                    Err(err) => {
                        tool_msg.content = format!("error: {err}");
                        let _ = app.emit(
                            "agent://tool_result",
                            ToolPayload {
                                conversation_id: conversation_id.to_string(),
                                id: call.id.clone(),
                                name: call.name.clone(),
                                arguments: call.arguments.clone(),
                                result: Some(tool_msg.content.clone()),
                                image_base64: None,
                                status: "error".into(),
                            },
                        );
                    }
                }
            }

            let db = state.db.lock().map_err(|_| "db bloqueada")?;
            store::insert_message(&db, &tool_msg)?;
        }
    }

    Ok(())
}

async fn decide(
    app: &AppHandle,
    state: &Arc<AppState>,
    conversation_id: &str,
    call: &ToolCall,
    args: &Value,
    settings: &Settings,
) -> AppResult<(bool, String)> {
    match policy::assess(&call.name, args, &settings.allowed_roots) {
        Risk::AutoAllow => Ok((true, String::new())),
        Risk::NeedsApproval { reason, sensitive } => {
            let (tx, rx) = tokio::sync::oneshot::channel();
            {
                let mut pending = state.pending.lock().await;
                pending.insert(call.id.clone(), tx);
            }
            let _ = app.emit(
                "agent://approval_required",
                ApprovalPayload {
                    conversation_id: conversation_id.to_string(),
                    request_id: call.id.clone(),
                    name: call.name.clone(),
                    arguments: call.arguments.clone(),
                    reason,
                    sensitive,
                },
            );
            let allowed = match tokio::time::timeout(Duration::from_secs(300), rx).await {
                Ok(Ok(value)) => value,
                _ => false,
            };
            Ok((
                allowed,
                "El usuario denegó esta acción.".into(),
            ))
        }
    }
}
