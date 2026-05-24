import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Room, RoomEvent, RemoteParticipant, RemoteTrack, RemoteTrackPublication, Track } from "livekit-client";
import type { MessageObj } from "./chatService";
import { loadMicDevice, loadSpeakerDevice } from "./settingsService";

let _room: Room | null = null;
let _isDeafened = false;

const _audioElements = new Map<string, HTMLAudioElement>();

export async function joinVoiceChat(sessionKey: string): Promise<void> {
  if (_room) return;

  const { token, url } = await invoke<{ token: string; url: string }>(
    "join_voice_chat",
    { sessionKey }
  );

  _room = new Room({
    audioCaptureDefaults: {
      noiseSuppression: true,
      echoCancellation: true,
      autoGainControl: false,
    },
  });

  _room.on(RoomEvent.TrackSubscribed, (
    track: RemoteTrack,
    _publication: RemoteTrackPublication,
    participant: RemoteParticipant
  ) => {
    if (track.kind !== Track.Kind.Audio) return;

    const audioEl = track.attach();
    audioEl.volume = _isDeafened ? 0 : 1;
    if (typeof audioEl.setSinkId === "function") {
      audioEl.setSinkId(loadSpeakerDevice());
    }
    _audioElements.set(participant.identity, audioEl);
    document.body.appendChild(audioEl);
  });

  _room.on(RoomEvent.TrackUnsubscribed, (
    track: RemoteTrack,
    _publication: RemoteTrackPublication,
    participant: RemoteParticipant
  ) => {
    track.detach();
    const el = _audioElements.get(participant.identity);
    if (el) {
      el.remove();
      _audioElements.delete(participant.identity);
    }
  });

  _room.on(RoomEvent.ParticipantConnected, (participant: RemoteParticipant) => {
    if (_isDeafened) participant.setVolume(0);
  });

  await _room.connect(url, token);
  await _room.localParticipant.setMicrophoneEnabled(true, {
    deviceId: loadMicDevice(),
  });
  await _sendVoiceSignal("voicestart");
}

export async function leaveVoiceChat(): Promise<void> {
  if (!_room) return;

  _audioElements.forEach((el) => el.remove());
  _audioElements.clear();

  await _room.disconnect();
  _room = null;
  _isDeafened = false;
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

  _audioElements.forEach((el) => {
    el.volume = deafened ? 0 : 1;
  });

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

export function updateSpeakerLevel(level: number): void {
  _audioElements.forEach((el) => {
    el.volume = level / 100;
  });
}

export function updateSpeakerDevice(deviceId: string): void {
  _audioElements.forEach((el) => {
    if (typeof el.setSinkId === "function") {
      el.setSinkId(deviceId);
    }
  });
}
async function _sendVoiceSignal(mType: "voicestart" | "voiceend"): Promise<void> {
  try {
    await invoke("send_voice_signal", { mType });
  } catch (e) {
    console.error(`Failed to send ${mType} signal:`, e);
  }
}