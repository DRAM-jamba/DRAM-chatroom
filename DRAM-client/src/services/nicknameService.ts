import { invoke } from "@tauri-apps/api/core";

export async function getSavedNickname(): Promise<string> {
  return await invoke<string>("get_nickname");
}

export async function saveNickname(nickname: string): Promise<void> {
  await invoke<void>("save_nickname", { nickname });
}

export async function submitNickname(nickname: string): Promise<void> {
  await invoke("set_nickname", { newNickname: nickname });
}

export async function updateNickname(nickname: string): Promise<void> {
  await submitNickname(nickname);
}