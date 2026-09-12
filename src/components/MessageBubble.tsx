import Markdown from "react-markdown";
import remarkGfm from "remark-gfm";
import type { ChatMessage } from "../types";
import { ToolCard } from "./ToolCard";

type MessageBubbleProps = {
  message: ChatMessage;
};

export function MessageBubble({ message }: MessageBubbleProps) {
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
    </article>
  );
}
