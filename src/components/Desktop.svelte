<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, tick } from "svelte";

  interface DesktopIcon {
    name: string;
    path: string;
    icon_data: string;
    icon_size: number;
    x: number;
    y: number;
    is_shortcut: boolean;
  }

  let icons = $state<DesktopIcon[]>([]);
  let loading = $state(true);
  let error = $state("");

  onMount(async () => {
    try {
      icons = await invoke<DesktopIcon[]>("get_desktop_icons");
    } catch (e) {
      error = String(e);
      console.error("Failed to load desktop icons:", e);
    } finally {
      loading = false;
      await tick();
      renderAllIcons();
    }
  });

  function renderAllIcons() {
    for (const icon of icons) {
      const canvas = document.getElementById(`icon-${css(icon.path)}`) as HTMLCanvasElement;
      if (canvas && icon.icon_data) {
        renderIconToCanvas(canvas, icon);
      }
    }
  }

  function css(path: string): string {
    let hash = 0;
    for (let i = 0; i < path.length; i++) {
      hash = ((hash << 5) - hash + path.charCodeAt(i)) | 0;
    }
    return "i" + Math.abs(hash).toString(36);
  }

  function renderIconToCanvas(canvas: HTMLCanvasElement, icon: DesktopIcon) {
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const size = icon.icon_size || 48;
    canvas.width = size;
    canvas.height = size;

    try {
      const raw = atob(icon.icon_data);
      const pixels = new Uint8ClampedArray(raw.length);
      for (let i = 0; i < raw.length; i++) {
        pixels[i] = raw.charCodeAt(i);
      }
      if (pixels.length === size * size * 4) {
        const imgData = new ImageData(pixels, size, size);
        ctx.putImageData(imgData, 0, 0);
      } else {
        console.warn(`Icon ${icon.name}: expected ${size*size*4} bytes, got ${pixels.length}`);
      }
    } catch (e) {
      console.error("Icon render error:", icon.name, e);
    }
  }

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
    <div class="status-pill">
      <span class="spinner">❄</span>
      <span>Frostpane loading...</span>
    </div>
  {:else if error}
    <div class="status-pill error">
      <span>Error: {error}</span>
    </div>
  {:else}
    {#each icons as icon (icon.path)}
      <button
        class="desktop-icon"
        style="left: {icon.x}px; top: {icon.y}px"
        ondblclick={() => handleDoubleClick(icon)}
        title="{icon.name} — {icon.path}"
      >
        <canvas
          id="icon-{css(icon.path)}"
          class="icon-canvas"
          width="48"
          height="48"
        ></canvas>
        <span class="icon-label">{icon.name}</span>
      </button>
    {/each}

    <div class="status-pill">
      ❄ Frostpane M0 · {icons.length} icons loaded
    </div>
  {/if}
</div>

<style>
  .desktop-overlay {
    position: fixed;
    inset: 0;
  }

  .status-pill {
    position: fixed;
    bottom: 16px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 22px;
    background: var(--glass-bg);
    border: 1px solid var(--glass-border);
    border-radius: 14px;
    backdrop-filter: blur(20px);
    color: var(--text);
    font-size: 13px;
    white-space: nowrap;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  }

  .status-pill.error {
    border-color: #e04060;
    color: #ff8090;
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
    color: var(--text);
    transition: background 0.15s;
  }

  .desktop-icon:hover {
    background: rgba(255, 255, 255, 0.12);
  }

  .desktop-icon:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .icon-canvas {
    width: 48px;
    height: 48px;
    pointer-events: none;
  }

  .icon-label {
    font-size: 11px;
    text-align: center;
    line-height: 1.25;
    max-width: 76px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    text-shadow:
      0 1px 3px rgba(0, 0, 0, 0.9),
      0 0 8px rgba(0, 0, 0, 0.6);
    pointer-events: none;
  }
</style>
