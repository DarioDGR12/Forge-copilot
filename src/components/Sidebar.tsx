import { useMemo, useState } from "react";
import type { Conversation } from "../types";

type SidebarProps = {
  items: Conversation[];
  activeId: string | null;
  onSelect: (id: string) => void;
  onNew: () => void;
  onDelete: (id: string) => void;
};

function formatWhen(epoch: number): string {
  if (!epoch) return "";
  return new Date(epoch * 1000).toLocaleDateString("es", {
    day: "numeric",
    month: "short",
  });
}

export function Sidebar({ items, activeId, onSelect, onNew, onDelete }: SidebarProps) {
  const [query, setQuery] = useState("");
  const visible = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return items;
    return items.filter((item) => item.title.toLowerCase().includes(q));
  }, [items, query]);

  return (
    <aside className="sidebar">
      <button className="new-chat" type="button" onClick={onNew}>
        Nueva conversación
      </button>
      <input
        className="conv-search"
        data-testid="conv-search"
        type="search"
        placeholder="Buscar chats…"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
      />
      <ul className="conv-list">
        {visible.length === 0 ? (
          <li className="empty-list">{items.length === 0 ? "Aún no hay chats" : "Sin coincidencias"}</li>
        ) : (
          visible.map((item) => (
            <li key={item.id} className={item.id === activeId ? "conv active" : "conv"}>
              <button type="button" className="conv-main" onClick={() => onSelect(item.id)}>
                <span>{item.title}</span>
                <small>{formatWhen(item.updatedAt)}</small>
              </button>
              <button
                type="button"
                className="ghost danger"
                aria-label="Eliminar"
                onClick={() => onDelete(item.id)}
              >
                ×
              </button>
            </li>
          ))
        )}
      </ul>
    </aside>
  );
}
