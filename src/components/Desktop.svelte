<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import type { DesktopIcon, FenceData } from "../lib/types";
  import Fence from "./Fence.svelte";
  import ContextMenu from "./ContextMenu.svelte";
  import DesktopMenu from "./DesktopMenu.svelte";
  import Settings from "./Settings.svelte";

  const GRID_W = 90;
  const GRID_H = 100;
  const DRAG_THRESHOLD = 5;
  const DEFAULT_FENCE_W = 320;
  const DEFAULT_FENCE_H = 300;

  let icons = $state<DesktopIcon[]>([]);
  let fences = $state<FenceData[]>([]);
  let loading = $state(true);
  let error = $state("");

  // Icon drag state
  let draggingIcon = $state<string | null>(null);
  let dragOffsetX = 0;
  let dragOffsetY = 0;
  let dragStartX = 0;
  let dragStartY = 0;
  let didDrag = false;
  let dragOverFence = $state<string | null>(null);

  let draggedIconData = $derived(
    draggingIcon ? (icons.find((ic) => ic.path === draggingIcon) ?? null) : null
  );

  // Free icons: no fence_id
  let freeIcons = $derived(icons.filter((ic) => !ic.fence_id));

  // Context menu state (icon right-click)
  let ctxVisible = $state(false);
  let ctxX = $state(0);
  let ctxY = $state(0);
  let ctxPath = $state("");
  let ctxName = $state("");

  // Desktop menu state (empty space right-click)
  let deskMenuVisible = $state(false);
  let deskMenuX = $state(0);
  let deskMenuY = $state(0);

  // Settings panel
  let settingsVisible = $state(false);

  onMount(async () => {
    try {
      const [loadedIcons, loadedFences] = await Promise.all([
        invoke<DesktopIcon[]>("get_desktop_icons"),
        invoke<FenceData[]>("load_fence_layout").catch(() => []),
      ]);
      icons = loadedIcons;

      // Restore fence assignments from loaded fence data
      if (loadedFences.length > 0) {
        const fenceMap = new Map<string, string>();
        for (const f of loadedFences) {
          for (const p of f.icon_paths) {
            fenceMap.set(p, f.id);
          }
        }
        icons = icons.map((ic) => ({
          ...ic,
          fence_id: fenceMap.get(ic.path) ?? null,
        }));
        fences = loadedFences;
      }
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

  function generateId(): string {
    return "fence_" + Date.now().toString(36) + Math.random().toString(36).slice(2, 6);
  }

  // ── Icon drag ──

  function onIconPointerDown(e: PointerEvent, icon: DesktopIcon) {
    if (e.button !== 0) return;
    e.preventDefault();
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    dragOffsetX = e.clientX - icon.x;
    dragOffsetY = e.clientY - icon.y;
    dragStartX = e.clientX;
    dragStartY = e.clientY;
    didDrag = false;
    draggingIcon = icon.path;
  }

  function onPointerMove(e: PointerEvent) {
    if (!draggingIcon) return;
    const dx = e.clientX - dragStartX;
    const dy = e.clientY - dragStartY;
    if (!didDrag && Math.sqrt(dx * dx + dy * dy) > DRAG_THRESHOLD) {
      didDrag = true;
    }
    if (didDrag) {
      const newX = e.clientX - dragOffsetX;
      const newY = e.clientY - dragOffsetY;
      icons = icons.map((ic) =>
        ic.path === draggingIcon ? { ...ic, x: newX, y: newY } : ic
      );
      // Check if hovering over a fence
      dragOverFence = hitTestFence(e.clientX, e.clientY);
    }
  }

  async function onPointerUp(e: PointerEvent) {
    if (!draggingIcon) return;
    const path = draggingIcon;
    const targetFenceId = dragOverFence;
    draggingIcon = null;
    dragOverFence = null;

    if (!didDrag) return;

    if (targetFenceId) {
      // Drop onto fence
      icons = icons.map((ic) =>
        ic.path === path
          ? { ...ic, fence_id: targetFenceId, x: 0, y: 0 }
          : ic
      );
    } else {
      // Drop on free desktop
      const snappedX = snapToGrid(e.clientX - dragOffsetX, GRID_W);
      const snappedY = snapToGrid(e.clientY - dragOffsetY, GRID_H);
      icons = icons.map((ic) =>
        ic.path === path
          ? { ...ic, x: snappedX, y: snappedY, fence_id: null }
          : ic
      );
    }

    await saveAll();
  }

  function hitTestFence(mx: number, my: number): string | null {
    for (const f of fences) {
      if (f.collapsed) continue;
      if (
        mx >= f.x &&
        mx <= f.x + f.width &&
        my >= f.y &&
        my <= f.y + f.height
      ) {
        return f.id;
      }
    }
    return null;
  }

  // ── Fence callbacks ──

  function handleFenceMove(id: string, x: number, y: number) {
    fences = fences.map((f) => (f.id === id ? { ...f, x, y } : f));
  }

  function handleFenceResize(id: string, w: number, h: number) {
    fences = fences.map((f) =>
      f.id === id ? { ...f, width: w, height: h } : f
    );
  }

  function handleFenceRename(id: string, title: string) {
    fences = fences.map((f) => (f.id === id ? { ...f, title } : f));
    saveFences();
  }

  function handleFenceEmoji(id: string, emoji: string) {
    fences = fences.map((f) => (f.id === id ? { ...f, emoji } : f));
    saveFences();
  }

  function handleFenceCollapse(id: string) {
    fences = fences.map((f) =>
      f.id === id ? { ...f, collapsed: !f.collapsed } : f
    );
    saveFences();
  }

  function handleFenceDelete(id: string) {
    // Remove fence, release icons back to desktop
    icons = icons.map((ic) =>
      ic.fence_id === id ? { ...ic, fence_id: null } : ic
    );
    fences = fences.filter((f) => f.id !== id);
    saveAll();
  }

  // ── Icon interactions within fences ──

  function handleFenceIconDragStart(e: PointerEvent, icon: DesktopIcon) {
    icons = icons.map((ic) =>
      ic.path === icon.path
        ? { ...ic, fence_id: null, x: e.clientX - 40, y: e.clientY - 24 }
        : ic
    );
    dragOffsetX = 40;
    dragOffsetY = 24;
    dragStartX = e.clientX;
    dragStartY = e.clientY;
    didDrag = true;
    draggingIcon = icon.path;
  }

  function handleFenceIconClick(_icon: DesktopIcon) {
    // Selection logic can be added later
  }

  async function handleFenceIconDblClick(icon: DesktopIcon) {
    try {
      await invoke("open_item", { path: icon.path });
    } catch (e) {
      console.error("Failed to open:", e);
    }
  }

  function handleFenceIconContext(e: MouseEvent, icon: DesktopIcon) {
    e.preventDefault();
    ctxX = e.clientX;
    ctxY = e.clientY;
    ctxPath = icon.path;
    ctxName = icon.name;
    ctxVisible = true;
  }

  // ── Free icon interactions ──

  async function handleDoubleClick(icon: DesktopIcon) {
    if (didDrag) return;
    try {
      await invoke("open_item", { path: icon.path });
    } catch (e) {
      console.error("Failed to open:", e);
    }
  }

  function handleIconContext(e: MouseEvent, icon: DesktopIcon) {
    e.preventDefault();
    ctxX = e.clientX;
    ctxY = e.clientY;
    ctxPath = icon.path;
    ctxName = icon.name;
    ctxVisible = true;
  }

  // ── Desktop empty-space interactions ──

  function handleDesktopContext(e: MouseEvent) {
    if ((e.target as HTMLElement).closest("[data-fence-id]")) return;
    if ((e.target as HTMLElement).closest(".desktop-icon")) return;
    e.preventDefault();
    deskMenuX = e.clientX;
    deskMenuY = e.clientY;
    deskMenuVisible = true;
  }

  async function handleDesktopDblClick(e: MouseEvent) {
    if ((e.target as HTMLElement).closest("[data-fence-id]")) return;
    if ((e.target as HTMLElement).closest(".desktop-icon")) return;
    try {
      await getCurrentWindow().minimize();
    } catch (err) {
      console.error("Failed to minimize:", err);
    }
  }

  function handleDesktopMenuAction(id: string) {
    switch (id) {
      case "new_fence":
        createFence(deskMenuX, deskMenuY);
        break;
      case "show_all_fences":
        fences = fences.map((f) => ({ ...f, collapsed: false }));
        saveFences();
        break;
      case "hide_all_fences":
        fences = fences.map((f) => ({ ...f, collapsed: true }));
        saveFences();
        break;
      case "sort_by_name":
        icons = [...icons].sort((a, b) => a.name.localeCompare(b.name));
        saveAll();
        break;
      case "settings":
        settingsVisible = true;
        break;
    }
  }

  function createFence(x: number, y: number) {
    const newFence: FenceData = {
      id: generateId(),
      title: "New Fence",
      emoji: "📁",
      x,
      y,
      width: DEFAULT_FENCE_W,
      height: DEFAULT_FENCE_H,
      collapsed: false,
      icon_paths: [],
    };
    fences = [...fences, newFence];
    saveFences();
  }

  function handleIconDeleted(path: string) {
    icons = icons.filter((ic) => ic.path !== path);
    saveAll();
  }

  // ── Persistence ──

  function buildFencesForSave(): FenceData[] {
    return fences.map((f) => ({
      ...f,
      icon_paths: icons
        .filter((ic) => ic.fence_id === f.id)
        .map((ic) => ic.path),
    }));
  }

  async function saveFences() {
    try {
      await invoke("save_fence_layout", { fences: buildFencesForSave() });
    } catch (err) {
      console.error("Failed to save fences:", err);
    }
  }

  async function savePositions() {
    const positions = icons.map((ic) => ({
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

  async function saveAll() {
    await Promise.all([savePositions(), saveFences()]);
  }

  function iconsForFence(fenceId: string): DesktopIcon[] {
    return icons.filter((ic) => ic.fence_id === fenceId);
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="desktop-overlay"
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
  oncontextmenu={handleDesktopContext}
  ondblclick={handleDesktopDblClick}
>
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
    <!-- Fences -->
    {#each fences as fence (fence.id)}
      <Fence
        {fence}
        icons={iconsForFence(fence.id)}
        {dragOverFence}
        onmove={handleFenceMove}
        onresize={handleFenceResize}
        onrename={handleFenceRename}
        onemoji={handleFenceEmoji}
        oncollapse={handleFenceCollapse}
        ondelete={handleFenceDelete}
        oniconclick={handleFenceIconClick}
        onicondblclick={handleFenceIconDblClick}
        oniconcontextmenu={handleFenceIconContext}
        onicondragstart={handleFenceIconDragStart}
        onsave={saveFences}
      />
    {/each}

    <!-- Free icons (not in any fence) -->
    {#each freeIcons as icon (icon.path)}
      <button
        class="desktop-icon"
        class:is-dragging={icon.path === draggingIcon && didDrag}
        style="left: {icon.x}px; top: {icon.y}px"
        onpointerdown={(e) => onIconPointerDown(e, icon)}
        ondblclick={() => handleDoubleClick(icon)}
        oncontextmenu={(e) => handleIconContext(e, icon)}
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

    <div class="status-pill">
      ❄ Frostpane · {fences.length} fences · {icons.length} icons
    </div>
  {/if}
</div>

<!-- Icon context menu -->
<ContextMenu
  bind:visible={ctxVisible}
  x={ctxX}
  y={ctxY}
  targetPath={ctxPath}
  targetName={ctxName}
  onclose={() => (ctxVisible = false)}
  ondeleted={handleIconDeleted}
/>

<!-- Desktop context menu -->
<DesktopMenu
  bind:visible={deskMenuVisible}
  x={deskMenuX}
  y={deskMenuY}
  onclose={() => (deskMenuVisible = false)}
  onaction={handleDesktopMenuAction}
/>

<!-- Settings panel -->
<Settings
  bind:visible={settingsVisible}
  onclose={() => (settingsVisible = false)}
/>

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
    z-index: 500;
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
    to {
      transform: rotate(360deg);
    }
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
    z-index: 5;
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
</style>
