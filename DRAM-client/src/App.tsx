import { useState, useEffect } from "react";
import ServersPage from "./pages/ServersPage";
import NicknamePage from "./pages/NicknamePage";
import SessionsPage from "./pages/SessionsPage";
import ChatPage from "./pages/ChatPage";
import SettingsPage from "./pages/SettingsPage";
import { getSavedNickname } from "./services/nicknameService";
import { loadAllSettings } from "./services/settingsService";
import "./App.css";

type Page =
  | { name: "servers" }
  | { name: "nickname" }
  | { name: "sessions"; nickname: string }
  | { name: "chat"; sessionName: string; sessionKey: string; nickname: string }
  | { name: "settings"; nickname: string };

loadAllSettings();

function App() {
  const [page, setPage] = useState<Page>({ name: "servers" });
  useEffect(() => {
    navigator.mediaDevices.getUserMedia({ audio: true })
      .then((stream) => {
        stream.getTracks().forEach((track) => track.stop());
      })
      .catch(async () => {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("reset_mic_permission");
      });
  }, []);
  const handleServerConnected = async () => {
    const saved = await getSavedNickname();
    if (saved) {
      setPage({ name: "sessions", nickname: saved });
    } else {
      setPage({ name: "nickname" });
    }
  };

  if (page.name === "chat") {
    return (
      <ChatPage
        sessionName={page.sessionName}
        sessionKey={page.sessionKey}
        nickname={page.nickname}
        onLeaveSession={() =>
          setPage({ name: "sessions", nickname: page.nickname })
        }
      />
    );
  }

  if (page.name === "sessions") {
    return (
      <SessionsPage
        nickname={page.nickname}
        onDisconnect={() => setPage({ name: "servers" })}
        onNicknameChange={(newNickname) =>
          setPage({ name: "sessions", nickname: newNickname })
        }
        onConnectToSession={(sessionName, sessionKey) =>
          setPage({ name: "chat", sessionName, sessionKey, nickname: page.nickname })
        }
        onOpenSettings=
        {
          () => setPage({ name: "settings", nickname: page.nickname })
        }
      />
    );
  }

  if (page.name === "nickname") {
    return (
      <NicknamePage
        onNicknameSet={(nickname) => setPage({ name: "sessions", nickname })}
      />
    );
  }

  if (page.name === "settings") {
    return (
      <SettingsPage
        onBack={() => setPage({ name: "sessions", nickname: page.nickname })}
      />
    );
  }

  return <ServersPage onOpenSessions={handleServerConnected} />;
}

export default App;