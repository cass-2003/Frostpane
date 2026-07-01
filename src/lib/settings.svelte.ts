import { invoke } from "@tauri-apps/api/core";

const DEFAULTS = {
  iconSize: 48,
  animationLevel: "full" as "off" | "basic" | "full",
  locale: "en" as "en" | "zh",
};

let iconSize = $state(DEFAULTS.iconSize);
let animationLevel = $state(DEFAULTS.animationLevel);
let _locale = $state(DEFAULTS.locale);

export const settings = {
  get iconSize() { return iconSize; },
  set iconSize(v: number) { iconSize = v; save(); },
  get animationLevel() { return animationLevel; },
  set animationLevel(v: "off" | "basic" | "full") { animationLevel = v; save(); },
  get locale() { return _locale; },
  set locale(v: "en" | "zh") { _locale = v; save(); },
};

// Sets locale without triggering a save (used during init before saved prefs are loaded)
export function setLocaleInitial(v: "en" | "zh") {
  _locale = v;
}

async function save() {
  try {
    await invoke("save_app_settings", {
      settings: { icon_size: iconSize, animation_level: animationLevel, locale: _locale },
    });
  } catch (e) {
    console.error("Failed to save settings:", e);
  }
}

export async function loadSettings() {
  try {
    const s = await invoke<{ icon_size?: number; animation_level?: string; locale?: string }>(
      "load_app_settings"
    );
    if (s.icon_size) iconSize = s.icon_size;
    if (s.animation_level) animationLevel = s.animation_level as typeof animationLevel;
    if (s.locale === "en" || s.locale === "zh") _locale = s.locale;
  } catch {
    // First run, no settings file yet
  }
}
