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
  return (
    <aside className="sidebar">
      <button className="new-chat" type="button" onClick={onNew}>
        Nueva conversación
      </button>
      <ul className="conv-list">
        {items.length === 0 ? (
          <li className="empty-list">Aún no hay chats</li>
        ) : (
          items.map((item) => (
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
