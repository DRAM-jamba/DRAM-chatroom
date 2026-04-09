import { useState } from "react";
import ServersPage from "./pages/ServersPage";
import NicknamePage from "./pages/NicknamePage";
import SessionsPage from "./pages/SessionsPage";
import ChatPage from "./pages/ChatPage";
import "./App.css";

type Page =
  | { name: "servers" }
  | { name: "nickname" }
  | { name: "sessions"; nickname: string }
  | { name: "chat"; sessionName: string; nickname: string };

function App() {
  const [page, setPage] = useState<Page>({ name: "servers" });

  if (page.name === "chat") {
    return (
      <ChatPage
        sessionName={page.sessionName}
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

  return <ServersPage onOpenSessions={() => setPage({ name: "nickname" })} />;
}

export default App;