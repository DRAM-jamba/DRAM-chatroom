const CODE_MAP: Record<string, string> = {
  Space: "SPACE",
  ArrowLeft: "LEFT", ArrowRight: "RIGHT", ArrowUp: "UP", ArrowDown: "DOWN",
  NumpadAdd: "NUM+", NumpadSubtract: "NUM-", NumpadMultiply: "NUM*",
  NumpadDivide: "NUM/", NumpadDecimal: "NUM.", NumpadEnter: "NUMENTER",
  Numpad0: "NUM0", Numpad1: "NUM1", Numpad2: "NUM2", Numpad3: "NUM3",
  Numpad4: "NUM4", Numpad5: "NUM5", Numpad6: "NUM6", Numpad7: "NUM7",
  Numpad8: "NUM8", Numpad9: "NUM9",
};

export function getHotkeyString(e: KeyboardEvent): string {
  return CODE_MAP[e.code] ?? e.key.toUpperCase();
}