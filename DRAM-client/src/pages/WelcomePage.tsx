// useState used to store and update data (state)
import { useState } from "react";

export default function WelcomePage() {
  // State to store the session key entered by the user
  const [sessionKey, setSessionKey] = useState("");

  // State to store the name of a new session
  const [newSessionName, setNewSessionName] = useState("");

  // Function called when user clicks "Join"
  const handleJoin = () => {
    console.log("Join with key:", sessionKey);
  };

  // Function called when user clicks "Create"
  const handleCreate = () => {
    console.log("Create session:", newSessionName);
  };

  return (
    <div>

      {/* Panel to Join Session */}
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

      {/* Panel to Create Session */}
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

    </div>
  );
}