import { useEffect, useRef, useState } from "react";
import SessionCard from "../components/SessionCard";
import {
  addSession,
  createSession,
  deleteSession,
  disconnectFromServer,
  forgetSession,
  getSessions,
  updateSession,
} from "../services/sessionService";
import { updateNickname } from "../services/nicknameService";
import type { Session } from "../types/session";
import TitleBar from "../components/TitleBar";
import logoIcon from "../assets/icons/logorgb.png";
import confirmIcon from "../assets/icons/confirmbtnicon.svg";
import arrowUpIcon from "../assets/icons/arrowupicon.svg";
import settingsIcon from "../assets/icons/settingbtnicon.svg";
import exitIcon from "../assets/icons/exitbtnicon.svg";
import cancelIcon from "../assets/icons/cancelbtnicon.svg";

type SessionsPageProps = {
  nickname: string;
  onDisconnect?: () => void;
  onNicknameChange?: (newNickname: string) => void;
  onConnectToSession?: (sessionName: string) => void;
  onOpenSettings?: () => void;
};
type View = "list" | "create" | "generated" | "add";

function SessionsPage({
  nickname,
  onDisconnect,
  onNicknameChange,
  onConnectToSession,
  onOpenSettings,
}: SessionsPageProps) {
  const [sessions, setSessions] = useState<Session[]>([]);
  const [showPlusMenu, setShowPlusMenu] = useState(false);
  const [showHelpPopup, setShowHelpPopup] = useState(false);
  const [view, setView] = useState<View>("list");
  const [error, setError] = useState<string | null>(null);
  const [isEditingNickname, setIsEditingNickname] = useState(false);
  const [nicknameInput, setNicknameInput] = useState(nickname);

  const [sessionNameInput, setSessionNameInput] = useState("");
  const [sessionKeyInput, setSessionKeyInput] = useState("");
  const [generatedSessionKey, setGeneratedSessionKey] = useState("");
  const [keyCopied, setKeyCopied] = useState(false);
  const copyTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

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
    setKeyCopied(false);
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
    setKeyCopied(false);
    setView("generated");
  };

  const handleCloseGenerated = () => {
    setGeneratedSessionKey("");
    setSessionNameInput("");
    setSessionKeyInput("");
    setKeyCopied(false);
    setView("list");
  };

  const handleCopyGeneratedKey = async () => {
    try {
      await navigator.clipboard.writeText(generatedSessionKey);
      setKeyCopied(true);
      if (copyTimeoutRef.current) clearTimeout(copyTimeoutRef.current);
      copyTimeoutRef.current = setTimeout(() => setKeyCopied(false), 2000);
    } catch (error) {
      console.error("Failed to copy session key:", error);
    }
  };

  const handleOpenAdd = () => {
    setShowPlusMenu(false);
    setSessionKeyInput("");
    setError(null);
    setView("add");
  };

  const handleAddConfirm = async () => {
    if (!sessionKeyInput.trim()) return;
    setError(null);
    try {
      await addSession({ sessionKey: sessionKeyInput });
      getSessions().then(setSessions);
      setSessionKeyInput("");
      setView("list");
    } catch (e: any) {
      setError(e?.message ?? String(e));
    }
  };

  const handleCancel = () => {
    setSessionNameInput("");
    setSessionKeyInput("");
    setView("list");
  };

  const handleConnect = (id: string) => {
    onConnectToSession?.(id);
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

  const isFormView = view === "create" || view === "add" || view === "generated";

  // ─────────────────────────────────────────────────────────────────────────

  return (
    <div className="servers-page">
      <TitleBar />
      <aside className="sidebar session-sidebar">
        <h1 className="logo">
          <img src={logoIcon} width="24" height="24" />
          quorthon
        </h1>

        <div className="sidebar-line" />

        {/* Only show nickname row and + button when not in a form view */}
        {!isFormView && (
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
                  <img src={confirmIcon} width="16" height="16" />
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
                      <img src={arrowUpIcon} width="16" height="16" />
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
        )}

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
                      <img src={cancelIcon} width="16" height="16" />
                    </button>
                  </div>
                  <div className="session-add-row">
                    <input
                      className="session-add-input"
                      value={sessionNameInput}
                      onChange={(e) => setSessionNameInput(e.target.value)}
                      autoFocus
                    />
                    <button
                      className="session-add-confirm-btn"
                      type="button"
                      onClick={handleCreateConfirm}
                    >
                      confirm
                    </button>
                  </div>
                </div>
              </div>
            </div>
          )}

          {view === "generated" && (
            <div className="session-create-overlay">
              <div className="generated-session-box">
                <div className="create-panel-header">
                  <span>Generated session key</span>
                </div>

                <div className="generated-key-row" style={{ position: "relative" }}>
                  <button
                    className="generated-key-text"
                    type="button"
                    onClick={handleCopyGeneratedKey}
                    title="Click to copy session key"
                    style={{ cursor: "pointer", background: "none", border: "none", padding: 0, textAlign: "left", width: "100%" }}
                  >
                    {generatedSessionKey}
                  </button>

                  {keyCopied && (
                    <div className="copy-toast">
                      session key copied!
                    </div>
                  )}
                </div>

                <button
                  className="big-confirm-btn"
                  type="button"
                  onClick={handleCloseGenerated}
                >
                  close
                </button>
              </div>
            </div>
          )}

          {view === "add" && (
            <div className="session-create-overlay">
              <div className="session-create-panels">
                <div className="create-session-box">
                  <div className="create-panel-header">
                    <span>Session key</span>
                    <button
                      className="panel-close-btn"
                      type="button"
                      onClick={handleCancel}
                    >
                      <img src={cancelIcon} width="16" height="16" />
                    </button>
                  </div>
                  <div className="session-add-row">
                    <input
                      className="session-add-input"
                      value={sessionKeyInput}
                      onChange={(e) => setSessionKeyInput(e.target.value)}
                      autoFocus
                    />

                    <button
                      className="session-add-confirm-btn"
                      type="button"
                      onClick={handleAddConfirm}
                    >
                      confirm
                    </button>
                  </div>
                  {error && <p className="error-text">{error}</p>}
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

            <button className="settings-btn" type="button" onClick={() => onOpenSettings?.()}>
              <img src={settingsIcon} width="16" height="16" />
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
                <img src={exitIcon} width="16" height="16" />
              </button>
            )}
          </div>

          <p className="version-text">ver. 0.69</p>
        </div>
      </aside>
    </div>
  );
}

export default SessionsPage;