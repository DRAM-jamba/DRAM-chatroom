import { useEffect, useState } from "react";
import MessageList from "../components/MessageList";
import MessageInput from "../components/MessageInput";
import { getMessages, sendMessage, getMembers } from "../services/chatService";
import type { Message, Member } from "../types/message";

type ChatPageProps = {
  sessionName: string;
  onLeaveSession: () => void;
};

// TODO: wire up to Rust — replace with real current user from auth/session context
// import { invoke } from "@tauri-apps/api/core";
// const currentUsername = await invoke<string>("get_current_username");
const CURRENT_USERNAME = "your username";

function ChatPage({ sessionName, onLeaveSession }: ChatPageProps) {
  const [messages, setMessages] = useState<Message[]>([]);
  const [members, setMembers] = useState<Member[]>([]);
  const [showHelp, setShowHelp] = useState(false);

  useEffect(() => {
    loadMessages();
    loadMembers();

    // TODO: wire up to Rust — subscribe to real-time message events
    // import { listen } from "@tauri-apps/api/event";
    // const unlisten = await listen<Message>("new_message", (event) => {
    //   setMessages((prev) => [...prev, event.payload]);
    // });
    // return () => unlisten();
  }, []);

  const loadMessages = async () => {
    // TODO: wire up to Rust
    // import { invoke } from "@tauri-apps/api/core";
    // const data = await invoke<Message[]>("get_messages", { sessionId });
    const data = await getMessages();
    setMessages(data);
  };

  const loadMembers = async () => {
    // TODO: wire up to Rust
    // import { invoke } from "@tauri-apps/api/core";
    // const data = await invoke<Member[]>("get_members", { sessionId });
    const data = await getMembers();
    setMembers(data);
  };

  const handleSend = async (content: string) => {
    // TODO: wire up to Rust
    // import { invoke } from "@tauri-apps/api/core";
    // const newMessage = await invoke<Message>("send_message", { content, sessionId });
    const newMessage = await sendMessage(content);
    setMessages((prev) => [...prev, newMessage]);
  };

  const handleLeaveSession = () => {
    // TODO: wire up to Rust — notify server of disconnect
    // import { invoke } from "@tauri-apps/api/core";
    // await invoke("leave_session", { sessionId });
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

        <MessageList messages={messages} currentUsername={CURRENT_USERNAME} />

        <MessageInput currentUsername={CURRENT_USERNAME} onSend={handleSend} />
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
