// Temporary nickname store before connecting to Rust
let currentNickname = "";

export async function submitNickname(nickname: string): Promise<void> {
  // Example later when sending to Rust/Tauri:
  // import { invoke } from "@tauri-apps/api/core";
  // await invoke("send_nickname", { nickname });

  currentNickname = nickname;
  return Promise.resolve();
}

export async function updateNickname(nickname: string): Promise<void> {
  // Example later when updating in Rust/Tauri:
  // import { invoke } from "@tauri-apps/api/core";
  // await invoke("send_nickname", { nickname });

  currentNickname = nickname;
  return Promise.resolve();
}

export function getNickname(): string {
  return currentNickname;
}