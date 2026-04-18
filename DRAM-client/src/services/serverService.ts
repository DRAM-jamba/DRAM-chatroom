import type { Server } from "../types/server";

// Temporary data for testing before connecting to Rust
let temporaryServers: Server[] = [
  {
    id: "1",
    name: "servername",
    ipAddress: "87.247.64.20:1408",
  },
  {
    id: "2",
    name: "servername2",
    ipAddress: "87.247.64.21:1408",
  },
  {
    id: "3",
    name: "servername3",
    ipAddress: "87.247.64.22:1408",
  },
];

export async function getServers(): Promise<Server[]> {
  // Example later when fetching from Rust/Tauri:
  // import { invoke } from "@tauri-apps/api/core";
  // const servers = await invoke<Server[]>("get_servers");
  // return servers;

  return Promise.resolve([...temporaryServers]);
}

export async function addServer(data: {
  name: string;
  ipAddress: string;
}): Promise<Server> {
  // Example later when sending to Rust/Tauri:
  // import { invoke } from "@tauri-apps/api/core";
  // const newServer = await invoke<Server>("add_server", { data });
  // return newServer;

  const newServer: Server = {
    id: Date.now().toString(),
    name: data.name,
    ipAddress: data.ipAddress,
  };

  temporaryServers.push(newServer);
  return Promise.resolve(newServer);
}

export async function updateServer(
  id: string,
  data: { name: string; ipAddress: string }
): Promise<Server> {
  // Example later when updating in Rust/Tauri:
  // import { invoke } from "@tauri-apps/api/core";
  // const updatedServer = await invoke<Server>("update_server", { id, name: data.name });
  // return updatedServer;

  const server = temporaryServers.find((item) => item.id === id);

  if (!server) {
    throw new Error("Server not found");
  }

  // Only the name is editable — IP address remains unchanged
  server.name = data.name;

  return Promise.resolve({ ...server });
}

export async function removeServer(id: string): Promise<void> {
  // Example later when deleting in Rust/Tauri:
  // import { invoke } from "@tauri-apps/api/core";
  // await invoke("remove_server", { id });

  temporaryServers = temporaryServers.filter((item) => item.id !== id);
  return Promise.resolve();
}
