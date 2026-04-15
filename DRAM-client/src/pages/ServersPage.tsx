import { useEffect, useState } from "react";
import ServerCard from "../components/ServerCard";
import {
  addServer,
  getServers,
  removeServer,
  updateServer,
} from "../services/serverService";
import type { Server } from "../types/server";
import { invoke } from "@tauri-apps/api/core";
import { getNickname } from "../services/nicknameService";

type ServersPageProps = {
  onOpenSessions?: () => void;
};

function ServersPage({ onOpenSessions }: ServersPageProps) {
  const [servers, setServers] = useState<Server[]>([]);
  const [showAddForm, setShowAddForm] = useState(false);
  const [newServerName, setNewServerName] = useState("");
  const [newServerIp, setNewServerIp] = useState("");

  useEffect(() => {
    loadServers();
  }, []);

  const loadServers = async () => {
    const data = await getServers();
    setServers(data);
  };

  const handleAddServer = async () => {
    if (!newServerName.trim() || !newServerIp.trim()) {
      return;
    }

    const createdServer = await addServer({
      name: newServerName,
      ipAddress: newServerIp,
    });

    setServers((prev) => [...prev, createdServer]);
    setNewServerName("");
    setNewServerIp("");
    setShowAddForm(false);
  };

  const handleSaveEdit = async (
    id: string,
    name: string,
    ipAddress: string
  ) => {
    const updatedServer = await updateServer(id, { name, ipAddress });

    setServers((prev) =>
      prev.map((server) => (server.id === id ? updatedServer : server))
    );
  };

  const handleRemove = async (id: string) => {
    await removeServer(id);
    setServers((prev) => prev.filter((server) => server.id !== id));
  };

  const handleConnect = async (id: string) => {
    const selectedServer = servers.find((server) => server.id === id);
  if (!selectedServer) return;

  try {
    const nickname = getNickname();
    
    await invoke<void>("connect", {
      ip: selectedServer.ipAddress, 
      nickname: nickname || undefined,
    });

    if (onOpenSessions) onOpenSessions();
  } catch (error) {
    console.error("Failed to connect:", error);
  }
  };

  return (
    <div className="servers-page">
      <aside className="sidebar">
        <h1 className="logo">quorthon</h1>

        <div className="sidebar-line" />

        <div className="server-list-container">
          {servers.map((server) => (
            <ServerCard
              key={server.id}
              server={server}
              onSaveEdit={handleSaveEdit}
              onRemove={handleRemove}
              onConnect={handleConnect}
            />
          ))}
        </div>

        <p className="trusted-text">Connect only to trusted servers</p>

        {!showAddForm ? (
          <button
            className="plus-button"
            type="button"
            onClick={() => setShowAddForm(true)}
          >
            +
          </button>
        ) : (
          <div className="add-server-box">
            <label className="input-label">Preferred server name</label>
            <input
              className="server-input"
              value={newServerName}
              onChange={(e) => setNewServerName(e.target.value)}
            />

            <label className="input-label">IP Address</label>
            <input
              className="server-input"
              value={newServerIp}
              onChange={(e) => setNewServerIp(e.target.value)}
            />

            <div className="add-server-actions">
              <button
                className="cancel-btn"
                type="button"
                onClick={() => {
                  setShowAddForm(false);
                  setNewServerName("");
                  setNewServerIp("");
                }}
              >
                back
              </button>

              <button
                className="big-confirm-btn"
                type="button"
                onClick={handleAddServer}
              >
                confirm
              </button>
            </div>
          </div>
        )}

        <div className="sidebar-line bottom-line" />
        <p className="version-text">ver. 0.2</p>
      </aside>

      <main className="main-panel">
        <div className="instructions">
          <p>use left panel to:</p>
          <ul>
            <li>add servers</li>
            <li>edit server info</li>
            <li>remove servers</li>
          </ul>
        </div>
      </main>
    </div>
  );
}

export default ServersPage;