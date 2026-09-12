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

  useEffect(() => {
    endRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages, streaming]);

  function submit() {
    const text = draft.trim();
    if (!text || streaming) return;
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
            <ul>
              <li>Lista los archivos de mi home</li>
              <li>Ejecuta `uname -a`</li>
              <li>¿Qué procesos consumen más CPU?</li>
            </ul>
          </div>
        ) : (
          messages.map((message) => <MessageBubble key={message.id} message={message} />)
        )}
        <div ref={endRef} />
      </div>
      {error ? <p className="error-bar">{error}</p> : null}
      <form
        className="composer"
        onSubmit={(e) => {
          e.preventDefault();
          submit();
        }}
      >
        <textarea
          value={draft}
          placeholder="Pregunta o pide una acción en tu sistema…"
          rows={2}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              submit();
            }
          }}
        />
        {streaming ? (
          <button type="button" className="ghost" onClick={onCancel}>
            Detener
          </button>
        ) : (
          <button type="submit" className="primary" disabled={!draft.trim()}>
            Enviar
          </button>
        )}
      </form>
    </section>
  );
}
