import type { Session } from "../types/session";
import { invoke } from "@tauri-apps/api/core";


export async function getSessions(): Promise<Session[]> {
  return await invoke<Session[]>("get_sessions");
}


export async function createSession(data: {
  sessionName: string;
  sessionKey: string; // ignored — key is generated server-side
}): Promise<{ session: Session; generatedKey: string }> {
  const generatedKey = await invoke<string>("create_session", {
    name: data.sessionName,
  });

  const sessions = await invoke<Session[]>("get_sessions");
  const newSession =
    sessions.find((s) => s.id === generatedKey) ??
    ({
      id: generatedKey,
      name: data.sessionName,
    } as Session);

  return { session: newSession, generatedKey };
}


export async function addSession(data: {
  sessionKey: string;
}): Promise<void> {
  await invoke<void>("add_session", {
    sessionKey: data.sessionKey,
  });
}


export async function joinSession(sessionId: string): Promise<string> {
  await invoke<void>("connect_session", {
    sessionKey: sessionId,
  });

  return sessionId;
}

// updateSession
// Renames a session locally.
// NOTE: The server does not yet expose a rename endpoint, so this is a
// client-side-only update until the backend implements it.

export async function updateSession(
  id: string,
  data: { name: string }
): Promise<Session> {
  // TODO: replace with invoke("update_session", { id, name: data.name })
  // once the Rust command is implemented.
  const sessions = await invoke<Session[]>("get_sessions");
  const session = sessions.find((s) => s.id === id);

  if (!session) {
    throw new Error("Session not found");
  }

  return { ...session, name: data.name };
}


export async function forgetSession(sessionKey: string): Promise<void> {
  await invoke<void>("forget_session", {
    sessionKey: sessionKey,
  });
}


export async function deleteSession(sessionKey: string): Promise<void> {
  await invoke<void>("delete_session", {
    sessionKey: sessionKey,
  });
}


export async function disconnectFromServer(): Promise<void> {
  await invoke<void>("disconnect");
}