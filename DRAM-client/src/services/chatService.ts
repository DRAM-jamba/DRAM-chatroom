import type { Message, Member } from "../types/message";

// ---------------------------------------------------------------------------
// Temporary data — replace with Rust/Tauri invoke calls when ready
// ---------------------------------------------------------------------------

const temporaryMessages: Message[] = [
  {
    id: "1",
    authorUsername: "username1",
    content: "this is suuuuuuuuuuuuuuuuuuuuuuuuuuuper long message",
    timestamp: "14:08",
    date: "28/03/2026",
  },
  {
    id: "2",
    authorUsername: "username1",
    content: "this is suuuuuuuuuuuuuuuuuuuuuuuuuuuper long message",
    timestamp: "14:08",
    date: "28/03/2026",
  },
  {
    id: "3",
    authorUsername: "username2",
    content: "wow what a long message you sent",
    timestamp: "14:08",
    date: "28/03/2026",
  },
  {
    id: "4",
    authorUsername: "username2",
    content: "hello session",
    timestamp: "14:08",
    date: "Today",
  },
];

const temporaryMembers: Member[] = [
  { username: "username1", online: true },
  { username: "username2", online: true },
  { username: "username3", online: true },
  { username: "username4", online: true },
  { username: "username5", online: false },
  { username: "username6", online: false },
];

// ---------------------------------------------------------------------------
// Service functions
// ---------------------------------------------------------------------------

export async function getMessages(): Promise<Message[]> {
  // TODO: wire up to Rust
  // import { invoke } from "@tauri-apps/api/core";
  // return await invoke<Message[]>("get_messages", { sessionId });
  return Promise.resolve([...temporaryMessages]);
}

export async function sendMessage(content: string): Promise<Message> {
  // TODO: wire up to Rust
  // import { invoke } from "@tauri-apps/api/core";
  // return await invoke<Message>("send_message", { content, sessionId });
  const newMessage: Message = {
    id: Date.now().toString(),
    authorUsername: "your username",
    content,
    timestamp: new Date().toLocaleTimeString("en-GB", {
      hour: "2-digit",
      minute: "2-digit",
    }),
    date: "Today",
  };
  temporaryMessages.push(newMessage);
  return Promise.resolve(newMessage);
}

export async function getMembers(): Promise<Member[]> {
  // TODO: wire up to Rust
  // import { invoke } from "@tauri-apps/api/core";
  // return await invoke<Member[]>("get_members", { sessionId });
  return Promise.resolve([...temporaryMembers]);
}
