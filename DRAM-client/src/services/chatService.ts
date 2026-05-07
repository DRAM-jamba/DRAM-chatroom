import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Message, Member } from "../types/message";

type MessagePayload = {
  from: string;
  body: string;
  ts: number;
};

type SessionPayload = {
  session_id: string;
  participants: string[];
  chat_log: MessagePayload[];
};

let cachedMessages: Message[] = [];
let cachedMembers: Member[] = [];
let sessionUpdateUnlisten: (() => void) | null = null;

(async () => {
  sessionUpdateUnlisten = await listen<SessionPayload>(
    "session_update",
    (event) => {
      const payload = event.payload;

      cachedMessages = payload.chat_log.map(payloadToMessage);

      cachedMembers = payload.participants.map((username) => ({
        username,
        online: true,
      }));

      // One-shot: stop listening after the first event.
      sessionUpdateUnlisten?.();
      sessionUpdateUnlisten = null;
    }
  );
})();


function formatTimestamp(ts: number): string {
  return new Date(ts * 1000).toLocaleTimeString("en-GB", {
    hour: "2-digit",
    minute: "2-digit",
  });
}

function formatDate(ts: number): string {
  const d = new Date(ts * 1000);
  const today = new Date();
  if (
    d.getDate() === today.getDate() &&
    d.getMonth() === today.getMonth() &&
    d.getFullYear() === today.getFullYear()
  ) {
    return "Today";
  }
  return d.toLocaleDateString("en-GB");
}

function payloadToMessage(p: MessagePayload): Message {
  return {
    authorUsername: p.from,
    content: p.body,
    timestamp: formatTimestamp(p.ts),
    date: formatDate(p.ts),
    id: `${p.from}-${p.ts}-${p.body}`,
  };
}

export async function getMessages(): Promise<Message[]> {
  if (cachedMessages.length > 0 || sessionUpdateUnlisten === null) {
    return cachedMessages;
  }

  return new Promise((resolve) => {
    const timeout = setTimeout(() => resolve([]), 3000);

    const origUnlisten = sessionUpdateUnlisten;
    sessionUpdateUnlisten = () => {
      origUnlisten?.();
      clearTimeout(timeout);
      resolve(cachedMessages);
    };
  });
}

export async function getMembers(): Promise<Member[]> {
  if (cachedMembers.length > 0 || sessionUpdateUnlisten === null) {
    return cachedMembers;
  }

  return new Promise((resolve) => {
    const timeout = setTimeout(() => resolve([]), 3000);

    const origUnlisten = sessionUpdateUnlisten;
    sessionUpdateUnlisten = () => {
      origUnlisten?.();
      clearTimeout(timeout);
      resolve(cachedMembers);
    };
  });
}

export async function sendMessage(content: string): Promise<void> {
  await invoke<void>("send_message", { body: content });
}

export async function leaveSession(): Promise<void> {
  await invoke<void>("leave_session");
  cachedMessages = [];
  cachedMembers = [];
}

/**
 * Subscribes to real-time messages arriving over the WebSocket connection.
 */
export async function subscribeToMessages(
  onMessage: (msg: Message) => void
): Promise<() => void> {
  return await listen<MessagePayload>("message", (event) => {
    const p = event.payload;

    const msg: Message = {
      authorUsername: p.from,
      content: p.body,
      timestamp: formatTimestamp(p.ts),
      date: formatDate(p.ts),
      id: `${p.from}-${p.ts}-${p.body}`,
    };

    onMessage(msg);
  });
}

export async function subscribeToMembers(
  onUpdate: (members: Member[]) => void
): Promise<() => void> {
  return await listen<SessionPayload>("session_update", (event) => {
    const members = event.payload.participants.map((username) => ({
      username,
      online: true,
    }));
    onUpdate(members);
  });
}