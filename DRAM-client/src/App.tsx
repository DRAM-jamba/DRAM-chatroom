import { useState } from "react";
import "./App.css";
import JoinServerPage from "./pages/JoinServerPage";
import DashboardPage from "./pages/DashboardPage";
import { ConnectionData, SessionItem } from "./types/app";

function App() {
  const [connection, setConnection] = useState<ConnectionData | null>(null);
  const [joinedSessions, setJoinedSessions] = useState<SessionItem[]>([]);

  function handleJoinServer(data: ConnectionData) {
    setConnection(data);
  }

  function handleBackToJoin() {
    setConnection(null);
  }

  function handleChangeUserName(newName: string) {
    setConnection((previous) => {
      if (!previous) return previous;
      return { ...previous, displayName: newName };
    });
  }

  function handleCreateSession(sessionName: string) {
    const newSession: SessionItem = {
      id: crypto.randomUUID(),
      name: sessionName,
      sessionKey: Math.random().toString(36).slice(2, 8).toUpperCase(),
      role: "Owner",
    };

    setJoinedSessions((previous) => [newSession, ...previous]);
  }

  function handleJoinSessionByKey(sessionKey: string) {
    const newSession: SessionItem = {
      id: crypto.randomUUID(),
      name: `Session ${sessionKey}`,
      sessionKey: sessionKey.toUpperCase(),
      role: "Member",
    };

    setJoinedSessions((previous) => [newSession, ...previous]);
  }

  if (!connection) {
    return <JoinServerPage onJoinServer={handleJoinServer} />;
  }

  return (
    <DashboardPage
      connection={connection}
      joinedSessions={joinedSessions}
      onBackToJoin={handleBackToJoin}
      onChangeUserName={handleChangeUserName}
      onCreateSession={handleCreateSession}
      onJoinSessionByKey={handleJoinSessionByKey}
    />
  );
}

export default App;