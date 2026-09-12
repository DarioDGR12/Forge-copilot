import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { ApprovalModal } from "./components/ApprovalModal";
import { Chat } from "./components/Chat";
import { SettingsPanel } from "./components/SettingsPanel";
import { Sidebar } from "./components/Sidebar";
import { Titlebar } from "./components/Titlebar";
import { api, isTauri, listen } from "./lib/bridge";
import type { ApprovalRequest, ChatMessage, Conversation, Settings } from "./types";
import "./App.css";

export default function App() {
  const [view, setView] = useState<"chat" | "settings">("chat");
  const [conversations, setConversations] = useState<Conversation[]>([]);
  const [activeId, setActiveId] = useState<string | null>(null);
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [streaming, setStreaming] = useState(false);
  const [streamText, setStreamText] = useState("");
  const [liveTools, setLiveTools] = useState<ChatMessage[]>([]);
  const [approval, setApproval] = useState<ApprovalRequest | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [settings, setSettings] = useState<Settings | null>(null);
  const activeRef = useRef<string | null>(null);
  const sendingRef = useRef(false);
  const loadGen = useRef(0);

  const refreshConversations = useCallback(async () => {
    const list = await api.listConversations();
    setConversations(list);
    return list;
  }, []);

  const loadMessages = useCallback(async (id: string) => {
    const gen = ++loadGen.current;
    const list = await api.getMessages(id);
    if (gen !== loadGen.current) return;
    setMessages(list);
    setLiveTools([]);
    setStreamText("");
  }, []);

  useEffect(() => {
    void refreshConversations().catch((err: unknown) =>
      setError(err instanceof Error ? err.message : String(err)),
    );
    void api.getSettings().then(setSettings).catch(() => undefined);
  }, [refreshConversations]);

  useEffect(() => {
    const matches = (conversationId: string) => {
      const current = activeRef.current;
      return current === null || current === conversationId;
    };
    const offs: Array<Promise<() => void>> = [];

    offs.push(
      listen<{ conversationId: string; text: string }>("agent://token", (payload) => {
        if (!matches(payload.conversationId)) return;
        setStreamText((prev) => prev + payload.text);
      }),
    );
    offs.push(
      listen<{
        conversationId: string;
        id: string;
        name: string;
        arguments: string;
        result?: string | null;
        imageBase64?: string | null;
        status: string;
      }>("agent://tool_start", (payload) => {
        if (!matches(payload.conversationId)) return;
        setLiveTools((prev) => [
          ...prev.filter((m) => m.toolCallId !== payload.id),
          {
            id: payload.id,
            conversationId: payload.conversationId,
            role: "tool",
            content: "",
            toolName: payload.name,
            toolCallId: payload.id,
            createdAt: Date.now() / 1000,
            status: payload.status,
          },
        ]);
      }),
    );
    offs.push(
      listen<{
        conversationId: string;
        id: string;
        name: string;
        arguments: string;
        result?: string | null;
        imageBase64?: string | null;
        status: string;
      }>("agent://tool_result", (payload) => {
        if (!matches(payload.conversationId)) return;
        setLiveTools((prev) => {
          const next = prev.filter((m) => m.toolCallId !== payload.id);
          next.push({
            id: payload.id,
            conversationId: payload.conversationId,
            role: "tool",
            content: payload.result ?? "",
            toolName: payload.name,
            toolCallId: payload.id,
            createdAt: Date.now() / 1000,
            status: payload.status,
            imageBase64: payload.imageBase64,
          });
          return next;
        });
      }),
    );
    offs.push(
      listen<ApprovalRequest>("agent://approval_required", (payload) => {
        if (!matches(payload.conversationId)) return;
        setApproval(payload);
      }),
    );
    offs.push(
      listen<{ conversationId: string }>("agent://done", (payload) => {
        if (!matches(payload.conversationId)) return;
        activeRef.current = payload.conversationId;
        setActiveId(payload.conversationId);
        sendingRef.current = false;
        setStreaming(false);
        setApproval(null);
        void loadMessages(payload.conversationId);
        void refreshConversations();
      }),
    );
    offs.push(
      listen<{ conversationId: string; message: string }>("agent://error", (payload) => {
        if (!matches(payload.conversationId)) return;
        setError(payload.message);
      }),
    );

    return () => {
      void Promise.all(offs.map((off) => Promise.resolve(off))).then((fns) =>
        fns.forEach((fn) => fn()),
      );
    };
  }, [loadMessages, refreshConversations]);

  const visibleMessages = useMemo(() => {
    const extra: ChatMessage[] = [];
    if (streamText) {
      extra.push({
        id: "stream",
        conversationId: activeId ?? "",
        role: "assistant",
        content: streamText,
        createdAt: Date.now() / 1000,
      });
    }
    const persistedIds = new Set(
      messages.filter((m) => m.role === "tool" && m.toolCallId).map((m) => m.toolCallId),
    );
    const pendingTools = liveTools.filter((m) => !persistedIds.has(m.toolCallId));
    return [...messages, ...pendingTools, ...extra];
  }, [messages, liveTools, streamText, activeId]);

  async function handleSend(text: string) {
    if (sendingRef.current) return;
    sendingRef.current = true;
    loadGen.current += 1;
    setError(null);
    setStreaming(true);
    setStreamText("");
    setLiveTools([]);
    const localId = `local-${Date.now()}`;
    setMessages((prev) => {
      const last = prev[prev.length - 1];
      if (last?.role === "user" && last.content === text) {
        return prev;
      }
      return [
        ...prev,
        {
          id: localId,
          conversationId: activeId ?? "pending",
          role: "user",
          content: text,
          createdAt: Date.now() / 1000,
        },
      ];
    });
    try {
      const conv = await api.sendMessage(activeId, text);
      activeRef.current = conv.id;
      setActiveId(conv.id);
      await loadMessages(conv.id);
      await refreshConversations();
    } catch (err) {
      sendingRef.current = false;
      setStreaming(false);
      setError(err instanceof Error ? err.message : String(err));
    }
  }

  async function handleNew() {
    activeRef.current = null;
    setActiveId(null);
    setMessages([]);
    setLiveTools([]);
    setStreamText("");
    setView("chat");
  }

  async function handleSelect(id: string) {
    activeRef.current = id;
    setActiveId(id);
    setView("chat");
    await loadMessages(id);
  }

  async function handleDelete(id: string) {
    await api.deleteConversation(id);
    if (activeId === id) {
      activeRef.current = null;
      setActiveId(null);
      setMessages([]);
    }
    await refreshConversations();
  }

  return (
    <div className="shell">
      <Titlebar view={view} onView={setView} onNew={() => void handleNew()} />
      <div className="body">
        <Sidebar
          items={conversations}
          activeId={activeId}
          onSelect={(id) => void handleSelect(id)}
          onNew={() => void handleNew()}
          onDelete={(id) => void handleDelete(id)}
        />
        {view === "settings" ? (
          <SettingsPanel onSaved={setSettings} />
        ) : (
          <Chat
            messages={visibleMessages}
            streaming={streaming}
            error={error}
            onSend={(text) => void handleSend(text)}
            onCancel={() => {
              if (activeId) void api.cancelRun(activeId);
              setStreaming(false);
            }}
          />
        )}
      </div>
      <footer className="status-bar" data-testid="status-bar">
        <span data-testid="status-provider">{settings?.provider ?? "demo"}</span>
        <span aria-hidden>·</span>
        <span data-testid="status-model">{settings?.model ?? "forge-demo"}</span>
        <span aria-hidden>·</span>
        <kbd>{settings?.shortcut ?? "ctrl+shift+space"}</kbd>
        {!isTauri() ? <span className="status-note">navegador · tools simuladas</span> : null}
      </footer>
      {approval ? (
        <ApprovalModal
          request={approval}
          onResolve={(allowed) => {
            void api.resolveApproval(approval.requestId, allowed);
            setApproval(null);
          }}
        />
      ) : null}
    </div>
  );
}
