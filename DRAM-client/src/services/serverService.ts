import type { Server } from "../types/server";
import { invoke } from "@tauri-apps/api/core";
import { getNickname } from "./nicknameService";

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
  const servers = await invoke<Server[]>("get_servers");
  // return servers;

  return servers;
}

export async function addServer(data: {
  name: string;
  ipAddress: string;
}): Promise<Server> {
  // Call the Rust add command with correct parameter names
  // The Rust backend expects: ip and nickname
  try {
    await invoke<void>("add", { 
      ip: data.ipAddress, 
      nickname: data.name 
    });

    // Return the server object after successful command execution
    const newServer: Server = {
      id: Date.now().toString(),
      name: data.name,
      ipAddress: data.ipAddress,
    };

    return Promise.resolve(newServer);
  } catch (error) {
    console.error("Failed to add server:", error);
    throw error;
  }
}

export async function connectToServer(data: {
  ipAddress: string;
}): Promise<Server> {
  try {
    await invoke<void>("connect", {
      ip: data.ipAddress,
      nickname: getNickname(),
    });

    const connectedServer: Server = {
      id: Date.now().toString(),
      name: "Connected Server",
      ipAddress: data.ipAddress,
    };

    return Promise.resolve(connectedServer);
  } catch (error) {
    console.error("Failed to connect to server:", error);
    throw error;
  }
}

export async function updateServer(
  id: string,
  data: { name: string; ipAddress: string }
): Promise<Server> {
  // Example later when updating in Rust/Tauri:
  // import { invoke } from "@tauri-apps/api/core";
  // const updatedServer = await invoke<Server>("update_server", { id, data });
  // return updatedServer;

  const server = temporaryServers.find((item) => item.id === id);

  if (!server) {
    throw new Error("Server not found");
  }

  server.name = data.name;
  server.ipAddress = data.ipAddress;

  return Promise.resolve({ ...server });
}

export async function removeServer(id: string): Promise<void> {
  // Example later when deleting in Rust/Tauri:
  // import { invoke } from "@tauri-apps/api/core";
  // await invoke("remove_server", { id });

  temporaryServers = temporaryServers.filter((item) => item.id !== id);
  return Promise.resolve();
}