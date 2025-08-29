const CODE_LABELS: Record<string, string> = {
  KeyA: "A",
  KeyB: "B",
  KeyC: "C",
  KeyD: "D",
  KeyE: "E",
  KeyF: "F",
  KeyG: "G",
  KeyH: "H",
  KeyI: "I",
  KeyJ: "J",
  KeyK: "K",
  KeyL: "L",
  KeyM: "M",
  KeyN: "N",
  KeyO: "O",
  KeyP: "P",
  KeyQ: "Q",
  KeyR: "R",
  KeyS: "S",
  KeyT: "T",
  KeyU: "U",
  KeyV: "V",
  KeyW: "W",
  KeyX: "X",
  KeyY: "Y",
  KeyZ: "Z",
  // Numbers (row)
  Digit0: "0",
  Digit1: "1",
  Digit2: "2",
  Digit3: "3",
  Digit4: "4",
  Digit5: "5",
  Digit6: "6",
  Digit7: "7",
  Digit8: "8",
  Digit9: "9",
  // Punctuation/others
  Minus: "-",
  Equal: "=",
  BracketLeft: "[",
  BracketRight: "]",
  Backslash: "\\",
  Semicolon: ";",
  Quote: "'",
  Backquote: "`",
  Comma: ",",
  Period: ".",
  Slash: "/",
  // Numpad
  Numpad0: "Num0",
  Numpad1: "Num1",
  Numpad2: "Num2",
  Numpad3: "Num3",
  Numpad4: "Num4",
  Numpad5: "Num5",
  Numpad6: "Num6",
  Numpad7: "Num7",
  Numpad8: "Num8",
  Numpad9: "Num9",
  NumpadAdd: "+",
  NumpadSubtract: "-",
  NumpadMultiply: "*",
  NumpadDivide: "/",
  NumpadDecimal: ".",
  NumpadEnter: "Enter",
  // Function/navigation
  Escape: "Esc",
  Tab: "Tab",
  CapsLock: "CapsLock",
  Space: "Space",
  Enter: "Enter",
  Backspace: "Backspace",
  Delete: "Delete",
  ArrowUp: "↑",
  ArrowDown: "↓",
  ArrowLeft: "←",
  ArrowRight: "→",
  Home: "Home",
  End: "End",
  PageUp: "PageUp",
  PageDown: "PageDown",
};

for (let i = 1; i <= 24; i++) CODE_LABELS[`F${i}`] = `F${i}`;

export const baseKeyFromEvent = (e: KeyboardEvent): string | null => {
  // Ignore IME / dead keys
  if (e.isComposing || e.key === "Dead") return null;
  // Prefer layout-independent physical code
  const byCode = CODE_LABELS[e.code];
  if (byCode) return byCode;
  // Fallback to some known 'key' values (arrows on some browsers)
  const byKey = CODE_LABELS[e.key];
  if (byKey) return byKey;

  if (e.key && e.key.length === 1) return e.key.toUpperCase();
  return null;
};

const TOKEN_PAIRS = [
  ["⌘", "CommandOrControl"],
  ["Ctrl", "Control"],
  ["⌥", "Alt"],
  ["⇧", "Shift"],
  ["↑", "Up"],
  ["↓", "Down"],
  ["←", "Left"],
  ["→", "Right"],
  ["Esc", "Escape"],
  ["Space", "Space"],
  ["Enter", "Enter"],
  ["Backspace", "Backspace"],
  ["Delete", "Delete"],
  ["Tab", "Tab"],
  ["⌘", "Command"],
  ["⌥", "Option"],
] as const satisfies readonly (readonly [string, string])[];

export const UI_TO_ACCEL: Record<string, string> = {};
export const ACCEL_TO_UI: Record<string, string> = {};
for (const [ui, accel] of TOKEN_PAIRS) {
  if (!(ui in UI_TO_ACCEL)) UI_TO_ACCEL[ui] = accel;
  ACCEL_TO_UI[accel] = ui;
}

export const uiComboToAccelerator = (input: string): string => {
  const parts = input
    .split("+")
    .map((p) => p.trim())
    .filter(Boolean);
  const mods: string[] = [];
  let key = "";

  for (const p of parts) {
    if (UI_TO_ACCEL[p]) mods.push(UI_TO_ACCEL[p]);
    else key = p.length === 1 ? p.toUpperCase() : p;
  }

  const accel = [...mods, key].filter(Boolean).join("+");
  return accel;
};

export const acceleratorToUI = (accel: string): string => {
  if (!accel) return "";
  const parts = accel
    .split("+")
    .map((p) => p.trim())
    .filter(Boolean);
  const mods: string[] = [];
  let key = "";

  for (const p of parts) {
    if (ACCEL_TO_UI[p]) mods.push(ACCEL_TO_UI[p]);
    else key = p.length === 1 ? p.toUpperCase() : p;
  }
  return [...mods, key].filter(Boolean).join("+");
};
