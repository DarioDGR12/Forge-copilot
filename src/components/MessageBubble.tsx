import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";
import type { ChatMessage } from "../types";
import { ToolCard } from "./ToolCard";

type MessageBubbleProps = {
  message: ChatMessage;
  onRetry?: () => void;
};

export function MessageBubble({ message, onRetry }: MessageBubbleProps) {
  if (message.role === "tool") {
    return <ToolCard message={message} />;
  }
  const mine = message.role === "user";
  return (
    <article className={mine ? "bubble user" : "bubble assistant"}>
      <span className="who">{mine ? "Tú" : "Forge"}</span>
      <div className="md">
        <Markdown remarkPlugins={[remarkGfm]}>{message.content || " "}</Markdown>
      </div>
      <div className="bubble-actions">
        <button
          type="button"
          className="ghost"
          data-testid="copy-msg"
          onClick={() => void navigator.clipboard?.writeText(message.content).catch(() => undefined)}
        >
          Copiar
        </button>
        {onRetry ? (
          <button type="button" className="ghost" data-testid="retry-msg" onClick={onRetry}>
            Reintentar
          </button>
        ) : null}
      </div>
    </article>
  );
}
