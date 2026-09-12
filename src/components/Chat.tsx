import { useEffect, useRef, useState } from "react";
import type { ChatMessage } from "../types";
import { MessageBubble } from "./MessageBubble";

type ChatProps = {
  messages: ChatMessage[];
  streaming: boolean;
  error: string | null;
  onSend: (text: string) => void;
  onCancel: () => void;
};

export function Chat({ messages, streaming, error, onSend, onCancel }: ChatProps) {
  const [draft, setDraft] = useState("");
  const endRef = useRef<HTMLDivElement | null>(null);
  const locked = useRef(false);

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, streaming]);

  useEffect(() => {
    if (!streaming) locked.current = false;
  }, [streaming]);

  function submit() {
    const text = draft.trim();
    if (!text || streaming || locked.current) return;
    locked.current = true;
    onSend(text);
    setDraft("");
  }

  return (
    <section className="chat">
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
                  onClick={() => {
                    if (streaming || locked.current) return;
                    locked.current = true;
                    onSend(chip);
                  }}
                >
                  {chip}
                </button>
              ))}
            </div>
          </div>
        ) : (
          messages.map((message) => <MessageBubble key={message.id} message={message} />)
        )}
        <div ref={endRef} />
      </div>
      {error ? <p className="error-bar">{error}</p> : null}
      <div className="composer">
        <textarea
          data-testid="composer"
          value={draft}
          placeholder="Pregunta o pide una acción en tu sistema…"
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
        {streaming ? (
          <button type="button" className="ghost" data-testid="cancel-send" onClick={onCancel}>
            Detener
          </button>
        ) : (
          <button
            type="button"
            className="primary"
            data-testid="send"
            disabled={!draft.trim()}
            onClick={submit}
          >
            Enviar
          </button>
        )}
      </div>
    </section>
  );
}
