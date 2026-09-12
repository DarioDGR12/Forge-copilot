import type {
  ApprovalRequest,
  ChatMessage,
  Conversation,
  DonePayload,
  ErrorPayload,
  KeyStatus,
  Settings,
  TokenPayload,
  ToolPayload,
} from "../types";
import * as mock from "./mock";

export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    return mock.invoke<T>(cmd, args);
  }
  const { invoke: tauriInvoke } = await import("@tauri-apps/api/core");
  return tauriInvoke<T>(cmd, args);
}

export async function listen<T>(
  event: string,
  handler: (payload: T) => void,
): Promise<() => void> {
  if (!isTauri()) {
    return mock.listen<T>(event, handler);
  }
  const { listen: tauriListen } = await import("@tauri-apps/api/event");
  const unlisten = await tauriListen<T>(event, (e) => handler(e.payload));
  return unlisten;
}

export const api = {
  listConversations: () => invoke<Conversation[]>("list_conversations"),
  getMessages: (conversationId: string) =>
    invoke<ChatMessage[]>("get_messages", { conversationId }),
  createConversation: () => invoke<Conversation>("create_conversation"),
  deleteConversation: (id: string) => invoke<void>("delete_conversation", { id }),
  sendMessage: (conversationId: string | null, content: string) =>
    invoke<Conversation>("send_message", { conversationId, content }),
  cancelRun: (conversationId: string) => invoke<void>("cancel_run", { conversationId }),
  resolveApproval: (requestId: string, allowed: boolean) =>
    invoke<void>("resolve_approval", { requestId, allowed }),
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) => invoke<Settings>("save_settings", { settings }),
  setApiKey: (provider: string, key: string) =>
    invoke<KeyStatus>("set_api_key", { provider, key }),
  clearApiKey: (provider: string) => invoke<KeyStatus>("clear_api_key", { provider }),
  keyStatus: (provider: string) => invoke<KeyStatus>("key_status", { provider }),
  testConnection: () => invoke<string>("test_connection"),
  windowHide: () => invoke<void>("window_hide"),
  windowToggle: () => invoke<void>("window_toggle"),
};

export type EventMap = {
  "agent://token": TokenPayload;
  "agent://tool_start": ToolPayload;
  "agent://tool_result": ToolPayload;
  "agent://approval_required": ApprovalRequest;
  "agent://done": DonePayload;
  "agent://error": ErrorPayload;
};
