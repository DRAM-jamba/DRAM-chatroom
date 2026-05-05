import { useEffect, useState } from "react";
import SessionCard from "../components/SessionCard";
import {
  addSession,
  createSession,
  deleteSession,
  disconnectFromServer,
  forgetSession,
  getSessions,
  joinSession,
  updateSession,
} from "../services/sessionService";
import { updateNickname } from "../services/nicknameService";
import type { Session } from "../types/session";
import TitleBar from "../components/TitleBar";

type SessionsPageProps = {
  nickname: string;
  onDisconnect?: () => void;
  onNicknameChange?: (newNickname: string) => void;
  onConnectToSession?: (sessionName: string) => void;
};

type View = "list" | "create" | "generated" | "add";

function SessionsPage({
  nickname,
  onDisconnect,
  onNicknameChange,
  onConnectToSession,
}: SessionsPageProps) {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [showPlusMenu, setShowPlusMenu] = useState(false);
  const [showHelpPopup, setShowHelpPopup] = useState(false);
  const [view, setView] = useState<View>("list");

  const [isEditingNickname, setIsEditingNickname] = useState(false);
  const [nicknameInput, setNicknameInput] = useState(nickname);

  const [sessionNameInput, setSessionNameInput] = useState("");
  const [sessionKeyInput, setSessionKeyInput] = useState("");
  const [generatedSessionKey, setGeneratedSessionKey] = useState(""); 

  useEffect(() => {
    getSessions().then(setSessions);
  }, []);

  const handleNicknameConfirm = async () => {
    const trimmed = nicknameInput.trim();
    if (!trimmed) return;
    await updateNickname(trimmed);
    setIsEditingNickname(false);
    onNicknameChange?.(trimmed);
  };

  const handleNicknameKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") handleNicknameConfirm();
    if (e.key === "Escape") {
      setNicknameInput(nickname);
      setIsEditingNickname(false);
    }
  };

  const handleOpenCreate = () => {
    setShowPlusMenu(false);
    setSessionNameInput("");
    setSessionKeyInput("");
    setGeneratedSessionKey("");
    setView("create");
  };

  const handleCreateConfirm = async () => {
    if (!sessionNameInput.trim()) return;
    const result = await createSession({
      sessionName: sessionNameInput,
      sessionKey: sessionKeyInput,
    });
    setSessions((prev) => [...prev, result.session]);
    setGeneratedSessionKey(result.generatedKey);
    setView("generated");
  };

  const handleCloseGenerated = () => {
    setGeneratedSessionKey("");
    setSessionNameInput("");
    setSessionKeyInput("");
    setView("list");
  };

  const handleCopyGeneratedKey = async () => {
    try {
      await navigator.clipboard.writeText(generatedSessionKey);
    } catch (error) {
      console.error("Failed to copy session key:", error);
    }
  };


  const handleOpenAdd = () => {
    setShowPlusMenu(false);
    setSessionKeyInput("");
    setView("add");
  };

  const handleAddConfirm = async () => {
    if (!sessionKeyInput.trim()) return;
    await addSession({ sessionKey: sessionKeyInput });
    getSessions().then(setSessions);
    setSessionKeyInput("");
    setView("list");
  };

  const handleCancel = () => {
    setSessionNameInput("");
    setSessionKeyInput("");
    setView("list");
  };

  const handleConnect = async (id: string) => {
    const sessionName = await joinSession(id);
    onConnectToSession?.(sessionName);
  };

  const handleSaveEdit = async (id: string, name: string) => {
    const updated = await updateSession(id, { name });
    setSessions((prev) => prev.map((s) => (s.id === id ? updated : s)));
  };

  const handleForget = async (sessionKey: string) => {
    try {
      await forgetSession(sessionKey);
      setSessions((prev) => prev.filter((s) => s.id !== sessionKey));
    } catch (error) {
      console.error("Failed to forget session:", error);
      getSessions().then(setSessions);
    }
  };

  const handleDelete = async (sessionKey: string) => {
    try {
      await deleteSession(sessionKey);
      setSessions((prev) => prev.filter((s) => s.id !== sessionKey));
    } catch (error) {
      console.error("Failed to delete session:", error);
      getSessions().then(setSessions);
    }
  };

  // ─────────────────────────────────────────────────────────────────────────

  return (
    <div className="servers-page">
      <TitleBar />
      <aside className="sidebar session-sidebar">
        <h1 className="logo">
          <img src="/src/assets/icons/logorgb.png" width="24" height="24" />
          quorthon
        </h1>

        <div className="sidebar-line" />

        <div className="session-top-row">
          {isEditingNickname ? (
            <div className="nickname-edit-row">
              <input
                className="server-input nickname-edit-input"
                value={nicknameInput}
                onChange={(e) => setNicknameInput(e.target.value)}
                onKeyDown={handleNicknameKeyDown}
                autoFocus
                maxLength={32}
              />
              <button
                className="nickname-confirm-inline-btn"
                type="button"
                onClick={handleNicknameConfirm}
                title="Confirm nickname"
              >
                <img src="/src/assets/icons/confirmbtnicon.svg" width="16" height="16" />
              </button>
            </div>
          ) : (
            <button
              className="session-user-box session-user-box-btn"
              type="button"
              onClick={() => {
                setNicknameInput(nickname);
                setIsEditingNickname(true);
              }}
              title="Click to edit nickname"
            >
              Hello {nickname}!
            </button>
          )}

          <div className="session-plus-wrapper">
            {!showPlusMenu ? (
              <button
                className="session-plus-button"
                type="button"
                onClick={() => setShowPlusMenu(true)}
              >
                +
              </button>
            ) : (
              <div className="session-plus-menu">
                <button
                  className="session-plus-close"
                  type="button"
                  onClick={() => setShowPlusMenu(false)}
                >
                  ⌃
                </button>

                <button
                  className="session-plus-option"
                  type="button"
                  onClick={handleOpenCreate}
                >
                  create
                </button>

                <button
                  className="session-plus-option"
                  type="button"
                  onClick={handleOpenAdd}
                >
                  add
                </button>
              </div>
            )}
          </div>
        </div>

        <div className="session-content-area">
          {view === "create" && (
            <div className="session-create-overlay">
              <div className="session-create-panels">
                <div className="create-session-box">
                  <div className="create-panel-header">
                    <span>Preferred session name</span>
                    <button
                      className="panel-close-btn"
                      type="button"
                      onClick={handleCancel}
                    >
                      ×
                    </button>
                  </div>
                  <input
                    className="server-input"
                    value={sessionNameInput}
                    onChange={(e) => setSessionNameInput(e.target.value)}
                  />
                  <button
                    className="big-confirm-btn"
                    type="button"
                    onClick={handleCreateConfirm}
                  >
                    confirm
                  </button>
                </div>
              </div>
            </div>
          )}

          {view === "generated" && (
            <div className="generated-session-box">
              <div className="create-panel-header">
                <span>Generated session key</span>
              </div>

              <div className="generated-key-row">
                <div className="generated-key-text">{generatedSessionKey}</div>
                <button
                  className="copy-key-btn"
                  type="button"
                  onClick={handleCopyGeneratedKey}
                >
                  copy
                </button>
              </div>

              <button
                className="big-confirm-btn"
                type="button"
                onClick={handleCloseGenerated}
              >
                close
              </button>
            </div>
          )}

          {view === "add" && (
            <div className="session-create-overlay">
              <div className="session-create-panels">
                <div className="create-session-box">
                  <div className="create-panel-header">
                    <span>Session name</span>
                    <button
                      className="panel-close-btn"
                      type="button"
                      onClick={handleCancel}
                    >
                      ×
                    </button>
                  </div>
                  <input
                    className="server-input"
                    value={sessionNameInput}
                    onChange={(e) => setSessionNameInput(e.target.value)}
                  />
                </div>

                <div className="create-session-box">
                  <div className="create-panel-header">
                    <span>Session key</span>
                    <button
                      className="panel-close-btn"
                      type="button"
                      onClick={handleCancel}
                    >
                      ×
                    </button>
                  </div>
                  <input
                    className="server-input"
                    value={sessionKeyInput}
                    onChange={(e) => setSessionKeyInput(e.target.value)}
                  />
                  <button
                    className="big-confirm-btn"
                    type="button"
                    onClick={handleAddConfirm}
                  >
                    confirm
                  </button>
                </div>
              </div>
            </div>
          )}

          <div
            className={`server-list-container ${
              view !== "list" ? "session-list-hidden" : ""
            }`}
          >
            {sessions.map((session) => (
              <SessionCard
                key={session.id}
                session={session}
                onSaveEdit={handleSaveEdit}
                onRemove={handleForget}
                onDelete={handleDelete}
                onConnect={handleConnect}
              />
            ))}
          </div>
        </div>

        <div className="sidebar-line bottom-line" />

        <div className="session-bottom-row">
          <div className="left-bottom-buttons">
            <div className="help-popup-wrapper">
              <button
                className="tiny-square-btn"
                type="button"
                onClick={() => setShowHelpPopup((prev) => !prev)}
              >
                ?
              </button>

              {showHelpPopup && (
                <div
                  className="help-popup"
                  onClick={() => setShowHelpPopup(false)}
                >
                  <div className="help-popup-content">
                    <p>• You can have up to 10 sessions</p>
                    <p>
                      • You are removed from the session
                      <br />
                      after 1 month of inactivity
                    </p>
                    <span className="help-popup-close-text">click to close</span>
                  </div>
                </div>
              )}
            </div>

            <button className="settings-btn" type="button">
              <img src="/src/assets/icons/settingbtnicon.svg" width="16" height="16" />
            </button>

            {onDisconnect && (
              <button
                className="small-btn disconnect-btn"
                type="button"
                onClick={async () => {
                  try {
                    await disconnectFromServer();
                  } catch (error) {
                    console.error("Failed to disconnect:", error);
                  } finally {
                    onDisconnect();
                  }
                }}
              >
                <img src="/src/assets/icons/exitbtnicon.svg" width="16" height="16" />
              </button>
            )}
          </div>

          <p className="version-text">ver. 0.2</p>
        </div>
      </aside>
    </div>
  );
}

export default SessionsPage;