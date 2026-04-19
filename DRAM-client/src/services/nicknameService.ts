import { invoke } from "@tauri-apps/api/core";

export async function submitNickname(nickname: string): Promise<void> {
  await invoke("set_nickname", { newNickname: nickname });
}

export async function updateNickname(nickname: string): Promise<void> {
  await submitNickname(nickname);
}