import { invoke } from "@tauri-apps/api/core";

// Temporary nickname store before connecting to Rust
let currentNickname = "";

export async function submitNickname(nickname: string): Promise<void> {
  currentNickname = nickname;
  return Promise.resolve();
}

export async function updateNickname(nickname: string): Promise<void> {
  currentNickname = nickname;
  return Promise.resolve();
}

export function getNickname(): string {
  return currentNickname;
}

export async function sendNickname(new_nickname: string): Promise<void> {
  console.log("Sending nickname to Rust:", new_nickname);
  await invoke("set_nickname", { newNickname: new_nickname });
  currentNickname = new_nickname;
  return Promise.resolve();
}