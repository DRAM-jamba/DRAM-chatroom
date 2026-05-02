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
  return d.toLocaleDateString("en-GB"); // dd/mm/yyyy
}

function payloadToMessage(p: MessagePayload): Message {
  return {
    id: `${p.from}-${p.ts}-${Math.random()}`,
    authorUsername: p.from,
    content: p.body,
    timestamp: formatTimestamp(p.ts),
    date: formatDate(p.ts),
  };
}

export async function getMessages(): Promise<Message[]> {
  return new Promise((resolve) => {
    // TODO: listen<SessionPayload>("session_update", (event) => {
    //   const messages = event.payload.chat_log.map(payloadToMessage);
    //   resolve(messages);
    // }).then((unlisten) => {
    //   // Clean up the one-shot listener after it fires
    //   listen<SessionPayload>("session_update", () => {
    //     unlisten();
    //   });
    // });
    resolve([]);
  });
}


export async function getMembers(): Promise<Member[]> {
  return new Promise((resolve) => {
    // TODO: listen<SessionPayload>("session_update", (event) => {
    //   const members: Member[] = event.payload.participants.map((username) => ({
    //     username,
    //     online: true,
    //   }));
    //   resolve(members);
    // }).then((unlisten) => {
    //   listen<SessionPayload>("session_update", () => {
    //     unlisten();
    //   });
    // });
    resolve([]);
  });
}

export async function sendMessage(
  content: string,
  currentUsername: string
): Promise<Message> {
  // TODO: await invoke<void>("send_message", { body: content });

  const now = Math.floor(Date.now() / 1000);
  return {
    id: `${currentUsername}-${now}-${Math.random()}`,
    authorUsername: currentUsername,
    content,
    timestamp: formatTimestamp(now),
    date: "Today",
  };
}

export async function leaveSession(): Promise<void> {
  // TODO: await invoke<void>("leave_session");
}

export async function subscribeToMessages(
  onMessage: (msg: Message) => void
): Promise<() => void> {
  // TODO: const unlisten = await listen<string>("message", (event) => {
  //   const raw = event.payload;
  //   const colonIndex = raw.indexOf(": ");
  //
  //   let authorUsername: string;
  //   let content: string;
  //
  //   if (colonIndex !== -1) {
  //     authorUsername = raw.slice(0, colonIndex);
  //     content = raw.slice(colonIndex + 2);
  //   } else {
  //     authorUsername = "unknown";
  //     content = raw;
  //   }
  //
  //   const now = Math.floor(Date.now() / 1000);
  //   const msg: Message = {
  //     id: `${authorUsername}-${now}-${Math.random()}`,
  //     authorUsername,
  //     content,
  //     timestamp: formatTimestamp(now),
  //     date: "Today",
  //   };
  //
  //   onMessage(msg);
  // });
  //
  // return unlisten;
  return () => { };
}