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

// ─── getSessions ────────────────────────────────────────────────────────────
// Fetches the list of sessions the user belongs to.

export async function getSessions(): Promise<Session[]> {
  // Later from Rust:
  // import { invoke } from "@tauri-apps/api/core";
  // return await invoke<Session[]>("get_sessions");

  return Promise.resolve([...temporarySessions]);
}

// ─── createSession ──────────────────────────────────────────────────────────
// Creates a new session with the given name and key.
// Returns the server-generated session key to be shared with other users.

export async function createSession(data: {
  sessionName: string;
  sessionKey: string;
}): Promise<{ session: Session; generatedKey: string }> {
  // Later from Rust:
  // import { invoke } from "@tauri-apps/api/core";
  // return await invoke<{ session: Session; generatedKey: string }>("create_session", {
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
    session: newSession,
    generatedKey: `${data.sessionName}-${data.sessionKey}-generated-key`,
  });
}

// ─── addSession ─────────────────────────────────────────────────────────────
// Adds an existing session to the user's session list using a session name
// and session key provided by another user. The server validates the
// credentials before the session appears in the list.

export async function addSession(data: {
  sessionName: string;
  sessionKey: string;
}): Promise<Session> {
  // Later from Rust:
  // import { invoke } from "@tauri-apps/api/core";
  // return await invoke<Session>("add_session", {
  //   sessionName: data.sessionName,
  //   sessionKey: data.sessionKey,
  // });

  const newSession: Session = {
    id: Date.now().toString(),
    name: data.sessionName,
    lastConnected: "just now",
  };

  temporarySessions.push(newSession);

  return Promise.resolve(newSession);
}

// ─── joinSession ─────────────────────────────────────────────────────────────
// Called when the user clicks "connect" on a session card.
// Establishes an active connection to the session on the server.
// Returns the session name to navigate to the chat view.

export async function joinSession(sessionId: string): Promise<string> {
  // Later from Rust:
  // import { invoke } from "@tauri-apps/api/core";
  // return await invoke<string>("join_session", { sessionId });

  const session = temporarySessions.find((s) => s.id === sessionId);

  if (!session) {
    throw new Error("Session not found");
  }

  session.lastConnected = "just now";

  return Promise.resolve(session.name);
}

// ─── updateSession ───────────────────────────────────────────────────────────
// Renames a session. Only the display name can be changed.

export async function updateSession(
  id: string,
  data: { name: string }
): Promise<Session> {
  // Later from Rust:
  // import { invoke } from "@tauri-apps/api/core";
  // return await invoke<Session>("update_session", { id, name: data.name });

  const session = temporarySessions.find((item) => item.id === id);

  if (!session) {
    throw new Error("Session not found");
  }

  session.name = data.name;

  return Promise.resolve({ ...session });
}

// ─── forgetSession ───────────────────────────────────────────────────────────
// Removes the session from the user's session list.
// The user will no longer see or be able to connect to this session.

export async function forgetSession(id: string): Promise<void> {
  // Later from Rust:
  // import { invoke } from "@tauri-apps/api/core";
  // await invoke("forget_session", { id });

  temporarySessions = temporarySessions.filter((item) => item.id !== id);

  return Promise.resolve();
}
