export type Provider = "demo" | "openai" | "anthropic" | "openrouter" | "ollama";

export interface Conversation {
  id: string;
  title: string;
  createdAt: number;
  updatedAt: number;
}

export interface ToolCallInfo {
  id: string;
  name: string;
  arguments: string;
}

export interface ChatMessage {
  id: string;
  conversationId: string;
  role: "user" | "assistant" | "tool" | "system";
  content: string;
  toolName?: string | null;
  toolCallId?: string | null;
  toolCalls?: ToolCallInfo[] | null;
  createdAt: number;
  status?: string | null;
  imageBase64?: string | null;
}

export interface Settings {
  provider: Provider | string;
  model: string;
  baseUrl: string;
  allowedRoots: string[];
  shortcut: string;
}

export interface KeyStatus {
  provider: string;
  configured: boolean;
  hint?: string | null;
}

export interface ApprovalRequest {
  conversationId: string;
  requestId: string;
  name: string;
  arguments: string;
  reason: string;
  sensitive: boolean;
}

export interface TokenPayload {
  conversationId: string;
  text: string;
}

export interface ToolPayload {
  conversationId: string;
  id: string;
  name: string;
  arguments: string;
  result?: string | null;
  imageBase64?: string | null;
  status: string;
}

export interface DonePayload {
  conversationId: string;
  status: string;
}

export interface ErrorPayload {
  conversationId: string;
  message: string;
}

export interface DiagnosticItem {
  name: string;
  ok: boolean;
  detail: string;
}

export interface Diagnostics {
  host: string;
  items: DiagnosticItem[];
}
