import { useEffect, useRef, useState } from "react";
import MessageList from "../components/MessageList";
import MessageInput from "../components/MessageInput";
import SettingsPage from "./SettingsPage";
import { joinSession } from "../services/sessionService";
import {
  sendMessage,
  leaveSession,
  subscribeToMessages,
  subscribeToMemberUpdates,
  subscribeToMemberEvents,
} from "../services/chatService";
import { 
  joinVoiceChat, 
  leaveVoiceChat, 
  setMicMuted, 
  setDeafened as setServiceDeafened, 
  subscribeToVoiceList,
} from "../services/voiceChatService.ts";
import { loadMicHotkey, loadHeadphonesHotkey } from "../services/settingsService";
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
  const [voiceMembers, setVoiceMembers] = useState<string[]>([]);
  
  const [showHelp, setShowHelp] = useState(false);
  const [copied, setCopied] = useState(false);
  const copyTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const [showSettings, setShowSettings] = useState(false);

  const [isInVoiceCall, setIsInVoiceCall] = useState(false);
  const [muted, setMuted] = useState(false);
  const [deafened, setDeafened] = useState(false);

  const appendMessage = (msg: Message) => {
    setMessages((prev) => {
      if (prev.some((m) => m.id === msg.id)) return prev;
      return [...prev, msg];
    });
  };

  useEffect(() => {
    if (!sessionName) {
      console.warn("ChatPage mounted without a sessionName (key). Waiting...");
      return;
    }

    let unlistenFuncs: Array<() => void> = [];
    let isMounted = true;

    const setupChat = async () => {
      try {
        const unlistenMsgs = await subscribeToMessages(appendMessage);
        const unlistenUserEvents = await subscribeToMemberEvents(appendMessage);
        const unlistenMembers = await subscribeToMemberUpdates(setMembers);
        const unlistenVoice = await subscribeToVoiceList(setVoiceMembers);

        if (!isMounted) return;
        unlistenFuncs = [unlistenMsgs, unlistenUserEvents, unlistenMembers, unlistenVoice];
        
        await joinSession(sessionName);
      } catch (err) {
        if (isMounted) {
          console.error("Failed to connect to session:", err);
        }
      }
    };

    setupChat();

    return () => {
      isMounted = false;
      unlistenFuncs.forEach((unlisten) => unlisten());
      leaveVoiceChat().catch(console.error);
    };
  }, [sessionName]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const micKey = loadMicHotkey();
      const headphonesKey = loadHeadphonesHotkey();

      if (micKey && e.key.toUpperCase() === micKey) {
        setMuted((prev) => {
          const next = !prev;
          if (!next) setHidden(false);
          return next;
        });
      }

      if (headphonesKey && e.key.toUpperCase() === headphonesKey) {
        setHidden((prev) => {
          const next = !prev;
          if (next) setMuted(true);
          return next;
        });
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, []);

  const handleSend = async (content: string) => {
    await sendMessage(content);
  };

  const handleLeaveSession = async () => {
    await leaveVoiceChat();
    await leaveSession();
    onLeaveSession();
  };

  const handleCopySessionKey = async () => {
    await navigator.clipboard.writeText(sessionName);
    setCopied(true);
    if (copyTimeoutRef.current) clearTimeout(copyTimeoutRef.current);
    copyTimeoutRef.current = setTimeout(() => setCopied(false), 2000);
  };

  const handleToggleCall = async () => {
    if (isInVoiceCall) {
      await leaveVoiceChat();
      setIsInVoiceCall(false);
      setVoiceMembers([]);
    } else {
      await joinVoiceChat(sessionName);
      setIsInVoiceCall(true);
      await setMicMuted(muted);
      await setServiceDeafened(deafened);
    }
  };

  const handleToggleMute = async () => {
    const nextMuted = !muted;
    setMuted(nextMuted);
    if (!nextMuted && deafened) {
      setDeafened(false);
      if (isInVoiceCall) await setServiceDeafened(false);
    }
    
    if (isInVoiceCall) await setMicMuted(nextMuted);
  };

  const handleToggleDeafen = async () => {
    const nextDeafened = !deafened;
    setDeafened(nextDeafened);
    if (nextDeafened && !muted) {
      setMuted(true);
      if (isInVoiceCall) await setMicMuted(true);
    }
    
    if (isInVoiceCall) await setServiceDeafened(nextDeafened);
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100vh" }}>
      <TitleBar showMaximize />
      <div className="chat-page" style={{ flex: 1, minHeight: 0 }}>

        {/* Left sidebar */}
        <aside className="chat-left-sidebar">
          <div className="chat-logo-row">
            <img src={logoIcon} width="18" height="18" />
            <span className="chat-logo-text">quorthon</span>
          </div>

          <div className="voicechat-members-sidebar">
            <div className="chat-members-header">voice members</div>

            <div className="voice-members-list">
              {voiceMembers.length > 0 ? (
                voiceMembers.map((username) => (
                  <div key={`voice-${username}`} className="voice-member-card">
                    <span className="voice-indicator-dot" />
                    <span className="voice-member-username">{username}</span>
                  </div>
                ))
              ) : (
                <div className="members-empty">empty</div>
              )}
            </div>
          </div>

          <div className="chat-left-bottom">
            <div className="chat-left-actions">
              <button
                className={`chat-small-btn ${muted ? "active" : ""}`}
                type="button"
                onClick={handleToggleMute}
                title="Mute/Unmute"
              >
                <img src={muted ? micOffIcon : micIcon} width="16" height="16" className={muted ? "" : "icon-img"} />
              </button>

              <button
                className={`chat-small-btn ${deafened ? "active" : ""}`}
                type="button"
                onClick={handleToggleDeafen}
                title="Deafen/Undeafen"
              >
                <img src={hidden ? headphonesOffIcon : headphonesIcon} width="16" height="16" className={hidden ? "" : "icon-img"} />
              </button>

              <button className="chat-settings-btn" type="button" onClick={() => setShowSettings(true)}>
                <img src={settingsIcon} width="16" height="16" />
              </button>

              <button
                className="leave-session-btn"
                type="button"
                onClick={handleLeaveSession}
                title="Leave Session"
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

          {members.length > 0 && (
            <>
              {members.map((member) => (
                <div key={member.username} className="member-card">
                  {member.username}
                </div>
              ))}
            </>
          )}

          {members.length === 0 && (
            <div className="members-empty">empty</div>
          )}

          <div className="chat-members-bottom">
            <button 
              className={`call-btn ${isInVoiceCall ? "active" : ""}`} 
              type="button"
              onClick={handleToggleCall}
            >
              {isInVoiceCall ? "leave call" : "call"}
            </button>
          </div>
        </aside>
      </div>
      {showSettings && (
        <div className="settings-modal-overlay" onClick={() => setShowSettings(false)}>
          <div className="settings-modal" onClick={(e) => e.stopPropagation()}>
            <SettingsPage onBack={() => setShowSettings(false)} hideHeader />
          </div>
        </div>
      )}
    </div>
  );
}

export default ChatPage;