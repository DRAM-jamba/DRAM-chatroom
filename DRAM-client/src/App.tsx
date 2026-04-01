import { useState } from "react";
import ServersPage from "./pages/ServersPage";
// import SessionsPage from "./pages/SessionsPage";
// import ChatPage from "./pages/ChatPage";
import "./App.css";

type Page =
  | { name: "servers" }
  | { name: "sessions" }
  | { name: "chat"; sessionName: string };

function App() {
  const [page, setPage] = useState<Page>({ name: "servers" });

  // if (page.name === "chat") {
  //   return (
  //     <ChatPage
  //       sessionName={page.sessionName}
  //       onLeaveSession={() => setPage({ name: "sessions" })}
  //     />
  //   );
  // }

  // if (page.name === "sessions") {
  //   return (
  //     <SessionsPage
  //       onBackToServers={() => setPage({ name: "servers" })}
  //       onConnectToSession={(sessionName) =>
  //         setPage({ name: "chat", sessionName })
  //       }
  //     />
  //   );
  // }

  return <ServersPage onOpenSessions={() => setPage({ name: "sessions" })} />;
}

export default App;
