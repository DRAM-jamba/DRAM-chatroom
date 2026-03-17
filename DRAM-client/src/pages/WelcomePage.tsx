import { useState } from "react";

type WelcomePageProps = {
  nickname: string;
};

export default function WelcomePage({ nickname }: WelcomePageProps) {
  const [sessionKey, setSessionKey] = useState("");
  const [newSessionName, setNewSessionName] = useState("");
  const [newNickname, setNewNickname] = useState("");

  const handleJoin = () => {
    console.log("Join with key:", sessionKey);
  };

  const handleCreate = () => {
    console.log("Create session:", newSessionName);
  };

  const handleChangeNickname = () => {
    console.log("New nickname:", newNickname);
  };

  return (
    <div>

      {/* Panel 1: Join Session */}
      <div>
        <h2>Join Session</h2>
        <input
          type="text"
          value={sessionKey}
          onChange={(e) => setSessionKey(e.target.value)}
        />
        <br />
        <button onClick={handleJoin}>Join</button>
      </div>

      <hr />

      {/* Panel 2: Create Session */}
      <div>
        <h2>Create Session</h2>
        <input
          type="text"
          value={newSessionName}
          onChange={(e) => setNewSessionName(e.target.value)}
        />
        <br />
        <button onClick={handleCreate}>Create</button>
      </div>

      <hr />

      {/* Panel 3: Change Nickname */}
      <div>
        <h2>Change Nickname</h2>
        <input
          type="text"
          value={newNickname}
          onChange={(e) => setNewNickname(e.target.value)}
        />
        <br />
        <button onClick={handleChangeNickname}>Change</button>
      </div>
    </div>
  );
}