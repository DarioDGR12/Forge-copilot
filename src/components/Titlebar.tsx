import { api, isTauri } from "../lib/bridge";

type TitlebarProps = {
  view: "chat" | "settings";
  onView: (view: "chat" | "settings") => void;
};

export function Titlebar({ view, onView }: TitlebarProps) {
  return (
    <header className="titlebar" data-tauri-drag-region>
      <div className="brand" data-tauri-drag-region>
        <span className="mark" aria-hidden />
        <div>
          <strong>Forge</strong>
          <small>Copilot · Pop!_OS</small>
        </div>
      </div>
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
