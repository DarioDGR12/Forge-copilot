import { useEffect, useState } from "react";
import { api } from "../lib/bridge";
import type { KeyStatus, Settings } from "../types";

const PROVIDERS = [
  { id: "demo", label: "Demostración (sin clave)" },
  { id: "openai", label: "OpenAI" },
  { id: "anthropic", label: "Anthropic" },
  { id: "openrouter", label: "OpenRouter" },
  { id: "ollama", label: "Ollama (local)" },
] as const;

type SettingsPanelProps = {
  onSaved?: (settings: Settings) => void;
};

export function SettingsPanel({ onSaved }: SettingsPanelProps) {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [key, setKey] = useState("");
  const [status, setStatus] = useState<KeyStatus | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  async function reload(provider?: string) {
    const next = await api.getSettings();
    setSettings(next);
    const st = await api.keyStatus(provider ?? next.provider);
    setStatus(st);
  }

  useEffect(() => {
    void reload().catch((err: unknown) =>
      setNotice(err instanceof Error ? err.message : String(err)),
    );
  }, []);

  if (!settings) {
    return <section className="settings">Cargando ajustes…</section>;
  }

  const current = settings;
  const needsKey = current.provider !== "demo" && current.provider !== "ollama";

  async function save() {
    setBusy(true);
    setNotice(null);
    try {
      const saved = await api.saveSettings({
        provider: current.provider,
        model: current.model,
        baseUrl: current.baseUrl,
        shortcut: current.shortcut,
        allowedRoots: current.allowedRoots,
      });
      setSettings(saved);
      onSaved?.(saved);
      setNotice("Ajustes guardados.");
    } catch (err) {
      setNotice(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  async function saveKey() {
    setBusy(true);
    setNotice(null);
    try {
      const st = await api.setApiKey(current.provider, key);
      setStatus(st);
      setKey("");
      setNotice("Clave guardada en el llavero local.");
    } catch (err) {
      setNotice(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  async function clearKey() {
    const st = await api.clearApiKey(current.provider);
    setStatus(st);
    setNotice("Clave eliminada.");
  }

  async function test() {
    setBusy(true);
    setNotice(null);
    try {
      await api.saveSettings(current);
      const msg = await api.testConnection();
      setNotice(msg);
    } catch (err) {
      setNotice(err instanceof Error ? err.message : String(err));
    } finally {
      setBusy(false);
    }
  }

  return (
    <section className="settings">
      <h2>BYOK y sistema</h2>
      <p className="muted">
        Las claves se guardan en el keyring de Linux (o un archivo 0600 si no hay Secret Service).
        El frontend nunca las vuelve a leer.
      </p>

      <label>
        Proveedor
        <select
          value={settings.provider}
          onChange={(e) => {
            const provider = e.target.value;
            setSettings({ ...settings, provider });
            void api.keyStatus(provider).then(setStatus);
          }}
        >
          {PROVIDERS.map((p) => (
            <option key={p.id} value={p.id}>
              {p.label}
            </option>
          ))}
        </select>
      </label>

      <label>
        Modelo
        <input
          value={settings.model}
          onChange={(e) => setSettings({ ...settings, model: e.target.value })}
          placeholder="gpt-4o-mini"
        />
      </label>

      <label>
        Base URL (opcional)
        <input
          value={settings.baseUrl}
          onChange={(e) => setSettings({ ...settings, baseUrl: e.target.value })}
          placeholder="https://openrouter.ai/api/v1"
        />
      </label>

      {needsKey || settings.provider === "ollama" ? (
        <label>
          API key
          <input
            type="password"
            value={key}
            autoComplete="off"
            placeholder={status?.configured ? `Guardada ${status.hint}` : "sk-…"}
            onChange={(e) => setKey(e.target.value)}
          />
        </label>
      ) : null}

      <label>
        Carpetas permitidas (una por línea)
        <textarea
          rows={4}
          value={settings.allowedRoots.join("\n")}
          onChange={(e) =>
            setSettings({
              ...settings,
              allowedRoots: e.target.value.split("\n"),
            })
          }
        />
      </label>

      <label>
        Atajo global
        <input
          value={settings.shortcut}
          onChange={(e) => setSettings({ ...settings, shortcut: e.target.value })}
        />
      </label>
      <p className="hint">
        Por defecto <kbd>Ctrl+Shift+Space</kbd>. En Wayland el atajo global puede fallar; usa la
        bandeja del sistema.
      </p>

      <div className="settings-actions">
        <button type="button" className="primary" disabled={busy} onClick={() => void save()}>
          Guardar
        </button>
        {needsKey || settings.provider === "openai" || settings.provider === "anthropic" || settings.provider === "openrouter" ? (
          <>
            <button type="button" disabled={busy || !key} onClick={() => void saveKey()}>
              Guardar clave
            </button>
            <button type="button" className="ghost" disabled={!status?.configured} onClick={() => void clearKey()}>
              Borrar clave
            </button>
          </>
        ) : null}
        <button type="button" className="ghost" disabled={busy} onClick={() => void test()}>
          Probar conexión
        </button>
      </div>
      {notice ? <p className="notice">{notice}</p> : null}
    </section>
  );
}
