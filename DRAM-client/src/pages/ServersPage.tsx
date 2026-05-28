import { useEffect, useState } from "react";
import ServerCard from "../components/ServerCard";
import {
  addServer,
  connectServer,
  getServers,
  removeServer,
  updateServer,
} from "../services/serverService";
import type { Server } from "../types/server";
import TitleBar from "../components/TitleBar";
import logoIcon from "../assets/icons/logorgb.png";

type ServersPageProps = {
  onOpenSessions?: () => void;
};

function ServersPage({ onOpenSessions }: ServersPageProps) {
  const [servers, setServers] = useState<Server[]>([]);
  const [showAddForm, setShowAddForm] = useState(false);
  const [newServerName, setNewServerName] = useState("");
  const [newServerIp, setNewServerIp] = useState("");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    getServers().then(setServers);
  }, []);

  const isValidAddress = (ip: string): boolean => {
    return /^[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}(:\d+)?$|^\d{1,3}(\.\d{1,3}){3}(:\d+)?$/.test(ip.trim());
  };

  const handleAddServer = async () => {
    if (!newServerName.trim() && !newServerIp.trim()) {
      setError("Name and address can't be empty.");
      return;
    }
    if (!newServerName.trim()) {
      setError("Name can't be empty.");
      return;
    }
    if (!newServerIp.trim()) {
      setError("Address can't be empty.");
      return;
    }
    if (!isValidAddress(newServerIp)) {
      setError("Address is invalid.");
      return;
    }
    if (servers.some((s) => s.ipAddress === newServerIp.trim())) {
      setError("This server already exists.");
      return;
    }
    setError(null);
    try {
      await addServer({ nickname: newServerName, ip: newServerIp });
      const updated = await getServers();
      setServers(updated);
      setNewServerName("");
      setNewServerIp("");
      setShowAddForm(false);
    } catch (e: any) {
      setError("Connection failed: Invalid or inactive link.");
    }
  };

  const handleConnect = async (ip: string) => {
    setError(null);
    try {
      await connectServer(ip);
      onOpenSessions?.();
    } catch (e: any) {
      setError(e?.message ?? String(e));
    }
  };

  const handleSaveEdit = async (ip: string, nickname: string) => {
    const updated = await updateServer(ip, { nickname });
    setServers((prev) =>
      prev.map((s) => (s.ipAddress === ip ? updated : s))
    );
  };

  const handleRemove = async (ip: string) => {
    setError(null);
    try {
      await removeServer(ip);
      setServers((prev) => prev.filter((s) => s.ipAddress !== ip));
    } catch (e: any) {
      setError(e?.message ?? String(e));
    }
  };

  return (
    <div className="servers-page">
      <TitleBar />
      <aside className="sidebar">
        <h1 className="logo">
          <img src={logoIcon} width="24" height="24" />
          quorthon
        </h1>

        <div className="sidebar-line" />

        <div className="server-list-container">
          {servers.length === 0 ? (
            <p className="empty-list-text" style={{ textAlign: "center" }}>
              Your server list is empty
            </p>
          ) : (
            servers.map((server) => (
              <ServerCard
                key={server.id}
                server={server}
                onSaveEdit={handleSaveEdit}
                onRemove={handleRemove}
                onConnect={handleConnect}
              />
            ))
          )}
        </div>

        {error && <p className="error-text">{error}</p>}

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

              <label className="input-label">Address</label>
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
                  setError(null);
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
        <p className="version-text">ver. 1.0.0</p>
      </aside>
    </div>
  );
}

export default ServersPage;