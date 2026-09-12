import type {
  ApprovalRequest,
  ChatMessage,
  Conversation,
  KeyStatus,
  Settings,
  TokenPayload,
  ToolPayload,
} from "../types";

type Handler = (payload: unknown) => void;

const listeners = new Map<string, Set<Handler>>();
const conversations = new Map<string, Conversation>();
const messages = new Map<string, ChatMessage[]>();
const keys = new Map<string, string>();
const pendingApprovals = new Map<string, (allowed: boolean) => void>();

let settings: Settings = {
  provider: "demo",
  model: "forge-demo",
  baseUrl: "",
  allowedRoots: ["/home"],
  shortcut: "ctrl+shift+space",
};

function uid(): string {
  return crypto.randomUUID();
}

function now(): number {
  return Math.floor(Date.now() / 1000);
}

function emit(event: string, payload: unknown) {
  const set = listeners.get(event);
  if (!set) return;
  for (const handler of set) handler(payload);
}

function titleFrom(text: string): string {
  const compact = text.trim().replace(/\s+/g, " ");
  return compact.length <= 42 ? compact || "Nueva conversación" : `${compact.slice(0, 41)}…`;
}

function inferTool(text: string): { name: string; args: Record<string, unknown> } | null {
  const t = text.toLowerCase();
  if (t.includes("screenshot") || t.includes("captura") || t.includes("pantalla")) {
    return { name: "screenshot", args: {} };
  }
  if (t.includes("sistema") || t.includes("distro") || t.includes("host_info")) {
    return { name: "host_info", args: {} };
  }
  if (t.includes("portapapeles") || t.includes("clipboard") || t.includes("wl-paste")) {
    if (
      t.includes("copia ") ||
      t.includes("copiar ") ||
      t.includes("pon ") ||
      t.includes("escribe ")
    ) {
      const quoted = text.match(/["«]([^"»]+)["»]/);
      return { name: "clipboard_write", args: { text: quoted?.[1] ?? "Forge Copilot" } };
    }
    return { name: "clipboard_read", args: {} };
  }
  if (t.includes("enfoca") || t.includes("enfocar") || t.includes("focus window")) {
    const query = text.replace(/^(enfoca|enfocar|focus)\s+/i, "").trim() || "Firefox";
    return { name: "focus_window", args: { query } };
  }
  if (t.includes("ventana") || t.includes("wmctrl")) {
    return { name: "list_windows", args: {} };
  }
  if (t.includes("notifica") || t.includes("notify-send") || t.includes("avísame") || t.includes("avisame")) {
    const quoted = text.match(/["«]([^"»]+)["»]/);
    return { name: "notify", args: { title: "Forge Copilot", body: quoted?.[1] ?? "Hola desde Forge" } };
  }
  if (t.includes("teclea") || t.includes("tipea") || t.includes("en el teclado")) {
    const quoted = text.match(/["«]([^"»]+)["»]/);
    const typed = quoted?.[1] ?? (text.replace(/^(teclea|tipea|type)\s+/i, "").trim() || "hola");
    return { name: "type_text", args: { text: typed } };
  }
  if (t.includes("pulsa ") || t.includes("presiona ") || t.includes("press ")) {
    const keys = text.replace(/^(pulsa|presiona|press)\s+/i, "").trim() || "Return";
    const mapped = /^(enter|intro|return)$/i.test(keys) ? "Return" : keys;
    return { name: "press_keys", args: { keys: mapped } };
  }
  if (t.includes("haz clic") || t.includes("clic en") || t.includes("mouse_click")) {
    const nums = text.match(/\d+/g);
    const args: Record<string, unknown> = { button: "left" };
    if (nums && nums.length >= 2) {
      args.x = Number(nums[0]);
      args.y = Number(nums[1]);
    }
    return { name: "mouse_click", args };
  }
  if (t.includes("puntero") || t.includes("ratón") || t.includes("raton")) {
    return { name: "pointer_info", args: {} };
  }
  if (t.includes("abre ") || t.includes("abrir ") || t.includes("lanza ")) {
    if (t.includes("documento") || t.includes("home") || t.includes("carpeta")) {
      return { name: "open_path", args: { path: "~/Documents" } };
    }
    const name = text.replace(/^(abre|abrir|lanza|lanzar)\s+/i, "").trim() || "Firefox";
    return { name: "launch_app", args: { name } };
  }
  if (t.includes("proceso") || t.includes("cpu")) {
    return { name: "list_processes", args: { limit: 8 } };
  }
  if (t.includes("aplicación") || t.includes("aplicacion") || t.includes("apps")) {
    return { name: "list_apps", args: {} };
  }
  if (t.includes("ejecuta") || t.includes("terminal") || t.includes("comando") || t.includes("run ")) {
    const tick = text.match(/`([^`]+)`/);
    return { name: "run_terminal", args: { command: tick?.[1] ?? "uname -a" } };
  }
  if (
    (t.includes("lista") || t.includes("listar")) &&
    (t.includes("archivo") || t.includes("home") || t.includes("carpeta"))
  ) {
    return { name: "list_dir", args: { path: "~" } };
  }
  return null;
}

function mockToolResult(name: string, args: Record<string, unknown>): string {
  switch (name) {
    case "run_terminal":
      return `exit=0\n--- stdout ---\nLinux pop-os 6.12.0-76061200-generic\n--- stderr ---\n`;
    case "list_dir":
      return `${args.path ?? "~"} (4 entradas)\ndir      4096 Desktop\ndir      4096 Documents\nfile      220 .bashrc\nfile     1024 notes.txt`;
    case "list_apps":
      return "4 aplicaciones\nFirefox — firefox %u\nCOSMO Files — cosmic-files\nTerminal — gnome-terminal\nForge Copilot — forge-copilot";
    case "list_processes":
      return "CPU%   PID     MEM(MB)  NOMBRE\n 12.4  1422      480.1  firefox\n  8.1  2201      210.4  cosmic-comp\n  3.0  884       90.2  forge-copilot";
    case "screenshot":
      return "Captura guardada en /tmp/forge-copilot-demo.png";
    case "host_info":
      return "usuario=dario\nhostname=pop-os\nos=Pop!_OS 24.04 LTS\ndesktop=COSMIC\nhome=/home/dario";
    case "launch_app":
      return `lanzada ${String(args.name ?? "Firefox")} (demo)`;
    case "open_path":
      return `abierto ${String(args.path ?? "~/Documents")}`;
    case "clipboard_read":
      return "hola desde el portapapeles (demo)";
    case "clipboard_write":
      return `copiados ${String(args.text ?? "").length} caracteres al portapapeles`;
    case "notify":
      return `notificación: ${String(args.title ?? "Forge Copilot")}`;
    case "list_windows":
      return "2 ventanas\n0x03c00007  Firefox\n0x02a00001  Forge Copilot";
    case "focus_window":
      return `enfocada: ${String(args.query ?? "Firefox")}`;
    case "type_text":
      return `tecleados ${String(args.text ?? "").length} caracteres (demo)`;
    case "press_keys":
      return `pulsado ${String(args.keys ?? "Return")} (demo)`;
    case "mouse_click":
      return `clic ${String(args.button ?? "left")} (demo)`;
    case "pointer_info":
      return "X=120\nY=340\nSCREEN=0";
    default:
      return "ok";
  }
}

function needsApproval(name: string): boolean {
  return (
    name === "run_terminal" ||
    name === "write_file" ||
    name === "launch_app" ||
    name === "clipboard_write" ||
    name === "focus_window" ||
    name === "type_text" ||
    name === "press_keys" ||
    name === "mouse_click"
  );
}

function approvalMeta(
  name: string,
  args: Record<string, unknown>,
): { reason: string; sensitive: boolean } {
  switch (name) {
    case "type_text":
      return {
        reason: `Teclear en la ventana activa: «${String(args.text ?? "")}»`,
        sensitive: true,
      };
    case "press_keys":
      return { reason: `Pulsar teclas: ${String(args.keys ?? "")}`, sensitive: true };
    case "mouse_click":
      return {
        reason: `Clic ${String(args.button ?? "left")}`,
        sensitive: true,
      };
    case "clipboard_write":
      return { reason: "Escribir al portapapeles", sensitive: false };
    case "focus_window":
      return { reason: `Enfocar ventana: ${String(args.query ?? "")}`, sensitive: false };
    case "launch_app":
      return { reason: `Lanzar aplicación: ${String(args.name ?? "")}`, sensitive: false };
    default:
      return {
        reason: `Comando: ${String(args.command ?? name)}`,
        sensitive: false,
      };
  }
}

async function streamText(conversationId: string, text: string) {
  for (let i = 0; i < text.length; i += 4) {
    emit("agent://token", {
      conversationId,
      text: text.slice(i, i + 4),
    } satisfies TokenPayload);
    await new Promise((r) => setTimeout(r, 12));
  }
}

async function runDemoAgent(conversationId: string) {
  const hist = messages.get(conversationId) ?? [];
  const lastUserIdx = hist.map((m) => m.role).lastIndexOf("user");
  const lastToolIdx = hist.map((m) => m.role).lastIndexOf("tool");
  const lastUser = lastUserIdx >= 0 ? hist[lastUserIdx] : undefined;
  const lastTool = lastToolIdx >= 0 ? hist[lastToolIdx] : undefined;

  try {
    if (lastTool && lastToolIdx > lastUserIdx) {
      const reply = `Listo. Esto es lo que devolvió **${lastTool.toolName ?? "la herramienta"}**:\n\n\`\`\`\n${lastTool.content}\n\`\`\``;
      await streamText(conversationId, reply);
      pushMessage(conversationId, {
        role: "assistant",
        content: reply,
      });
      emit("agent://done", { conversationId, status: "ok" });
      return;
    }

    const tool = lastUser ? inferTool(lastUser.content) : null;
    if (tool) {
      const intro = `Voy a usar \`${tool.name}\` para responderte con datos del sistema.`;
      await streamText(conversationId, intro);
      const callId = uid();
      pushMessage(conversationId, {
        role: "assistant",
        content: intro,
        toolCalls: [{ id: callId, name: tool.name, arguments: JSON.stringify(tool.args) }],
      });

      let allowed = true;
      if (needsApproval(tool.name)) {
        allowed = await new Promise<boolean>((resolve) => {
          pendingApprovals.set(callId, resolve);
          emit("agent://approval_required", {
            conversationId,
            requestId: callId,
            name: tool.name,
            arguments: JSON.stringify(tool.args, null, 2),
            ...approvalMeta(tool.name, tool.args),
          } satisfies ApprovalRequest);
        });
      }

      if (!allowed) {
        pushMessage(conversationId, {
          role: "tool",
          content: "El usuario denegó esta acción.",
          toolName: tool.name,
          toolCallId: callId,
          status: "denied",
        });
        emit("agent://tool_result", {
          conversationId,
          id: callId,
          name: tool.name,
          arguments: JSON.stringify(tool.args),
          result: "El usuario denegó esta acción.",
          status: "denied",
        } satisfies ToolPayload);
      } else {
        emit("agent://tool_start", {
          conversationId,
          id: callId,
          name: tool.name,
          arguments: JSON.stringify(tool.args),
          status: "running",
        } satisfies ToolPayload);
        await new Promise((r) => setTimeout(r, 350));
        const result = mockToolResult(tool.name, tool.args);
        pushMessage(conversationId, {
          role: "tool",
          content: result,
          toolName: tool.name,
          toolCallId: callId,
          status: "done",
        });
        emit("agent://tool_result", {
          conversationId,
          id: callId,
          name: tool.name,
          arguments: JSON.stringify(tool.args),
          result,
          status: "done",
        } satisfies ToolPayload);
      }

      const follow = [...(messages.get(conversationId) ?? [])]
        .reverse()
        .find((m) => m.role === "tool");
      if (follow) {
        const reply = `Listo. Resultado de **${follow.toolName}**:\n\n\`\`\`\n${follow.content}\n\`\`\``;
        await streamText(conversationId, reply);
        pushMessage(conversationId, { role: "assistant", content: reply });
      }
      emit("agent://done", { conversationId, status: "ok" });
      return;
    }

    const help =
      "Soy Forge Copilot. En el navegador estoy en modo demostración. En Pop!_OS, la app Tauri usa tu terminal, archivos, apps, portapapeles y ventanas de verdad.\n\nPrueba: «qué hay en el portapapeles», «lista las ventanas» o «ejecuta `uname -a`».";
    await streamText(conversationId, help);
    pushMessage(conversationId, { role: "assistant", content: help });
    emit("agent://done", { conversationId, status: "ok" });
  } catch (err) {
    emit("agent://error", {
      conversationId,
      message: err instanceof Error ? err.message : String(err),
    });
    emit("agent://done", { conversationId, status: "error" });
  }
}

function pushMessage(
  conversationId: string,
  partial: Omit<ChatMessage, "id" | "conversationId" | "createdAt">,
): ChatMessage {
  const msg: ChatMessage = {
    id: uid(),
    conversationId,
    createdAt: now(),
    ...partial,
  };
  const list = messages.get(conversationId) ?? [];
  list.push(msg);
  messages.set(conversationId, list);
  const conv = conversations.get(conversationId);
  if (conv) {
    conv.updatedAt = now();
    conversations.set(conversationId, conv);
  }
  return msg;
}

function keyStatus(provider: string): KeyStatus {
  const key = keys.get(provider);
  return {
    provider,
    configured: Boolean(key),
    hint: key ? `…${key.slice(-4)}` : null,
  };
}

export async function invoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  switch (cmd) {
    case "list_conversations":
      return [...conversations.values()].sort((a, b) => b.updatedAt - a.updatedAt) as T;
    case "get_messages":
      return (messages.get(String(args?.conversationId)) ?? []) as T;
    case "create_conversation": {
      const conv: Conversation = {
        id: uid(),
        title: "Nueva conversación",
        createdAt: now(),
        updatedAt: now(),
      };
      conversations.set(conv.id, conv);
      messages.set(conv.id, []);
      return conv as T;
    }
    case "delete_conversation": {
      const id = String(args?.id ?? "");
      conversations.delete(id);
      messages.delete(id);
      return undefined as T;
    }
    case "send_message": {
      const content = String(args?.content ?? "").trim();
      if (!content) throw new Error("escribe un mensaje");
      let id = args?.conversationId ? String(args.conversationId) : "";
      let conv = id ? conversations.get(id) : undefined;
      if (!conv) {
        conv = {
          id: uid(),
          title: titleFrom(content),
          createdAt: now(),
          updatedAt: now(),
        };
        conversations.set(conv.id, conv);
        messages.set(conv.id, []);
      }
      const existing = messages.get(conv.id) ?? [];
      const last = existing[existing.length - 1];
      if (!(last?.role === "user" && last.content === content)) {
        pushMessage(conv.id, { role: "user", content });
        void runDemoAgent(conv.id);
      }
      return conv as T;
    }
    case "cancel_run":
      return undefined as T;
    case "resolve_approval": {
      const requestId = String(args?.requestId ?? "");
      const allowed = Boolean(args?.allowed);
      pendingApprovals.get(requestId)?.(allowed);
      pendingApprovals.delete(requestId);
      return undefined as T;
    }
    case "get_settings":
      return { ...settings, allowedRoots: [...settings.allowedRoots] } as T;
    case "save_settings":
      settings = { ...(args?.settings as Settings) };
      return settings as T;
    case "set_api_key": {
      const provider = String(args?.provider ?? "demo");
      const key = String(args?.key ?? "").trim();
      if (key) keys.set(provider, key);
      else keys.delete(provider);
      return keyStatus(provider) as T;
    }
    case "clear_api_key": {
      const provider = String(args?.provider ?? "demo");
      keys.delete(provider);
      return keyStatus(provider) as T;
    }
    case "key_status":
      return keyStatus(String(args?.provider ?? "demo")) as T;
    case "test_connection":
      if (settings.provider === "demo") {
        return "Modo demostración listo (no requiere clave)" as T;
      }
      if (!keys.get(String(settings.provider))) {
        throw new Error("falta la API key");
      }
      return "Clave guardada. La prueba real ocurre en la app Tauri." as T;
    case "window_hide":
    case "window_toggle":
      return undefined as T;
    default:
      throw new Error(`comando mock desconocido: ${cmd}`);
  }
}

export function listen<T>(event: string, handler: (payload: T) => void): () => void {
  const set = listeners.get(event) ?? new Set();
  const wrapped: Handler = (payload) => handler(payload as T);
  set.add(wrapped);
  listeners.set(event, set);
  return () => set.delete(wrapped);
}
