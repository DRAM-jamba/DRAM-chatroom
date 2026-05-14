import { useState, useEffect } from "react";
import TitleBar from "../components/TitleBar";
import logoIcon from "../assets/icons/logorgb.png";
import {
  saveTheme, loadTheme,
  saveFont, loadFont,
  saveMicHotkey, loadMicHotkey,
  saveHeadphonesHotkey, loadHeadphonesHotkey,
  saveMicDevice, loadMicDevice,
  saveMicLevel, loadMicLevel,
  saveSpeakerDevice, loadSpeakerDevice,
  saveSpeakerLevel, loadSpeakerLevel,
  type Theme, type Font,
} from "../services/settingsService";


type SettingsPageProps = {
  onBack: () => void;
};

function SettingsPage({ onBack }: SettingsPageProps) {
  const [theme, setTheme] = useState<Theme>(loadTheme());
  const [font, setFont] = useState<Font>(loadFont());
  const [micDevice, setMicDevice] = useState(loadMicDevice());
  const [micLevel, setMicLevel] = useState(loadMicLevel());
  const [micHotkey, setMicHotkey] = useState<string | null>(loadMicHotkey());
  const [headphonesHotkey, setHeadphonesHotkey] = useState<string | null>(loadHeadphonesHotkey());
  const [listeningFor, setListeningFor] = useState<"mic" | "headphones" | null>(null);
  const [micDevices, setMicDevices] = useState<MediaDeviceInfo[]>([]);
  const [speakerDevice, setSpeakerDevice] = useState(loadSpeakerDevice());
  const [speakerLevel, setSpeakerLevel] = useState(loadSpeakerLevel());
  const [speakerDevices, setSpeakerDevices] = useState<MediaDeviceInfo[]>([]);
  


  useEffect(() => {
    if (!listeningFor) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      e.preventDefault();
      const key = e.key.toUpperCase();
      if (listeningFor === "mic") {
        setMicHotkey(key);
        saveMicHotkey(key);
      }
      if (listeningFor === "headphones") {
        setHeadphonesHotkey(key);
        saveHeadphonesHotkey(key);
      }
      setListeningFor(null);
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [listeningFor]);

  useEffect(() => {
  const loadDevices = () => {
    navigator.mediaDevices.enumerateDevices().then((devices) => {
      const mics = devices.filter((d) => d.kind === "audioinput");
      const speakers = devices.filter((d) => d.kind === "audiooutput");
      setMicDevices(mics);
      setSpeakerDevices(speakers);
      if (mics.length > 0) {
        const saved = loadMicDevice();
        const exists = mics.some((d) => d.deviceId === saved);
        if (!exists) {
          const def = mics.find((d) => d.deviceId === "default") ?? mics[0];
          setMicDevice(def.deviceId);
          saveMicDevice(def.deviceId);
        }
      }
      if (speakers.length > 0) {
        const saved = loadSpeakerDevice();
        const exists = speakers.some((d) => d.deviceId === saved);
        if (!exists) {
          const def = speakers.find((d) => d.deviceId === "default") ?? speakers[0];
          setSpeakerDevice(def.deviceId);
          saveSpeakerDevice(def.deviceId);
        }
      }
    });
  };

  navigator.mediaDevices.getUserMedia({ audio: true })
    .then(loadDevices)
    .catch(loadDevices);

  navigator.mediaDevices.addEventListener("devicechange", loadDevices);
  return () => navigator.mediaDevices.removeEventListener("devicechange", loadDevices);
  }, []);

  return (
    <div className="servers-page">
      <TitleBar />
      <aside className="sidebar">
        <h1 className="logo">
          <img src={logoIcon} width="24" height="24" />
          quorthon
        </h1>
        <div className="sidebar-line" />

        <div className="server-list-container settings-sections">

          {/* Appearance */}
          <div className="server-card">
            <div className="server-header">
              <span className="server-title">Appearance</span>
            </div>
            <div className="server-details">
              <p className="input-label">Theme</p>
              <div className="settings-option-row">
                <button
                  className={`settings-option-btn ${theme === "dark" ? "active" : ""}`}
                  type="button"
                  onClick={() => { setTheme("dark"); saveTheme("dark"); }}
                >
                  dark
                </button>
                <button
                  className={`settings-option-btn ${theme === "light" ? "active" : ""}`}
                  type="button"
                  onClick={() => { setTheme("light"); saveTheme("light"); }}
                >
                  light
                </button>
              </div>

              <p className="input-label" style={{ marginTop: "12px" }}>Font</p>
              <div className="settings-option-row">
                <button
                  className={`settings-option-btn ${font === "default" ? "active" : ""}`}
                  type="button"
                  onClick={() => { setFont("default"); saveFont("default"); }}
                >
                  exo
                </button>
                <button
                  className={`settings-option-btn ${font === "alternative" ? "active" : ""}`}
                  type="button"
                  onClick={() => { setFont("alternative"); saveFont("alternative"); }}
                >
                  monocraft
                </button>
              </div>
            </div>
          </div>

          {/* Voice */}
          <div className="server-card">
            <div className="server-header">
              <span className="server-title">Voice</span>
            </div>
            <div className="server-details">
                <p className="input-label">Microphone device</p>
                <div className="mic-device-list">
                    {micDevices.length === 0 ? (
                    <p className="settings-slider-value">No microphones found</p>
                    ) : (
                    micDevices.map((device) => (
                        <button
                        key={device.deviceId}
                        className={`mic-device-btn ${micDevice === device.deviceId ? "active" : ""}`}
                        type="button"
                        onClick={() => { setMicDevice(device.deviceId); saveMicDevice(device.deviceId); }}
                        >
                        {device.label || "Microphone"}
                        </button>
                    ))
                    )}
                </div>

                <p className="input-label">Microphone level</p>
                <input
                    type="range"
                    min={0}
                    max={100}
                    value={micLevel}
                    onChange={(e) => {
                        const val = Number(e.target.value);
                        setMicLevel(val);
                        saveMicLevel(val);
                    }}
                    className="settings-slider"
                />
                <p className="settings-slider-value">{micLevel}%</p>
                </div>
          </div>

          {/* Audio */}
          <div className="server-card">
            <div className="server-header">
                <span className="server-title">Audio</span>
            </div>
            <div className="server-details">
                <p className="input-label">Speaker device</p>
                <div className="mic-device-list">
                {speakerDevices.length === 0 ? (
                    <p className="settings-slider-value">No speakers found</p>
                ) : (
                    speakerDevices.map((device) => (
                    <button
                        key={device.deviceId}
                        className={`mic-device-btn ${speakerDevice === device.deviceId ? "active" : ""}`}
                        type="button"
                        onClick={() => { setSpeakerDevice(device.deviceId); saveSpeakerDevice(device.deviceId); }}
                    >
                        {device.label || (device.deviceId === "default" ? "Windows default" : "Speaker")}
                    </button>
                    ))
                )}
                </div>

                <p className="input-label">Speaker level</p>
                <input
                type="range"
                min={0}
                max={100}
                value={speakerLevel}
                onChange={(e) => {
                    const val = Number(e.target.value);
                    setSpeakerLevel(val);
                    saveSpeakerLevel(val);
                }}
                className="settings-slider"
                />
                <p className="settings-slider-value">{speakerLevel}%</p>
            </div>
          </div>

          {/* Hotkeys */}
          <div className="server-card">
            <div className="server-header">
              <span className="server-title">Hotkeys</span>
            </div>
            <div className="server-details">
              <div className="hotkey-row">
                <span className="hotkey-label">Mute / Unmute microphone</span>
                <div style={{ display: "flex", gap: "6px", alignItems: "center" }}>
                    {micHotkey && listeningFor !== "mic" ? (
                        <kbd
                        className="hotkey-key"
                        style={{ cursor: "pointer" }}
                        onClick={() => setListeningFor("mic")}
                        title="Click to change"
                        >
                        {micHotkey}
                        </kbd>
                    ) : (
                        <button
                        className={`hotkey-set-btn ${listeningFor === "mic" ? "active" : ""}`}
                        type="button"
                        onClick={() => setListeningFor("mic")}
                        >
                        {listeningFor === "mic" ? "..." : "set"}
                        </button>
                    )}
                    </div>
              </div>
              <div className="hotkey-row">
                <span className="hotkey-label">Mute / Unmute headphones</span>
                <div style={{ display: "flex", gap: "6px", alignItems: "center" }}>
                    {headphonesHotkey && listeningFor !== "headphones" ? (
                        <kbd
                        className="hotkey-key"
                        style={{ cursor: "pointer" }}
                        onClick={() => setListeningFor("headphones")}
                        title="Click to change"
                        >
                        {headphonesHotkey}
                        </kbd>
                    ) : (
                        <button
                        className={`hotkey-set-btn ${listeningFor === "headphones" ? "active" : ""}`}
                        type="button"
                        onClick={() => setListeningFor("headphones")}
                        >
                        {listeningFor === "headphones" ? "..." : "set"}
                        </button>
                    )}
                    </div>
              </div>
            </div>
          </div>

        </div>

        <div className="sidebar-line bottom-line" />
        <button
          className="settings-back-btn"
          type="button"
          onClick={onBack}
        >
          back
        </button>
      </aside>
    </div>
  );
}

export default SettingsPage;