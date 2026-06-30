<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  interface DesktopIcon {
    name: string;
    path: string;
    icon_data: string; // base64 encoded PNG
    x: number;
    y: number;
    is_shortcut: boolean;
  }

  let icons = $state<DesktopIcon[]>([]);
  let loading = $state(true);

  onMount(async () => {
    try {
      icons = await invoke<DesktopIcon[]>("get_desktop_icons");
    } catch (e) {
      console.error("Failed to load desktop icons:", e);
    } finally {
      loading = false;
    }
  });

  async function handleDoubleClick(icon: DesktopIcon) {
    try {
      await invoke("open_item", { path: icon.path });
    } catch (e) {
      console.error("Failed to open:", e);
    }
  }
</script>

<div class="desktop-overlay">
  {#if loading}
    <div class="loading-indicator">
      <span class="spinner">❄</span>
      <span>Frostpane loading...</span>
    </div>
  {:else}
    {#each icons as icon (icon.path)}
      <button
        class="desktop-icon"
        style="left: {icon.x}px; top: {icon.y}px"
        ondblclick={() => handleDoubleClick(icon)}
      >
        {#if icon.icon_data}
          <img
            class="icon-image"
            src="data:image/png;base64,{icon.icon_data}"
            alt={icon.name}
          />
        {:else}
          <div class="icon-placeholder">📄</div>
        {/if}
        <span class="icon-label">{icon.name}</span>
      </button>
    {/each}
  {/if}
</div>

<style>
  .desktop-overlay {
    position: fixed;
    inset: 0;
    pointer-events: none;
  }

  .loading-indicator {
    position: absolute;
    bottom: 40px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 20px;
    background: var(--glass-bg);
    border: 1px solid var(--glass-border);
    border-radius: 12px;
    backdrop-filter: blur(20px);
    color: var(--text);
    font-size: 14px;
    pointer-events: auto;
  }

  .spinner {
    display: inline-block;
    font-size: 18px;
    animation: spin 2s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .desktop-icon {
    position: absolute;
    width: 80px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 6px 4px;
    border-radius: 8px;
    border: none;
    background: transparent;
    cursor: default;
    pointer-events: auto;
    color: var(--text);
    transition: background 0.15s;
  }

  .desktop-icon:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .desktop-icon:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .icon-image {
    width: 48px;
    height: 48px;
    pointer-events: none;
  }

  .icon-placeholder {
    width: 48px;
    height: 48px;
    display: grid;
    place-items: center;
    font-size: 28px;
  }

  .icon-label {
    font-size: 11px;
    text-align: center;
    line-height: 1.2;
    max-width: 76px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    text-shadow: 0 1px 4px rgba(0, 0, 0, 0.7);
    pointer-events: none;
  }
</style>
