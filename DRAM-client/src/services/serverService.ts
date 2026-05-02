import { invoke } from "@tauri-apps/api/core";
import type { Server } from "../types/server";


export async function getServers(): Promise<Server[]> {
  return await invoke<Server[]>("get_servers");
}

export async function addServer(data: {
  nickname: string;
  ip: string;
}): Promise<void> {
  await invoke("add", {
    ip: data.ip.trim(),
    nickname: data.nickname.trim(),
  });
}

export async function connectServer(ip: string): Promise<void> {
  await invoke("connect", { ip });
}

export async function updateServer(
  ip: string,
  data: { nickname: string }
): Promise<Server> {
  // Wire up to Rust when the command is available:
  // await invoke("rename_server", { ip, nickname: data.nickname });

  // Temporary: reflect the rename optimistically on the frontend only
  return Promise.resolve({ id: "", ipAddress: ip, name: data.nickname, user_key: "" });
}

export async function removeServer(id: string): Promise<void> {
  await invoke("remove_server", { id });
}