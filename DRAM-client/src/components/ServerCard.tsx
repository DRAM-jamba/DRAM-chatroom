import { useState } from "react";
import type { Server } from "../types/server";

type ServerCardProps = {
  server: Server;
  onSaveEdit: (ip: string, nickname: string) => void;
  onRemove: (id: string) => void;
  onConnect: (ip: string) => void;
};

function ServerCard({
  server,
  onSaveEdit,
  onRemove,
  onConnect,
}: ServerCardProps) {
  const [expanded, setExpanded] = useState(false);
  const [isEditing, setIsEditing] = useState(false);
  const [confirmForget, setConfirmForget] = useState(false);
  const [editedNickname, setEditedNickname] = useState(server.name);

  const handleConfirmEdit = () => {
    if (!editedNickname.trim()) return;
    onSaveEdit(server.ipAddress, editedNickname);
    setIsEditing(false);
  };

  if (isEditing) {
    return (
      <div className="server-card">
        <div className="edit-box">
          <input
            className="server-input"
            value={editedNickname}
            onChange={(e) => setEditedNickname(e.target.value)}
            placeholder="Server name"
            autoFocus
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
        <img
          src={expanded ? "/src/assets/icons/arrowupicon.svg" : "/src/assets/icons/arrowdownicon.svg"}
          width="16"
          height="16"
        />
      </button>

      {expanded && (
        <div className="server-details">
          <p className="server-ip">IP: {server.ipAddress}</p>

          {!confirmForget ? (
            <div className="server-actions">
              <button
                className="small-btn edit-btn"
                type="button"
                onClick={() => {
                  setEditedNickname(server.name);
                  setIsEditing(true);
                }}
              >
                edit
              </button>

              <button
                className="small-btn forget-btn"
                type="button"
                onClick={() => setConfirmForget(true)}
              >
                forget
              </button>

              <button
                className="small-btn connect-btn"
                type="button"
                onClick={() => onConnect(server.ipAddress)}
              >
                connect
              </button>
            </div>
          ) : (
            <div className="session-delete-row">
              <span className="session-delete-text">forget server?</span>

              <button
                className="icon-btn"
                type="button"
                onClick={() => {
                  setConfirmForget(false);
                  onRemove(server.id);
                }}
              >
                <img src="/src/assets/icons/confirmbtnicon.svg" width="14" height="14" />
              </button>

              <button
                className="icon-btn"
                type="button"
                onClick={() => setConfirmForget(false)}
              >
                <img src="/src/assets/icons/cancelbtnicon.svg" width="14" height="14" />
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

export default ServerCard;