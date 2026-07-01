import { register, unregisterAll } from "@tauri-apps/plugin-global-shortcut";

export interface ShortcutCallbacks {
  onToggleSearch: () => void;
  onToggleFences: () => void;
}

export async function initGlobalShortcuts(callbacks: ShortcutCallbacks): Promise<void> {
  await unregisterAll();
  await register("Alt+Space", () => callbacks.onToggleSearch());
  await register("Ctrl+Shift+F", () => callbacks.onToggleFences());
}

export { unregisterAll };
