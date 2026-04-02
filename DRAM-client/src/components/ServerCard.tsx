import { useState } from "react";
import type { Server } from "../types/server";

type ServerCardProps = {
  server: Server;
  onSaveEdit: (id: string, name: string, ipAddress: string) => void;
  onRemove: (id: string) => void;
  onConnect: (id: string) => void;
};

function ServerCard({
  server,
  onSaveEdit,
  onRemove,
  onConnect,
}: ServerCardProps) {
  const [expanded, setExpanded] = useState(false);
  const [isEditing, setIsEditing] = useState(false);
  const [editedName, setEditedName] = useState(server.name);
  const [editedIp, setEditedIp] = useState(server.ipAddress);

  const handleConfirmEdit = () => {
    onSaveEdit(server.id, editedName, editedIp);
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
            placeholder="Server name"
          />
          <input
            className="server-input"
            value={editedIp}
            onChange={(e) => setEditedIp(e.target.value)}
            placeholder="IP Address"
          />
          <button
            className="small-btn connect-btn"
            type="button"
            onClick={handleConfirmEdit}
          >
            confirm
          </button>
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
        <span className="server-title">{server.name}</span>
        <span className="server-arrow">{expanded ? "⌃" : "⌄"}</span>
      </button>

      {expanded && (
        <div className="server-details">
          <p className="server-ip">IP: {server.ipAddress}</p>

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
              onClick={() => onRemove(server.id)}
            >
              forget
            </button>

            <button
              className="small-btn connect-btn"
              type="button"
              onClick={() => onConnect(server.id)}
            >
              connect
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

export default ServerCard;