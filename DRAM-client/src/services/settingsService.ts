const KEYS = {
  theme: "setting_theme",
  font: "setting_font",
  micHotkey: "setting_mic_hotkey",
  headphonesHotkey: "setting_headphones_hotkey",
  micDevice: "setting_mic_device",
  micLevel: "setting_mic_level",
  speakerDevice: "setting_speaker_device",
  speakerLevel: "setting_speaker_level",
};

export type Theme = "dark" | "light";
export type Font = "default" | "alternative";


export function saveTheme(theme: Theme): void {
  localStorage.setItem(KEYS.theme, theme);
  applyTheme(theme);
}

export function loadTheme(): Theme {
  return (localStorage.getItem(KEYS.theme) as Theme) ?? "dark";
}

export function applyTheme(theme: Theme): void {
  document.documentElement.setAttribute("data-theme", theme);
}


export function saveFont(font: Font): void {
  localStorage.setItem(KEYS.font, font);
  applyFont(font);
}

export function loadFont(): Font {
  return (localStorage.getItem(KEYS.font) as Font) ?? "default";
}

export function applyFont(font: Font): void {
  document.documentElement.setAttribute("data-font", font);
}


export function saveMicHotkey(key: string): void {
  localStorage.setItem(KEYS.micHotkey, key);
}

export function loadMicHotkey(): string | null {
  return localStorage.getItem(KEYS.micHotkey);
}


export function saveHeadphonesHotkey(key: string): void {
  localStorage.setItem(KEYS.headphonesHotkey, key);
}

export function loadHeadphonesHotkey(): string | null {
  return localStorage.getItem(KEYS.headphonesHotkey);
}

export function saveMicDevice(deviceId: string): void {
  localStorage.setItem(KEYS.micDevice, deviceId);
}

export function loadMicDevice(): string {
  return localStorage.getItem(KEYS.micDevice) ?? "default";
}

export function saveMicLevel(level: number): void {
  localStorage.setItem(KEYS.micLevel, String(level));
}

export function loadMicLevel(): number {
  return Number(localStorage.getItem(KEYS.micLevel)) || 50;
}

export function saveSpeakerDevice(deviceId: string): void {
  localStorage.setItem(KEYS.speakerDevice, deviceId);
}

export function loadSpeakerDevice(): string {
  return localStorage.getItem(KEYS.speakerDevice) ?? "default";
}

export function saveSpeakerLevel(level: number): void {
  localStorage.setItem(KEYS.speakerLevel, String(level));
}

export function loadSpeakerLevel(): number {
  return Number(localStorage.getItem(KEYS.speakerLevel)) || 50;
}

export function loadAllSettings(): void {
  applyTheme(loadTheme());
  applyFont(loadFont());
}

