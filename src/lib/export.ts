import type { ChatMessage } from "../types";

export function conversationMarkdown(messages: ChatMessage[], title = "Forge Copilot"): string {
  const lines = [`# ${title}`, "", `_Exportado ${new Date().toLocaleString("es")}_`, ""];
  for (const message of messages) {
    if (message.id === "stream") continue;
    if (message.role === "user") {
      lines.push("## Tú", "", message.content, "");
    } else if (message.role === "assistant") {
      lines.push("## Forge", "", message.content || "_(vacío)_", "");
    } else if (message.role === "tool") {
      lines.push(
        `### ${message.toolName ?? "herramienta"}`,
        "",
        "```",
        message.content || "",
        "```",
        "",
      );
    }
  }
  return lines.join("\n").trim() + "\n";
}

export function downloadText(filename: string, text: string) {
  const blob = new Blob([text], { type: "text/markdown;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}
