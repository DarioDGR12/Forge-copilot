import type { ChatMessage } from "../types";

type ToolCardProps = {
  message: ChatMessage;
};

const LABELS: Record<string, string> = {
  run_terminal: "Terminal",
  read_file: "Leer archivo",
  write_file: "Escribir archivo",
  list_dir: "Listar carpeta",
  list_apps: "Aplicaciones",
  list_processes: "Procesos",
  screenshot: "Captura",
};

export function ToolCard({ message }: ToolCardProps) {
  const name = message.toolName ?? "herramienta";
  const status = message.status ?? "done";
  return (
    <article className={`tool-card ${status}`}>
      <header>
        <span className="tool-name">{LABELS[name] ?? name}</span>
        <span className="tool-status">{statusLabel(status)}</span>
      </header>
      {message.content ? <pre>{message.content}</pre> : <p className="muted">Ejecutando…</p>}
      {message.imageBase64 ? (
        <img
          className="shot"
          alt="Captura de pantalla"
          src={`data:image/png;base64,${message.imageBase64}`}
        />
      ) : null}
    </article>
  );
}

function statusLabel(status: string): string {
  switch (status) {
    case "running":
      return "en curso";
    case "denied":
      return "denegado";
    case "error":
      return "error";
    default:
      return "listo";
  }
}
