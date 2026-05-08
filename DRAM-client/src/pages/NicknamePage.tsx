import { useState } from "react";
import { submitNickname, saveNickname } from "../services/nicknameService";
import TitleBar from "../components/TitleBar";
import logoIcon from "../assets/icons/logorgb.png";

type NicknamePageProps = {
  onNicknameSet: (nickname: string) => void;
};

function NicknamePage({ onNicknameSet }: NicknamePageProps) {
  const [nickname, setNickname] = useState("");
  const [error, setError] = useState("");
  const [loading, setLoading] = useState(false);

  const handleSubmit = async () => {
    const trimmed = nickname.trim();
    if (!trimmed) {
      setError("Please enter a nickname.");
      return;
    }

    setLoading(true);
    setError("");

    try {
      // Send to the server so other users see the nickname immediately.
      await submitNickname(trimmed);
      // Persist locally so this page is skipped on every future launch.
      await saveNickname(trimmed);
      onNicknameSet(trimmed);
    } catch {
      setError("Failed to set nickname. Please try again.");
    } finally {
      setLoading(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      handleSubmit();
    }
  };

  return (
    <div className="servers-page">
  <TitleBar />
  <aside className="sidebar">
    <h1 className="logo">
          <img src={logoIcon} width="24" height="24" />
          quorthon
        </h1>
    <div className="sidebar-line" />

    <div className="nickname-card">
      <h2 className="nickname-title">Choose your nickname</h2>
      <p className="nickname-subtitle">
        This is how other users will see you on this server.
      </p>
      
      <div className="nickname-input-group">
        <input
          className="server-input nickname-input"
          value={nickname}
          onChange={(e) => {
            setNickname(e.target.value);
            if (error) setError("");
          }}
          onKeyDown={handleKeyDown}
          autoFocus
          maxLength={32}
        />

        <button
          className="big-confirm-btn nickname-confirm-btn"
          type="button"
          onClick={handleSubmit}
          disabled={loading}
        >
          {loading ? "connecting..." : "confirm"}
        </button>
      </div>

      {error && <p className="nickname-error">{error}</p>}
    </div>

    <div className="sidebar-line bottom-line" />
    <p className="version-text">ver. 0.2</p>
  </aside>
</div>
  );
}

export default NicknamePage;