import { useEffect, useRef, useState } from "react";
import MessageList from "../components/MessageList";
import MessageInput from "../components/MessageInput";
import {
  getMessages,
  getMembers,
  sendMessage,
  leaveSession,
  subscribeToMessages,
  subscribeToMembers,
  subscribeToUserEvents,
} from "../services/chatService";
import type { Message, Member } from "../types/message";
import TitleBar from "../components/TitleBar";
import micIcon from "../assets/icons/micbtnicon.svg";
import micOffIcon from "../assets/icons/micoffbtnicon.svg";
import headphonesIcon from "../assets/icons/headphonesbtnicon.svg";
import headphonesOffIcon from "../assets/icons/headphonesoffbtnicon.svg";
import settingsIcon from "../assets/icons/settingbtnicon.svg";
import exitIcon from "../assets/icons/exitbtnicon.svg";
import logoIcon from "../assets/icons/logorgb.png";

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
  const [copied, setCopied] = useState(false);
  const copyTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Helper to append a message deduplicating by id
  const appendMessage = (msg: Message) => {
    setMessages((prev) => {
      if (prev.some((m) => m.id === msg.id)) return prev;
      return [...prev, msg];
    });
  };

  useEffect(() => {
    // Load initial messages and members from the session_update event
    getMessages().then((initialMsgs) => {
      if (initialMsgs.length > 0) {
        setMessages(initialMsgs);
      }
    });

    getMembers().then((initialMembers) => {
      if (initialMembers.length > 0) {
        setMembers(initialMembers);
      }
    });

    // Subscribe to new incoming chat messages
    const unlistenMsgPromise = subscribeToMessages(appendMessage);

    // Subscribe to join/leave system messages
    const unlistenUserEventsPromise = subscribeToUserEvents(appendMessage);

    // Subscribe to member list updates (join/leave events)
    const unlistenMembersPromise = subscribeToMembers((updatedMembers) => {
      setMembers(updatedMembers);
    });

    return () => {
      unlistenMsgPromise.then((unlisten) => unlisten());
      unlistenUserEventsPromise.then((unlisten) => unlisten());
      unlistenMembersPromise.then((unlisten) => unlisten());
    };
  }, []);

  const handleSend = async (content: string) => {
    await sendMessage(content);
  };

  const handleLeaveSession = async () => {
    await leaveSession();
    onLeaveSession();
  };

  const handleCopySessionKey = async () => {
    try {
      await navigator.clipboard.writeText(sessionName);
      setCopied(true);
      if (copyTimeoutRef.current) clearTimeout(copyTimeoutRef.current);
      copyTimeoutRef.current = setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      console.error("Failed to copy session key:", error);
    }
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
            <img src={logoIcon} width="18" height="18" />
            <span className="chat-logo-text">quorthon</span>
            <span className="chat-version">ver. 0.69</span>
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
                <img src={muted ? micOffIcon : micIcon} width="16" height="16" />
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
                <img src={hidden ? headphonesOffIcon : headphonesIcon} width="16" height="16" />
              </button>

              <button className="chat-settings-btn" type="button">
                <img src={settingsIcon} width="16" height="16" />
              </button>

              <button
                className="leave-session-btn"
                type="button"
                onClick={handleLeaveSession}
              >
                <img src={exitIcon} width="16" height="16" />
              </button>
            </div>
          </div>
        </aside>

        {/* Main chat area */}
        <main className="chat-main">
          <div className="chat-topbar">
            <div className="chat-session-key-wrapper">
              <span className="chat-session-key-label">session key</span>
              <button
                className="chat-session-name"
                type="button"
                onClick={handleCopySessionKey}
                title="Click to copy session key"
              >
                {sessionName}
              </button>
            </div>

            {/* Copied toast */}
            {copied && (
              <div className="copy-toast">
                session key copied!
              </div>
            )}

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

          {onlineMembers.length === 0 && offlineMembers.length === 0 && (
            <div className="members-empty">empty</div>
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