import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Room, RoomEvent, RemoteParticipant } from "livekit-client";
import type { MessageObj } from "./chatService";

let _room: Room | null = null;
let _isDeafened = false;

export async function joinVoiceChat(sessionKey: string): Promise<void> {
  if (_room) return;

  const { token, url } = await invoke<{ token: string; url: string }>(
    "join_voice_chat",
    { sessionKey }
  );

  const room = new Room();

  room.on(RoomEvent.ParticipantConnected, (participant: RemoteParticipant) => {
    if (_isDeafened) participant.setVolume(0);
  });

  room.on(RoomEvent.Disconnected, () => {
    _room = null;
    _isDeafened = false;
  });

  try {
    await room.connect(url, token);
    await room.localParticipant.setMicrophoneEnabled(true);
    _room = room;
    await _sendVoiceSignal("voicestart");
  } catch (error) {
    console.error("Voice chat connection failed:", error);
    try {
      await room.disconnect();
    } catch (disconnectError) {
      console.warn("Failed to clean up failed voice chat room:", disconnectError);
    }
    throw error;
  }
}

export async function leaveVoiceChat(): Promise<void> {
  if (!_room) return;

  const room = _room;
  _room = null;
  _isDeafened = false;

  try {
    await room.disconnect();
  } catch (error) {
    console.warn("Error disconnecting voice chat room:", error);
  }

  await _sendVoiceSignal("voiceend");
}

export async function setMicMuted(muted: boolean): Promise<void> {
  if (!_room) return;
  await _room.localParticipant.setMicrophoneEnabled(!muted);
}

export async function setDeafened(deafened: boolean): Promise<void> {
  _isDeafened = deafened;
  if (!_room) return;
  _room.remoteParticipants.forEach((p) => p.setVolume(deafened ? 0 : 1));
}

export async function subscribeToVoiceList(
  onUpdate: (voiceUsers: string[]) => void
): Promise<() => void> {
  return listen<MessageObj>("voice_list", (event) => {
    try {
      onUpdate(JSON.parse(event.payload.body) as string[]);
    } catch (e) {
      console.error("Failed to parse voice_list payload", e);
    }
  });
}

// Uses a dedicated Tauri command so the signal isn't re-wrapped as MessageType::Message
async function _sendVoiceSignal(mType: "voicestart" | "voiceend"): Promise<void> {
  try {
    await invoke("send_voice_signal", { mType });
  } catch (e) {
    console.error(`Failed to send ${mType} signal:`, e);
  }
}