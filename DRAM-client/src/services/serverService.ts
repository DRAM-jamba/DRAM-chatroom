import { invoke } from "@tauri-apps/api/core";
import type { Server } from "../types/server";

// ─── getServers ──────────────────────────────────────────────────────────────
// Returns the list of servers the user has previously added.
// Backed by Tauri's persisted store (servers.json).

export async function getServers(): Promise<Server[]> {
  return await invoke<Server[]>("get_servers");
}

// ─── addServer ───────────────────────────────────────────────────────────────
// Registers the client with a new server using the given IP and nickname.
// The server responds with a user_key which Tauri persists for future connects.
// Corresponds to the `add` Tauri command in lib.rs.

export async function addServer(data: {
  nickname: string;
  ip: string;
}): Promise<void> {
  await invoke("add", {
    ip: data.ip,
    nickname: data.nickname,
  });
}

// ─── connectServer ───────────────────────────────────────────────────────────
// Establishes an active connection to a known server using its persisted user_key.
// Called when the user clicks "connect" on a server card.
// Corresponds to the `connect` Tauri command in lib.rs.

export async function connectServer(ip: string): Promise<void> {
  await invoke("connect", { ip });
}

// ─── updateServer ────────────────────────────────────────────────────────────
// Renames a server locally. Only the display nickname can be changed —
// the IP and user_key are immutable after registration.
// Note: there is no Tauri command for this yet; the rename is kept in-memory
// until a `rename_server` command is added to lib.rs.

export async function updateServer(
  ip: string,
  data: { nickname: string }
): Promise<Server> {
  // Wire up to Rust when the command is available:
  // await invoke("rename_server", { ip, nickname: data.nickname });

  // Temporary: reflect the rename optimistically on the frontend only
  return Promise.resolve({ ip, nickname: data.nickname, user_key: "" });
}

// ─── removeServer ────────────────────────────────────────────────────────────
// Removes a server from the persisted list.
// Corresponds to the `remove_server` Tauri command in lib.rs.

export async function removeServer(ip: string): Promise<void> {
  await invoke("remove_server", { ip });
}
