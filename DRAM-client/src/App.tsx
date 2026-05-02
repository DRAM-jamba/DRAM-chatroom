import { useState } from "react";
import ServersPage from "./pages/ServersPage";
import NicknamePage from "./pages/NicknamePage";
import SessionsPage from "./pages/SessionsPage";
import ChatPage from "./pages/ChatPage";
import { getSavedNickname } from "./services/nicknameService";
import "./App.css";

type Page =
  | { name: "servers" }
  | { name: "nickname" }
  | { name: "sessions"; nickname: string }
  | { name: "chat"; sessionName: string; nickname: string };

function App() {
  const [page, setPage] = useState<Page>({ name: "servers" });
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
        onConnectToSession={(sessionName) =>
          setPage({ name: "chat", sessionName, nickname: page.nickname })
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

  return <ServersPage onOpenSessions={handleServerConnected} />;
}

export default App;