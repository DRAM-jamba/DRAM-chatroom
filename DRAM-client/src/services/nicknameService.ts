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

export async function sendNickname(nickname: string): Promise<void> {
  await invoke("set_nickname", { new_nickname: nickname });
}