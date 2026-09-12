import { api, isTauri } from "../lib/bridge";

type TitlebarProps = {
  view: "chat" | "settings";
  onView: (view: "chat" | "settings") => void;
  onNew: () => void;
  onExport?: () => void;
  canExport?: boolean;
};

export function Titlebar({ view, onView, onNew, onExport, canExport }: TitlebarProps) {
  return (
    <header className="titlebar" data-tauri-drag-region>
      <div className="brand" data-tauri-drag-region>
        <span className="mark" aria-hidden />
        <div>
          <strong>Forge Copilot</strong>
          <small>Pop!_OS · local</small>
        </div>
      </div>
      <button
        type="button"
        className="ghost new-overlay"
        data-testid="new-chat-title"
        onClick={onNew}
      >
        Nueva
      </button>
      {onExport && canExport ? (
        <button
          type="button"
          className="ghost new-overlay"
          data-testid="export-chat"
          onClick={onExport}
        >
          Exportar
        </button>
      ) : null}
      <nav className="tabs">
        <button
          className={view === "chat" ? "tab active" : "tab"}
          onClick={() => onView("chat")}
          type="button"
        >
          Chat
        </button>
        <button
          className={view === "settings" ? "tab active" : "tab"}
          onClick={() => onView("settings")}
          type="button"
        >
          Ajustes
        </button>
      </nav>
      {isTauri() ? (
        <button
          className="icon-btn"
          type="button"
          aria-label="Ocultar"
          onClick={() => void api.windowHide()}
        >
          –
        </button>
      ) : (
        <span className="demo-pill">Demo</span>
      )}
    </header>
  );
}
