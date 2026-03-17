import { useState } from "react";
import ConnectPage from "./pages/ConnectPage";
import WelcomePage from "./pages/WelcomePage";

function App() {
  const [currentPage, setCurrentPage] = useState<"connect" | "welcome">(
    "connect"
  );
  const [nickname, setNickname] = useState("");
  const [ipAddress, setIpAddress] = useState("");

  const handleConnect = (ip: string, name: string) => {
    setIpAddress(ip);
    setNickname(name);
    setCurrentPage("welcome");
  };

  return (
    <>
      {currentPage === "connect" ? (
        <ConnectPage onSubmit={handleConnect} />
      ) : (
        <WelcomePage nickname={nickname} />
      )}
    </>
  );
}

export default App;