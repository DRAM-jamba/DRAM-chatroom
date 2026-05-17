import { useEffect, useRef } from "react";
import type { Message } from "../types/message";

type MessageListProps = {
  messages: Message[];
  currentUsername: string;
};

function MessageList({ messages, currentUsername }: MessageListProps) {
  const listRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = listRef.current;
    if (!el) return;
    el.scrollTop = el.scrollHeight;
  }, [messages]);

  const items: Array<{ type: "separator"; date: string } | { type: "message"; message: Message; showAuthor: boolean }> = [];

  let lastDate = "";
  let lastAuthor = "";

  for (const msg of messages) {
    if (msg.date !== lastDate) {
      items.push({ type: "separator", date: msg.date });
      lastDate = msg.date;
      lastAuthor = "";
    }
    const isSystem = msg.authorUsername === "";

    items.push({
      type: "message",
      message: msg,
      showAuthor: !isSystem && msg.authorUsername !== lastAuthor,
    });

    lastAuthor = msg.authorUsername;
  }

  return (
    <div className="message-list" ref={listRef}>
      {items.map((item, i) => {
        if (item.type === "separator") {
          return (
            <div key={`sep-${i}`} className="date-separator">
              <span className="date-separator-line" />
              <span className="date-separator-text">{item.date}</span>
              <span className="date-separator-line" />
            </div>
          );
        }

        const { message, showAuthor } = item;
        const isOwn = message.authorUsername === currentUsername;
        const isSystem = message.authorUsername === "";

        return (
          <div 
            key={message.id} 
            className={`message-item ${showAuthor ? "message-item-with-author" : ""} ${isSystem ? "message-system" : ""}`}
          >
            {showAuthor && (
              <div className="message-author-row">
                <span className={`message-author ${isOwn ? "message-author-own" : "message-author-other"}`}>
                  {message.authorUsername}
                </span>
                <span className="message-time">{message.timestamp}</span>
              </div>
            )}
            <p className="message-content">{message.content}</p>
          </div>
        );
      })}
    </div>
  );
}

export default MessageList;