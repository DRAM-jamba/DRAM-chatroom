import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Room, RoomEvent, RemoteParticipant, RemoteTrack, RemoteTrackPublication, Track } from "livekit-client";
import type { MessageObj } from "./chatService";
import { loadMicDevice, loadSpeakerDevice, loadMicLevel, loadNoiseSuppression, saveNoiseSuppression } from "./settingsService";
import { NoiseSuppressorWorklet_Name } from "@timephy/rnnoise-wasm";
import NoiseSuppressorWorklet from "@timephy/rnnoise-wasm/NoiseSuppressorWorklet?worker&url";

let _room: Room | null = null;
let _isDeafened = false;
let _gainNode: GainNode | null = null;
let _audioContext: AudioContext | null = null;
let _rawStream: MediaStream | null = null;
let _useRnnoise = loadNoiseSuppression();

const _audioElements = new Map<string, HTMLAudioElement>();

export async function joinVoiceChat(sessionKey: string): Promise<void> {
  if (_room) return;

  const { token, url } = await invoke<{ token: string; url: string }>(
    "join_voice_chat",
    { sessionKey }
  );

  _room = new Room();

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

  _rawStream = await navigator.mediaDevices.getUserMedia({
    audio: {
      deviceId: { exact: loadMicDevice() },
      noiseSuppression: !_useRnnoise,
      echoCancellation: true,
      autoGainControl: false,
    }
  });

  _audioContext = new AudioContext();
  const source = _audioContext.createMediaStreamSource(_rawStream);
  _gainNode = _audioContext.createGain();
  const destination = _audioContext.createMediaStreamDestination();
  _gainNode.gain.value = loadMicLevel() / 100;

  if (_useRnnoise) {
    await _audioContext.audioWorklet.addModule(NoiseSuppressorWorklet);
    const noiseNode = new AudioWorkletNode(_audioContext, NoiseSuppressorWorklet_Name);
    source.connect(noiseNode);
    noiseNode.connect(_gainNode);
  } else {
    source.connect(_gainNode);
  }

  _gainNode.connect(destination);
  const processedTrack = destination.stream.getAudioTracks()[0];
  await _room.localParticipant.publishTrack(processedTrack, {
    source: Track.Source.Microphone,
  });

  await _sendVoiceSignal("voicestart");
}

export async function leaveVoiceChat(): Promise<void> {
  if (!_room) return;

  _audioElements.forEach((el) => el.remove());
  _audioElements.clear();

  _rawStream?.getTracks().forEach(t => t.stop());
  _rawStream = null;

  _gainNode = null;
  _audioContext?.close();
  _audioContext = null;

  await _room.disconnect();
  _room = null;
  _isDeafened = false;
  await _sendVoiceSignal("voiceend");
}

export function isInVoiceChat(): boolean {
  return _room !== null;
}

export async function reconnectMic(): Promise<void> {
  if (!_room || !_audioContext) return;

  _rawStream?.getTracks().forEach(t => t.stop());
  _rawStream = null;

  const existingPublication = _room.localParticipant.getTrackPublication(Track.Source.Microphone);
  if (existingPublication?.track) {
    await _room.localParticipant.unpublishTrack(existingPublication.track.mediaStreamTrack);
  }

  _audioContext.close();
  _audioContext = new AudioContext();

  _rawStream = await navigator.mediaDevices.getUserMedia({
    audio: {
      deviceId: { exact: loadMicDevice() },
      noiseSuppression: !_useRnnoise,
      echoCancellation: true,
      autoGainControl: false,
    }
  });

  const source = _audioContext.createMediaStreamSource(_rawStream);
  _gainNode = _audioContext.createGain();
  const destination = _audioContext.createMediaStreamDestination();
  const level = loadMicLevel();
  _gainNode.gain.value = level / 100;

  if (_useRnnoise) {
    await _audioContext.audioWorklet.addModule(NoiseSuppressorWorklet);
    const noiseNode = new AudioWorkletNode(_audioContext, NoiseSuppressorWorklet_Name);
    source.connect(noiseNode);
    noiseNode.connect(_gainNode);
  } else {
    source.connect(_gainNode);
  }

  _gainNode.connect(destination);
  const processedTrack = destination.stream.getAudioTracks()[0];
  await _room.localParticipant.publishTrack(processedTrack, {
    source: Track.Source.Microphone,
  });
}

export function updateMicLevel(level: number): void {
  if (!_gainNode) return;
  _gainNode.gain.value = level / 100;
}

export function setUseRnnoise(value: boolean): void {
  _useRnnoise = value;
  saveNoiseSuppression(value);
}

export async function setMicMuted(muted: boolean): Promise<void> {
  if (!_gainNode) return;
  _gainNode.gain.value = muted ? 0 : loadMicLevel() / 100;
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

export function setParticipantVolume(username: string, volume: number): void {

}

async function _sendVoiceSignal(mType: "voicestart" | "voiceend"): Promise<void> {
  try {
    await invoke("send_voice_signal", { mType });
  } catch (e) {
    console.error(`Failed to send ${mType} signal:`, e);
  }
}