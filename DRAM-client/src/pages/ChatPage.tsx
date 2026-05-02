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

type ChatPageProps = {
  sessionName: string;
  nickname: string;
  onLeaveSession: () => void;
};

function ChatPage({ sessionName, nickname, onLeaveSession }: ChatPageProps) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [members, setMembers] = useState<Member[]>([]);
  const [showHelp, setShowHelp] = useState(false);

  useEffect(() => {
    // Load initial messages and members from the session_update event
    // that the ClientBackend emits right after connect_session() completes.
    getMessages().then(setMessages);
    getMembers().then(setMembers);

    // Subscribe to real-time incoming messages over WebSocket.
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

  const handleSend = async (content: string) => {
    const newMessage = await sendMessage(content, nickname);
    setMessages((prev) => [...prev, newMessage]);
  };

  const handleLeaveSession = async () => {
    await leaveSession();
    onLeaveSession();
  };

  const onlineMembers = members.filter((m) => m.online);
  const offlineMembers = members.filter((m) => !m.online);

  return (
    <div className="chat-page">
      {/* Left sidebar */}
      <aside className="chat-left-sidebar">
        <div className="chat-logo-row">
          <span className="chat-logo-text">quorthon</span>
          <span className="chat-version">ver. 0.2</span>
        </div>

        <div className="chat-left-bottom">
          <button
            className="leave-session-btn"
            type="button"
            onClick={handleLeaveSession}
          >
            leave session
          </button>

          <div className="chat-left-actions">
            <button className="chat-small-btn" type="button">M</button>
            <button className="chat-small-btn" type="button">H</button>
            <button className="chat-settings-btn" type="button">settings</button>
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
  );
}

export default ChatPage;