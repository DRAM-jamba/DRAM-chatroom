import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Message, Member } from "../types/message";

export type MessageType =
  | "message"
  | "connect"
  | "disconnect"
  | "userlist"
  | "voicelist"
  | "voicestart"
  | "voiceend";

export type MessageObj = {
  m_type: MessageType;
  from: string;
  body: string;
  ts: number;
};

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

export async function sendMessage(content: string): Promise<void> {
  await invoke("send_message", { body: content });
}

export async function leaveSession(): Promise<void> {
  await invoke("leave_session");
}

function mapToUiMessage(p: MessageObj): Message {
  switch (p.m_type) {
    case "connect": {
      return {
        authorUsername: "",
        content: `${p.from} joined the session`,
        timestamp: formatTimestamp(p.ts),
        date: formatDate(p.ts),
        id: `${p.from}-${p.ts}-${p.m_type}`,
      };
    }
    case "disconnect": {
      return {
        authorUsername: "",
        content: `${p.from} left the session`,
        timestamp: formatTimestamp(p.ts),
        date: formatDate(p.ts),
        id: `${p.from}-${p.ts}-${p.m_type}`,
      };
    }
    case "message": {
      return {
        authorUsername: p.from,
        content: p.body,
        timestamp: formatTimestamp(p.ts),
        date: formatDate(p.ts),
        id: `${p.from}-${p.ts}-${p.m_type}`,
      };
    }
    default: {
      return {
        authorUsername: p.from,
        content: p.body,
        timestamp: formatTimestamp(p.ts),
        date: formatDate(p.ts),
        id: `${p.from}-${p.ts}-${p.m_type}`,
      };
    }
  }
}

export async function subscribeToMessages(
  onMessage: (msg: Message) => void
): Promise<() => void> {
  return await listen<MessageObj>("message", (event) => {
    console.log(event.payload);
    onMessage(mapToUiMessage(event.payload));
  });
}

export async function subscribeToMemberUpdates(
  onUpdate: (members: Member[]) => void
): Promise<() => void> {
  return await listen<MessageObj>("session_update", (event) => {
    console.log(event.payload);
    try {
      const usernames: string[] = JSON.parse(event.payload.body);
      const members: Member[] = usernames.map((username) => ({
        username,
      }));
      onUpdate(members);
    } catch (e) {
      console.error("Failed to parse member list from session_update", e);
    }
  });
}

export async function subscribeToMemberEvents(
  onMessage: (msg: Message) => void
): Promise<() => void> {
  return await listen<MessageObj>("session_update", (event) => {
    console.log(event.payload);
    onMessage(mapToUiMessage(event.payload));
  });
}