import { useState } from "react";
import type { Server } from "../types/server";
import arrowUpIcon from "../assets/icons/arrowupicon.svg";
import arrowDownIcon from "../assets/icons/arrowdownicon.svg";
import confirmIcon from "../assets/icons/confirmbtnicon.svg";
import cancelIcon from "../assets/icons/cancelbtnicon.svg";

type ServerCardProps = {
  server: Server;
  onSaveEdit: (ip: string, nickname: string) => void;
  onRemove: (ip: string) => void;
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
      <div className="server-edit-card">
        <div className="server-edit-header">
          <span>Preferred server name</span>
        </div>
        <div className="edit-box">
          <input
            className="server-edit-input"
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
          src={expanded ? arrowUpIcon : arrowDownIcon}
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
                  onRemove(server.ipAddress);
                }}
              >
                <img src={confirmIcon} width="14" height="14" />
              </button>

              <button
                className="icon-btn"
                type="button"
                onClick={() => setConfirmForget(false)}
              >
                <img src={cancelIcon} width="14" height="14" />
              </button>
            </div>
          )}
        </div>
      )}
    </div>
  );
}

export default ServerCard;