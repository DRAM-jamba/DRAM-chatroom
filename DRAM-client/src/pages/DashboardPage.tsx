import { useMemo, useState } from "react";
import DashboardCard from "../components/DashboardCard";
import { ConnectionData, SessionItem } from "../types/app";

type DashboardPageProps = {
  connection: ConnectionData;
  joinedSessions: SessionItem[];
  onBackToJoin: () => void;
  onChangeUserName: (newName: string) => void;
  onCreateSession: (sessionName: string) => void;
  onJoinSessionByKey: (sessionKey: string) => void;
};

function DashboardPage({
  connection,
  joinedSessions,
  onBackToJoin,
  onChangeUserName,
  onCreateSession,
  onJoinSessionByKey,
}: DashboardPageProps) {
  const [newUserName, setNewUserName] = useState(connection.displayName);
  const [newSessionName, setNewSessionName] = useState("");
  const [sessionKeyInput, setSessionKeyInput] = useState("");
  const [feedback, setFeedback] = useState("");

  const sessionCountLabel = useMemo(() => {
    if (joinedSessions.length === 0) return "No joined sessions yet";
    if (joinedSessions.length === 1) return "1 joined session";
    return `${joinedSessions.length} joined sessions`;
  }, [joinedSessions]);

  function handleChangeUserName() {
    const trimmed = newUserName.trim();

    if (trimmed.length < 2) {
      setFeedback("User name must contain at least 2 characters.");
      return;
    }

    onChangeUserName(trimmed);
    setFeedback(`User name updated to ${trimmed}.`);
  }

  function handleCreateSession() {
    const trimmed = newSessionName.trim();

    if (trimmed.length < 3) {
      setFeedback("Session name must contain at least 3 characters.");
      return;
    }

    onCreateSession(trimmed);
    setNewSessionName("");
    setFeedback(`Session "${trimmed}" created.`);
  }

  function handleJoinSessionByKey() {
    const trimmed = sessionKeyInput.trim();

    if (trimmed.length < 4) {
      setFeedback("Please enter a valid session key.");
      return;
    }

    onJoinSessionByKey(trimmed);
    setSessionKeyInput("");
    setFeedback(`Join request prepared for key: ${trimmed}.`);
  }

  return (
    <main className="dashboard-shell">
      <section className="background-glow background-glow-left" />
      <section className="background-glow background-glow-right" />

      <div className="dashboard-layout">
        <header className="dashboard-hero">
          <div>
            <div className="join-card__badge">Connected Client</div>
            <h1>Welcome, {connection.displayName}</h1>
            <p>
              Server: {connection.serverAddress}:{connection.port}
            </p>
          </div>

          <button className="secondary-button" onClick={onBackToJoin}>
            Change Server
          </button>
        </header>

        <section className="dashboard-summary">
          <div className="summary-box">
            <span className="summary-box__label">Current User</span>
            <strong>{connection.displayName}</strong>
          </div>
          <div className="summary-box">
            <span className="summary-box__label">Server Address</span>
            <strong>{connection.serverAddress}</strong>
          </div>
          <div className="summary-box">
            <span className="summary-box__label">Sessions</span>
            <strong>{sessionCountLabel}</strong>
          </div>
        </section>

        <section className="dashboard-grid">
          <DashboardCard
            title="Create a Session"
            description="Create a new session and become its owner."
          >
            <div className="dashboard-form">
              <input
                type="text"
                placeholder="Enter session name"
                value={newSessionName}
                onChange={(e) => setNewSessionName(e.target.value)}
              />
              <button className="primary-button" onClick={handleCreateSession}>
                Create Session
              </button>
            </div>
          </DashboardCard>

          <DashboardCard
            title="Join with Session Key"
            description="Enter a key provided by another user to join a session."
          >
            <div className="dashboard-form">
              <input
                type="text"
                placeholder="Enter session key"
                value={sessionKeyInput}
                onChange={(e) => setSessionKeyInput(e.target.value)}
              />
              <button className="primary-button" onClick={handleJoinSessionByKey}>
                Join Session
              </button>
            </div>
          </DashboardCard>

          <DashboardCard
            title="Change User Name"
            description="Update the display name shown in your sessions."
          >
            <div className="dashboard-form">
              <input
                type="text"
                placeholder="Enter new user name"
                value={newUserName}
                onChange={(e) => setNewUserName(e.target.value)}
              />
              <button className="primary-button" onClick={handleChangeUserName}>
                Save Name
              </button>
            </div>
          </DashboardCard>

          <DashboardCard
            title="Joined Sessions"
            description="View sessions you created or joined."
          >
            <div className="session-list">
              {joinedSessions.length === 0 ? (
                <div className="empty-state">No sessions yet. Create one or join with a key.</div>
              ) : (
                joinedSessions.map((session) => (
                  <div className="session-list__item" key={session.id}>
                    <div>
                      <strong>{session.name}</strong>
                      <span>Key: {session.sessionKey}</span>
                    </div>
                    <div className="session-role">{session.role}</div>
                  </div>
                ))
              )}
            </div>
          </DashboardCard>
        </section>

        {feedback && <div className="status-box dashboard-status">{feedback}</div>}
      </div>
    </main>
  );
}

export default DashboardPage;