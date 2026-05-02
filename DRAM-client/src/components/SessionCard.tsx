import { useState } from "react";
import type { Session } from "../types/session";

type SessionCardProps = {
  session: Session;
  onSaveEdit: (id: string, name: string) => void;
  onRemove: (id: string) => void;
  onDelete: (id: string) => void;
  onConnect: (id: string) => void;
};

function SessionCard({
  session,
  onSaveEdit,
  onRemove,
  onDelete,
  onConnect,
}: SessionCardProps) {
  const [expanded, setExpanded] = useState(false);
  const [isEditing, setIsEditing] = useState(false);
  const [confirmAction, setConfirmAction] = useState<"forget" | "delete" | null>(null);
  const [editedName, setEditedName] = useState(session.name);

  const handleConfirmEdit = () => {
    if (!editedName.trim()) {
      return;
    }

    onSaveEdit(session.id, editedName);
    setIsEditing(false);
  };

  if (isEditing) {
    return (
      <div className="server-card">
        <div className="edit-box">
          <input
            className="server-input"
            value={editedName}
            onChange={(e) => setEditedName(e.target.value)}
            placeholder="Session name"
          />

          <div className="add-server-actions">
            <button
              className="cancel-btn"
              type="button"
              onClick={() => {
                setIsEditing(false);
                setEditedName(session.name);
              }}
            >
              back
            </button>

            <button
              className="small-btn connect-btn"
              type="button"
              onClick={handleConfirmEdit}
            >
              confirm
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="server-card">
      <button
        className="server-header"
        type="button"
        onClick={() => setExpanded(!expanded)}
      >
        <span className="server-title">{session.name}</span>
        <span className="server-arrow">{expanded ? "⌃" : "⌄"}</span>
      </button>

      {expanded && (
        <div className="server-details">
          {confirmAction === null && (
            <div className="server-actions">
              <button
                className="small-btn edit-btn"
                type="button"
                onClick={() => setIsEditing(true)}
              >
                edit
              </button>

              <button
                className="small-btn forget-btn"
                type="button"
                onClick={() => setConfirmAction("forget")}
              >
                forget
              </button>

              <button
                className="small-btn delete-btn"
                type="button"
                onClick={() => setConfirmAction("delete")}
              >
                delete
              </button>

              <button
                className="small-btn connect-btn"
                type="button"
                onClick={() => onConnect(session.id)}
              >
                connect
              </button>
            </div>
          )}

          {confirmAction === "forget" && (
            <div className="session-delete-row">
              <span className="session-delete-text">forget session?</span>

              <button
                className="icon-btn"
                type="button"
                onClick={() => {
                  setConfirmAction(null);
                  onRemove(session.id);
                }}
              >
                ✓
              </button>

              <button
                className="icon-btn"
                type="button"
                onClick={() => setConfirmAction(null)}
              >
                ×
              </button>
            </div>
          )}

          {confirmAction === "delete" && (
            <div className="session-delete-row">
              <span className="session-delete-text">delete for everyone?</span>

              <button
                className="icon-btn"
                type="button"
                onClick={() => {
                  setConfirmAction(null);
                  onDelete(session.id);
                }}
              >
                ✓
              </button>

              <button
                className="icon-btn"
                type="button"
                onClick={() => setConfirmAction(null)}
              >
                ×
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

export default SessionCard;