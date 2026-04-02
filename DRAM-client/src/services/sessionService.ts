import type { Session } from "../types/session";

// Temporary data for testing
let temporarySessions: Session[] = [
  {
    id: "1",
    name: "session1",
    lastConnected: "today, 12:40",
  },
  {
    id: "2",
    name: "session2",
    lastConnected: "today, 14:08",
  },
  {
    id: "3",
    name: "session3",
    lastConnected: "2 days ago",
  },
];

export async function getSessions(): Promise<Session[]> {
  // Later from Rust:
  // import { invoke } from "@tauri-apps/api/core";
  // return await invoke<Session[]>("get_sessions");

  return Promise.resolve([...temporarySessions]);
}

export async function createSession(data: {
  sessionName: string;
  sessionKey: string;
}): Promise<{ generatedKey: string }> {
  // Later from Rust:
  // import { invoke } from "@tauri-apps/api/core";
  // return await invoke<{ generatedKey: string }>("create_session", {
  //   sessionName: data.sessionName,
  //   sessionKey: data.sessionKey,
  // });

  const newSession: Session = {
    id: Date.now().toString(),
    name: data.sessionName,
    lastConnected: "just now",
  };

  temporarySessions.push(newSession);

  return Promise.resolve({
    generatedKey: `${data.sessionName}-${data.sessionKey}-generated-key`,
  });
}

export async function updateSession(
  id: string,
  data: { name: string }
): Promise<Session> {
  const session = temporarySessions.find((item) => item.id === id);

  if (!session) {
    throw new Error("Session not found");
  }

  session.name = data.name;

  return Promise.resolve({ ...session });
}

export async function removeSession(id: string): Promise<void> {
  temporarySessions = temporarySessions.filter((item) => item.id !== id);
  return Promise.resolve();
}