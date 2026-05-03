import { useEffect, useState } from "react";
import MessageList from "../components/MessageList";
import MessageInput from "../components/MessageInput";
import {
  getMessages,
  getMembers,
  sendMessage,
  leaveSession,
  subscribeToMessages,
} from "../services/chatService";
import type { Message, Member } from "../types/message";
import TitleBar from "../components/TitleBar";

type ChatPageProps = {
  sessionName: string;
  nickname: string;
  onLeaveSession: () => void;
};

function ChatPage({ sessionName, nickname, onLeaveSession }: ChatPageProps) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [members, setMembers] = useState<Member[]>([]);
  const [showHelp, setShowHelp] = useState(false);
  const [muted, setMuted] = useState(false);
  const [hidden, setHidden] = useState(false);

  useEffect(() => {
    getMessages().then(setMessages);
    getMembers().then(setMembers);

    let unlisten: (() => void) | undefined;
    subscribeToMessages((msg) => {
      setMessages((prev) => [...prev, msg]);
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  // Send the message text to the server. The echo will arrive via the
  // "message" WebSocket event and be appended by subscribeToMessages.
  const handleSend = async (content: string) => {
    await sendMessage(content);
  };

  const handleLeaveSession = async () => {
    await leaveSession();
    onLeaveSession();
  };

  const onlineMembers = members.filter((m) => m.online);
  const offlineMembers = members.filter((m) => !m.online);

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100vh" }}>
      <TitleBar showMaximize />
      <div className="chat-page" style={{ flex: 1, minHeight: 0 }}>
      {/* Left sidebar */}
      <aside className="chat-left-sidebar">
        <div className="chat-logo-row">
            <img src="/src/assets/icons/logorgb.png" width="18" height="18" />
          <span className="chat-logo-text">quorthon</span>
          <span className="chat-version">ver. 0.2</span>
        </div>

        <div className="chat-left-bottom">
            <div className="chat-left-actions">
              <button
                className={`chat-small-btn ${muted ? "active" : ""}`}
                type="button"
                onClick={() => {
                  setMuted(prev => {
                    const next = !prev;
                    if (!next) setHidden(false);
                    return next;
                  });
                }}
              >
                <img
                  src={muted ? "/src/assets/icons/micoffbtnicon.svg" : "/src/assets/icons/micbtnicon.svg"}
                  width="16"
                  height="16"
                />
              </button>

              <button
                className={`chat-small-btn ${hidden ? "active" : ""}`}
                type="button"
                onClick={() => {
                  setHidden(prev => {
                    const next = !prev;
                    if (next) setMuted(true);
                    return next;
                  });
                }}
              >
                <img
                  src={hidden ? "/src/assets/icons/headphonesoffbtnicon.svg" : "/src/assets/icons/headphonesbtnicon.svg"}
                  width="16"
                  height="16"
                />
              </button>

              <button className="chat-settings-btn" type="button">
                <img src="/src/assets/icons/settingbtnicon.svg" width="16" height="16" />
              </button>

              <button
            className="leave-session-btn"
            type="button"
            onClick={handleLeaveSession}
          >
                <img src="/src/assets/icons/exitbtnicon.svg" width="16" height="16" />
              </button>
          </div>
        </div>
      </aside>

      {/* Main chat area */}
      <main className="chat-main">
        <div className="chat-topbar">
          <span className="chat-session-name">{sessionName}</span>

          <div className="chat-help-wrapper">
            <button
              className="chat-help-btn"
              type="button"
              onClick={() => setShowHelp((prev) => !prev)}
            >
              ?
            </button>

            {showHelp && (
              <div className="chat-help-popup" onClick={() => setShowHelp(false)}>
                <div className="help-popup-content">
                  <p>• M — mute microphone</p>
                  <p>• H — hide / show camera</p>
                  <p>• Call — start a voice/video call</p>
                  <span className="help-popup-close-text">click to close</span>
                </div>
              </div>
            )}
          </div>
        </div>

        <MessageList messages={messages} currentUsername={nickname} />

        <MessageInput currentUsername={nickname} onSend={handleSend} />
      </main>

      {/* Right members sidebar */}
      <aside className="chat-members-sidebar">
        <div className="chat-members-header">members</div>

        {onlineMembers.length > 0 && (
          <>
            <div className="members-status-label">online</div>
            {onlineMembers.map((member) => (
              <div key={member.username} className="member-card">
                {member.username}
              </div>
            ))}
          </>
        )}

        {offlineMembers.length > 0 && (
          <>
            <div className="members-status-label members-status-offline">offline</div>
            {offlineMembers.map((member) => (
              <div key={member.username} className="member-card member-card-offline">
                {member.username}
              </div>
            ))}
          </>
        )}

        <div className="chat-members-bottom">
          <button className="call-btn" type="button">
            call
          </button>
        </div>
      </aside>
    </div>
    </div>
  );
}

export default ChatPage;