import { useEffect, useRef, useState } from "react";
import type { ChatMessage } from "../types";
import { MessageBubble } from "./MessageBubble";

type ChatProps = {
  messages: ChatMessage[];
  streaming: boolean;
  error: string | null;
  onSend: (text: string) => void;
  onRetry: (text: string) => void;
  onCancel: () => void;
};

const MAX_ATTACH = 32 * 1024;

export function Chat({ messages, streaming, error, onSend, onRetry, onCancel }: ChatProps) {
  const [draft, setDraft] = useState("");
  const [attachName, setAttachName] = useState<string | null>(null);
  const [attachText, setAttachText] = useState<string | null>(null);
  const [attachError, setAttachError] = useState<string | null>(null);
  const [dragging, setDragging] = useState(false);
  const endRef = useRef<HTMLDivElement | null>(null);
  const fileRef = useRef<HTMLInputElement | null>(null);
  const locked = useRef(false);

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, streaming]);

  useEffect(() => {
    if (!streaming) locked.current = false;
  }, [streaming]);

  function compose(userText: string): string | null {
    const body = userText.trim();
    if (attachText && attachName) {
      const block = `Adjunto «${attachName}»:\n\`\`\`\n${attachText}\n\`\`\``;
      return body ? `${block}\n\n${body}` : block;
    }
    return body || null;
  }

  function submit() {
    const text = compose(draft);
    if (!text || streaming || locked.current) return;
    locked.current = true;
    onSend(text);
    setDraft("");
    setAttachName(null);
    setAttachText(null);
    setAttachError(null);
    if (fileRef.current) fileRef.current.value = "";
  }

  function onPickFile(file: File | undefined) {
    setAttachError(null);
    if (!file) return;
    if (file.size > MAX_ATTACH) {
      setAttachError(`El adjunto supera ${MAX_ATTACH / 1024} KB. Elige un archivo de texto más corto.`);
      return;
    }
    const reader = new FileReader();
    reader.onload = () => {
      const result = typeof reader.result === "string" ? reader.result : "";
      setAttachName(file.name);
      setAttachText(result);
    };
    reader.onerror = () => {
      setAttachError("No pude leer el archivo.");
    };
    reader.readAsText(file);
  }

  const canSend = Boolean(draft.trim() || attachText);
  const lastUserId = [...messages].reverse().find((m) => m.role === "user")?.id;

  function sendPreset(text: string) {
    if (streaming || locked.current) return;
    locked.current = true;
    onSend(text);
  }

  return (
    <section
      className={dragging ? "chat drag-over" : "chat"}
      data-testid="chat-drop"
      onDragEnter={(e) => {
        e.preventDefault();
        setDragging(true);
      }}
      onDragOver={(e) => {
        e.preventDefault();
        setDragging(true);
      }}
      onDragLeave={(e) => {
        if (e.currentTarget.contains(e.relatedTarget as Node | null)) return;
        setDragging(false);
      }}
      onDrop={(e) => {
        e.preventDefault();
        setDragging(false);
        onPickFile(e.dataTransfer.files?.[0]);
      }}
    >
      <div className="transcript">
        {messages.length === 0 ? (
          <div className="hero">
            <h1>Hola. Soy Forge.</h1>
            <p>
              Un Copilot para Pop!_OS: chat, terminal, archivos y tu escritorio, con tu propia
              clave.
            </p>
            <div className="chips">
              {(
                [
                  ["chip-host", "Qué sistema tengo"],
                  ["chip-files", "Lista los archivos de mi home"],
                  ["chip-clipboard", "Qué hay en el portapapeles"],
                  ["chip-windows", "Lista las ventanas"],
                  ["chip-screen", "Qué hay en pantalla"],
                  ["chip-type", "Teclea hola"],
                  ["chip-docs", "Abre Documentos"],
                  ["chip-uname", "Ejecuta `uname -a`"],
                ] as const
              ).map(([id, chip]) => (
                <button
                  key={id}
                  type="button"
                  className="chip"
                  data-testid={id}
                  disabled={streaming}
                  onClick={() => sendPreset(chip)}
                >
                  {chip}
                </button>
              ))}
            </div>
          </div>
        ) : (
          messages.map((message) => (
            <MessageBubble
              key={message.id}
              message={message}
              onRetry={
                !streaming && message.role === "user" && message.id === lastUserId
                  ? () => onRetry(message.content)
                  : undefined
              }
            />
          ))
        )}
        <div ref={endRef} />
      </div>
      {error ? <p className="error-bar">{error}</p> : null}
      {attachError ? <p className="error-bar">{attachError}</p> : null}
      <div className="composer">
        <input
          ref={fileRef}
          type="file"
          accept="text/*,.md,.txt,.json,.rs,.ts,.tsx,.js,.py,.toml,.yml,.yaml"
          hidden
          data-testid="attach-input"
          onChange={(e) => onPickFile(e.target.files?.[0])}
        />
        <div className="composer-main">
          {attachName ? (
            <div className="attach-chip" data-testid="attach-chip">
              <span>{attachName}</span>
              <button
                type="button"
                className="ghost"
                aria-label="Quitar adjunto"
                onClick={() => {
                  setAttachName(null);
                  setAttachText(null);
                  if (fileRef.current) fileRef.current.value = "";
                }}
              >
                ×
              </button>
            </div>
          ) : null}
          <textarea
            data-testid="composer"
            value={draft}
            placeholder="Pregunta, pide una acción o adjunta un archivo de texto…"
            rows={2}
            onChange={(e) => {
              if (!streaming) locked.current = false;
              setDraft(e.target.value);
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                e.stopPropagation();
                submit();
              }
            }}
          />
        </div>
        <div className="composer-actions">
          <button
            type="button"
            className="ghost"
            data-testid="attach"
            disabled={streaming}
            onClick={() => fileRef.current?.click()}
          >
            Adjuntar
          </button>
          <button
            type="button"
            className="ghost"
            data-testid="screenshot"
            disabled={streaming}
            onClick={() => sendPreset("captura la pantalla")}
          >
            Captura
          </button>
          {streaming ? (
            <button type="button" className="ghost" data-testid="cancel-send" onClick={onCancel}>
              Detener
            </button>
          ) : (
            <button
              type="button"
              className="primary"
              data-testid="send"
              disabled={!canSend}
              onClick={submit}
            >
              Enviar
            </button>
          )}
        </div>
      </div>
    </section>
  );
}
