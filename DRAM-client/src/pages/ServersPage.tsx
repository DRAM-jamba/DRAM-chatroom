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

  // ── Add server ────────────────────────────────────────────────────────────

  const handleAddServer = async () => {
    if (!newServerName.trim() || !newServerIp.trim()) return;
    setError(null);
    try {
      await addServer({ nickname: newServerName, ip: newServerIp });
      const updated = await getServers();
      setServers(updated);
      setNewServerName("");
      setNewServerIp("");
      setShowAddForm(false);
    } catch (e: any) {
      setError(e?.message ?? String(e));
    }
  };

  // ── Connect to server ─────────────────────────────────────────────────────

  const handleConnect = async (ip: string) => {
    setError(null);
    try {
      await connectServer(ip);
      onOpenSessions?.();
    } catch (e: any) {
      setError(e?.message ?? String(e));
    }
  };

  // ── Rename server (nickname only) ─────────────────────────────────────────

  const handleSaveEdit = async (ip: string, nickname: string) => {
    const updated = await updateServer(ip, { nickname });
    setServers((prev) => prev.map((s) => (s.ip === ip ? updated : s)));
  };

  // ── Remove server ─────────────────────────────────────────────────────────

  const handleRemove = async (ip: string) => {
    setError(null);
    try {
      await removeServer(ip);
      setServers((prev) => prev.filter((s) => s.ip !== ip));
    } catch (e: any) {
      setError(e?.message ?? String(e));
    }
  };

  // ─────────────────────────────────────────────────────────────────────────

  return (
    <div className="servers-page">
      <aside className="sidebar">
        <h1 className="logo">quorthon</h1>

        <div className="sidebar-line" />

        <div className="server-list-container">
          {servers.map((server) => (
            <ServerCard
              key={server.ip}
              server={server}
              onSaveEdit={handleSaveEdit}
              onRemove={handleRemove}
              onConnect={handleConnect}
            />
          ))}
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
