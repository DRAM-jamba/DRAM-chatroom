import type { Session } from "../types/session";
import { invoke } from "@tauri-apps/api/core";

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
  return await invoke<Session[]>("get_sessions");
}

// ─── createSession ──────────────────────────────────────────────────────────
// Creates a new session with the given name.
// The server generates and returns the session key to be shared with other users.

export async function createSession(data: {
  sessionName: string;
  sessionKey: string;
}): Promise<{ session: Session; generatedKey: string }> {
  const generatedKey = await invoke<string>("create_session", {
    name: data.sessionName,
  });

  const newSession: Session = {
    id: Date.now().toString(),
    name: data.sessionName,
    lastConnected: "just now",
  };

  temporarySessions.push(newSession);

  return Promise.resolve({
    session: newSession,
    generatedKey: generatedKey,
  });
}

// ─── addSession ─────────────────────────────────────────────────────────────
// Adds an existing session to the user's session list using a session name
// and session key provided by another user. The server validates the
// credentials before the session appears in the list.

export async function addSession(data: {
  sessionName: string;    // Accepted but not used by server
  sessionKey: string;
}): Promise<void> {
  await invoke<void>("add_session", {
    sessionKey: data.sessionKey,  // Only send the key
  });
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
