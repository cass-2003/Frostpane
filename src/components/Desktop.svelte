<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  interface DesktopIcon {
    name: string;
    path: string;
    icon_data: string;
    icon_size: number;
    x: number;
    y: number;
    is_shortcut: boolean;
  }

  interface IconPosition {
    path: string;
    x: number;
    y: number;
  }

  const GRID_W = 90;
  const GRID_H = 100;
  const DRAG_THRESHOLD = 5;

  let icons = $state<DesktopIcon[]>([]);
  let loading = $state(true);
  let error = $state("");

  // Drag state
  let dragging = $state<string | null>(null);
  let ghostX = $state(0);
  let ghostY = $state(0);
  let dragOffsetX = 0;
  let dragOffsetY = 0;
  let dragStartX = 0;
  let dragStartY = 0;
  let didDrag = false;

  let draggedIcon = $derived(
    dragging ? (icons.find((ic) => ic.path === dragging) ?? null) : null
  );

  onMount(async () => {
    try {
      icons = await invoke<DesktopIcon[]>("get_desktop_icons");
    } catch (e) {
      error = String(e);
      console.error("Failed to load desktop icons:", e);
    } finally {
      loading = false;
    }
  });

  function snapToGrid(val: number, cell: number): number {
    return Math.round(val / cell) * cell;
  }

  function onPointerDown(e: PointerEvent, icon: DesktopIcon) {
    if (e.button !== 0) return;
    e.preventDefault();
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    dragOffsetX = e.clientX - icon.x;
    dragOffsetY = e.clientY - icon.y;
    dragStartX = e.clientX;
    dragStartY = e.clientY;
    didDrag = false;
    ghostX = icon.x;
    ghostY = icon.y;
    dragging = icon.path;
  }

  function onPointerMove(e: PointerEvent) {
    if (!dragging) return;
    const dx = e.clientX - dragStartX;
    const dy = e.clientY - dragStartY;
    if (!didDrag && Math.sqrt(dx * dx + dy * dy) > DRAG_THRESHOLD) {
      didDrag = true;
    }
    if (didDrag) {
      const newX = e.clientX - dragOffsetX;
      const newY = e.clientY - dragOffsetY;
      icons = icons.map((ic) =>
        ic.path === dragging ? { ...ic, x: newX, y: newY } : ic
      );
    }
  }

  async function onPointerUp(e: PointerEvent) {
    if (!dragging) return;
    const path = dragging;
    dragging = null;

    if (!didDrag) return;

    const snappedX = snapToGrid(e.clientX - dragOffsetX, GRID_W);
    const snappedY = snapToGrid(e.clientY - dragOffsetY, GRID_H);

    icons = icons.map((ic) =>
      ic.path === path ? { ...ic, x: snappedX, y: snappedY } : ic
    );

    const positions: IconPosition[] = icons.map((ic) => ({
      path: ic.path,
      x: ic.x,
      y: ic.y,
    }));

    try {
      await invoke("save_icon_positions", { positions });
    } catch (err) {
      console.error("Failed to save positions:", err);
    }
  }

  async function handleDoubleClick(icon: DesktopIcon) {
    if (didDrag) return;
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
        class:is-dragging={icon.path === dragging && didDrag}
        style="left: {icon.x}px; top: {icon.y}px"
        onpointerdown={(e) => onPointerDown(e, icon)}
        onpointermove={onPointerMove}
        onpointerup={onPointerUp}
        ondblclick={() => handleDoubleClick(icon)}
        title={icon.name}
      >
        {#if icon.icon_data}
          <img
            class="icon-image"
            src="data:image/png;base64,{icon.icon_data}"
            alt={icon.name}
            draggable="false"
          />
        {:else}
          <div class="icon-placeholder">📄</div>
        {/if}
        <span class="icon-label">{icon.name}</span>
      </button>
    {/each}

    <!-- Drag ghost -->
    {#if dragging && didDrag && draggedIcon}
      <div
        class="icon-ghost"
        style="left: {ghostX}px; top: {ghostY}px"
        aria-hidden="true"
      >
        {#if draggedIcon.icon_data}
          <img
            class="icon-image"
            src="data:image/png;base64,{draggedIcon.icon_data}"
            alt={draggedIcon.name}
            draggable="false"
          />
        {:else}
          <div class="icon-placeholder">📄</div>
        {/if}
        <span class="icon-label">{draggedIcon.name}</span>
      </div>
    {/if}

    <div class="status-pill">
      ❄ Frostpane M0 · {icons.length} icons
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
    cursor: grab;
    color: var(--text);
    transition: background 0.15s, opacity 0.1s;
    touch-action: none;
  }

  .desktop-icon:hover {
    background: rgba(255, 255, 255, 0.12);
  }

  .desktop-icon:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .desktop-icon:focus .icon-label,
  .desktop-icon:focus-visible .icon-label {
    white-space: normal;
    word-break: break-all;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    background: rgba(0, 0, 0, 0.4);
    border-radius: 2px;
    padding: 1px 3px;
  }

  .desktop-icon.is-dragging {
    opacity: 0.35;
    cursor: grabbing;
  }

  .icon-image {
    width: 48px;
    height: 48px;
    pointer-events: none;
    image-rendering: auto;
    object-fit: contain;
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
    line-height: 1.25;
    max-width: 76px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: #fff;
    font-weight: 400;
    text-shadow:
      0 0 2px rgba(0, 0, 0, 0.9),
      0 0 4px rgba(0, 0, 0, 0.7),
      0 0 6px rgba(0, 0, 0, 0.4);
    pointer-events: none;
  }

  .icon-ghost {
    position: absolute;
    width: 80px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 6px 4px;
    border-radius: 8px;
    color: var(--text);
    pointer-events: none;
    z-index: 1000;
    transform: scale(1.1);
    transform-origin: top center;
    filter: drop-shadow(0 10px 28px rgba(0, 0, 0, 0.55));
    opacity: 0.92;
  }
</style>
