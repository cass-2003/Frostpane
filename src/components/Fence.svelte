<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { DesktopIcon, FenceData, PortalItem } from "../lib/types";
  import { settings } from "../lib/settings.svelte";
  import { t } from "../lib/i18n.svelte";

  interface Props {
    fence: FenceData;
    icons: DesktopIcon[];
    dragOverFence: string | null;
    onmove: (id: string, x: number, y: number) => void;
    onresize: (id: string, w: number, h: number) => void;
    onrename: (id: string, title: string) => void;
    onemoji: (id: string, emoji: string) => void;
    oncollapse: (id: string) => void;
    onfencemenu: (e: MouseEvent, id: string) => void;
    oniconclick: (icon: DesktopIcon) => void;
    onicondblclick: (icon: DesktopIcon) => void;
    oniconcontextmenu: (e: MouseEvent, icon: DesktopIcon) => void;
    onicondragstart: (e: PointerEvent, icon: DesktopIcon) => void;
    onsave: () => void;
    renameToken?: number;
  }

  let {
    fence,
    icons,
    dragOverFence,
    onmove,
    onresize,
    onrename,
    onemoji,
    oncollapse,
    onfencemenu,
    oniconclick,
    onicondblclick,
    oniconcontextmenu,
    onicondragstart,
    onsave,
    renameToken,
  }: Props = $props();

  const DRAG_THRESHOLD = 5;
  const MIN_W = 180;
  const MIN_H = 120;

  const EMOJI_PALETTE = [
    "📁", "📂", "🎮", "🎵", "🎬", "📷", "💼", "🔧",
    "📝", "📊", "🌐", "💡", "🎨", "📦", "🔒", "⭐",
    "🚀", "🏠", "📌", "🗂️", "💻", "🧩", "📚", "🔥",
  ];

  let editing = $state(false);
  let editValue = $state("");
  let titleInput: HTMLInputElement | undefined = $state();
  let showEmojiPicker = $state(false);

  // Drag (move) state
  let moving = $state(false);
  let moveStartX = 0;
  let moveStartY = 0;
  let moveOffsetX = 0;
  let moveOffsetY = 0;
  let didMove = false;

  // Resize state
  let resizing = $state(false);
  let resizeStartX = 0;
  let resizeStartY = 0;
  let resizeStartW = 0;
  let resizeStartH = 0;

  let isDragOver = $derived(dragOverFence === fence.id);
  let isPortal = $derived(!!fence.portalPath);
  let portalItems = $state<PortalItem[]>([]);
  let portalLoading = $state(false);

  $effect(() => {
    if (fence.portalPath) {
      loadPortalContents(fence.portalPath);
    }
  });

  async function loadPortalContents(folderPath: string) {
    portalLoading = true;
    try {
      portalItems = await invoke<PortalItem[]>("list_portal_contents", { folderPath });
    } catch (e) {
      console.error("Failed to load portal contents:", e);
      portalItems = [];
    } finally {
      portalLoading = false;
    }
  }

  async function openPortalItem(item: PortalItem) {
    try {
      await invoke("open_item", { path: item.path });
    } catch (e) {
      console.error("Failed to open portal item:", e);
    }
  }

  let fenceStyleVars = $derived.by(() => {
    const s = fence.style;
    const parts: string[] = [];
    if (s?.bgColor) parts.push(`--fence-bg:${s.bgColor}`);
    if (s?.borderColor) parts.push(`--fence-border:${s.borderColor}`);
    if (s?.borderRadius !== undefined) parts.push(`--fence-radius:${s.borderRadius}px`);
    if (s?.opacity !== undefined) parts.push(`opacity:${s.opacity}`);
    return parts.join(';');
  });

  $effect(() => {
    if (renameToken && renameToken > 0) startRename();
  });

  function startMove(e: PointerEvent) {
    if (e.button !== 0 || editing) return;
    e.preventDefault();
    e.stopPropagation();
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    moveOffsetX = e.clientX - fence.x;
    moveOffsetY = e.clientY - fence.y;
    moveStartX = e.clientX;
    moveStartY = e.clientY;
    didMove = false;
    moving = true;
  }

  function onMovePointer(e: PointerEvent) {
    if (!moving) return;
    const dx = e.clientX - moveStartX;
    const dy = e.clientY - moveStartY;
    if (!didMove && Math.sqrt(dx * dx + dy * dy) > DRAG_THRESHOLD) {
      didMove = true;
    }
    if (didMove) {
      onmove(fence.id, e.clientX - moveOffsetX, e.clientY - moveOffsetY);
    }
  }

  function onMoveUp(_e: PointerEvent) {
    if (moving && didMove) onsave();
    moving = false;
  }

  function startResize(e: PointerEvent) {
    if (e.button !== 0) return;
    e.preventDefault();
    e.stopPropagation();
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    resizeStartX = e.clientX;
    resizeStartY = e.clientY;
    resizeStartW = fence.width;
    resizeStartH = fence.height;
    resizing = true;
  }

  function onResizePointer(e: PointerEvent) {
    if (!resizing) return;
    const newW = Math.max(MIN_W, resizeStartW + (e.clientX - resizeStartX));
    const newH = Math.max(MIN_H, resizeStartH + (e.clientY - resizeStartY));
    onresize(fence.id, newW, newH);
  }

  function onResizeUp(_e: PointerEvent) {
    if (resizing) onsave();
    resizing = false;
  }

  function startRename() {
    editValue = fence.title;
    editing = true;
    requestAnimationFrame(() => {
      titleInput?.select();
    });
  }

  function commitRename() {
    const val = editValue.trim();
    if (val && val !== fence.title) {
      onrename(fence.id, val);
    }
    editing = false;
  }

  function onTitleKeydown(e: KeyboardEvent) {
    if (e.key === "Enter") {
      e.preventDefault();
      commitRename();
    } else if (e.key === "Escape") {
      editing = false;
    }
  }

  function pickEmoji(emoji: string) {
    onemoji(fence.id, emoji);
    showEmojiPicker = false;
  }

  function toggleEmojiPicker(e: MouseEvent) {
    e.stopPropagation();
    showEmojiPicker = !showEmojiPicker;
  }

  // Icon drag-out from fence
  let iconDragPath = $state<string | null>(null);
  let iconDragStartX = 0;
  let iconDragStartY = 0;
  let iconDidDrag = false;

  function onIconPointerDown(e: PointerEvent, icon: DesktopIcon) {
    if (e.button !== 0) return;
    e.stopPropagation();
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
    iconDragStartX = e.clientX;
    iconDragStartY = e.clientY;
    iconDidDrag = false;
    iconDragPath = icon.path;
  }

  function onIconPointerMove(e: PointerEvent) {
    if (!iconDragPath) return;
    const dx = e.clientX - iconDragStartX;
    const dy = e.clientY - iconDragStartY;
    if (!iconDidDrag && Math.sqrt(dx * dx + dy * dy) > DRAG_THRESHOLD) {
      iconDidDrag = true;
      const icon = icons.find((ic) => ic.path === iconDragPath);
      if (icon) {
        onicondragstart(e, icon);
      }
      iconDragPath = null;
    }
  }

  function onIconPointerUp(_e: PointerEvent) {
    iconDragPath = null;
  }

  function handleHeaderContext(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    onfencemenu(e, fence.id);
  }
</script>

<svelte:window
  onclick={() => { showEmojiPicker = false; }}
/>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="fence"
  class:collapsed={fence.collapsed}
  class:drag-over={isDragOver}
  class:is-moving={moving && didMove}
  class:is-resizing={resizing}
  style="left:{fence.x}px; top:{fence.y}px; width:{fence.width}px;{fenceStyleVars}"
  data-fence-id={fence.id}
>
  <!-- Header -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fence-header"
    onpointerdown={startMove}
    onpointermove={onMovePointer}
    onpointerup={onMoveUp}
    ondblclick={() => oncollapse(fence.id)}
    oncontextmenu={handleHeaderContext}
  >
    <button class="emoji-btn" onclick={toggleEmojiPicker} title={t.changeIcon}>
      {fence.emoji}
    </button>

    {#if editing}
      <input
        class="title-input"
        type="text"
        bind:this={titleInput}
        bind:value={editValue}
        onblur={commitRename}
        onkeydown={onTitleKeydown}
        onclick={(e) => e.stopPropagation()}
        onpointerdown={(e) => e.stopPropagation()}
      />
    {:else}
      <span class="fence-title" ondblclick={(e) => { e.stopPropagation(); startRename(); }}>
        {fence.title}
      </span>
    {/if}

    <span class="icon-count">{isPortal ? portalItems.length : icons.length}</span>

    <button
      class="collapse-btn"
      onclick={(e) => { e.stopPropagation(); oncollapse(fence.id); }}
      title={fence.collapsed ? t.expand : t.collapse}
    >
      {fence.collapsed ? "▸" : "▾"}
    </button>
  </div>

  <!-- Emoji picker dropdown -->
  {#if showEmojiPicker}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="emoji-picker" onclick={(e) => e.stopPropagation()}>
      {#each EMOJI_PALETTE as em}
        <button class="emoji-option" onclick={() => pickEmoji(em)}>{em}</button>
      {/each}
    </div>
  {/if}

  <!-- Portal path subtitle -->
  {#if isPortal && !fence.collapsed}
    <div class="portal-path-bar">
      <span class="portal-path" title={fence.portalPath ?? ""}>{fence.portalPath}</span>
      <button class="portal-action" onclick={() => fence.portalPath && loadPortalContents(fence.portalPath)} title={t.refreshPortal}>↻</button>
    </div>
  {/if}

  <!-- Body -->
  <div
    class="fence-body"
    style="height:{fence.collapsed ? 0 : fence.height - (isPortal ? 60 : 38)}px"
  >
    {#if !fence.collapsed}
      {#if isPortal}
        {#if portalLoading}
          <div class="portal-loading">Loading...</div>
        {:else if (fence.viewMode ?? "grid") === "list"}
          <div class="fence-list">
            {#each portalItems as item (item.path)}
              <button
                class="list-item"
                title={item.path}
                ondblclick={() => openPortalItem(item)}
              >
                {#if item.iconData}
                  <img class="list-icon" src="data:image/png;base64,{item.iconData}" alt={item.name} draggable="false" width="24" height="24" />
                {:else}
                  <span class="list-icon-placeholder">{item.is_dir ? "📁" : "📄"}</span>
                {/if}
                <span class="list-name">{item.name}</span>
              </button>
            {/each}
          </div>
        {:else}
          <div class="fence-icons">
            {#each portalItems as item (item.path)}
              <button
                class="fence-icon"
                title={item.name}
                ondblclick={() => openPortalItem(item)}
              >
                {#if item.iconData}
                  <img
                    class="icon-image"
                    src="data:image/png;base64,{item.iconData}"
                    alt={item.name}
                    draggable="false"
                    style="width: {settings.iconSize}px; height: {settings.iconSize}px"
                  />
                {:else}
                  <div class="icon-placeholder" style="width: {settings.iconSize}px; height: {settings.iconSize}px">{item.is_dir ? "📁" : "📄"}</div>
                {/if}
                {#if !settings.hideIconLabels}<span class="icon-label">{item.name}</span>{/if}
              </button>
            {/each}
          </div>
        {/if}
      {:else if (fence.viewMode ?? "grid") === "list"}
        <div class="fence-list">
          {#each icons as icon (icon.path)}
            <button
              class="list-item"
              title={icon.name}
              onpointerdown={(e) => onIconPointerDown(e, icon)}
              onpointermove={onIconPointerMove}
              onpointerup={onIconPointerUp}
              onclick={() => { if (!iconDidDrag) oniconclick(icon); }}
              ondblclick={() => { if (!iconDidDrag) onicondblclick(icon); }}
              oncontextmenu={(e) => oniconcontextmenu(e, icon)}
            >
              {#if icon.icon_data}
                <img class="list-icon" src="data:image/png;base64,{icon.icon_data}" alt={icon.name} draggable="false" width="24" height="24" />
              {:else}
                <span class="list-icon-placeholder">📄</span>
              {/if}
              <span class="list-name">{icon.name}</span>
            </button>
          {/each}
        </div>
      {:else}
        <div class="fence-icons">
          {#each icons as icon (icon.path)}
            <button
              class="fence-icon"
              title={icon.name}
              onpointerdown={(e) => onIconPointerDown(e, icon)}
              onpointermove={onIconPointerMove}
              onpointerup={onIconPointerUp}
              onclick={() => { if (!iconDidDrag) oniconclick(icon); }}
              ondblclick={() => { if (!iconDidDrag) onicondblclick(icon); }}
              oncontextmenu={(e) => oniconcontextmenu(e, icon)}
            >
              {#if icon.icon_data}
                <img
                  class="icon-image"
                  src="data:image/png;base64,{icon.icon_data}"
                  alt={icon.name}
                  draggable="false"
                  style="width: {settings.iconSize}px; height: {settings.iconSize}px"
                />
              {:else}
                <div class="icon-placeholder" style="width: {settings.iconSize}px; height: {settings.iconSize}px">📄</div>
              {/if}
              {#if !settings.hideIconLabels}<span class="icon-label">{icon.name}</span>{/if}
            </button>
          {/each}
        </div>
      {/if}
    {/if}
  </div>

  <!-- Resize handle -->
  {#if !fence.collapsed}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="resize-handle"
      onpointerdown={startResize}
      onpointermove={onResizePointer}
      onpointerup={onResizeUp}
    ></div>
  {/if}
</div>

<style>
  .fence {
    position: absolute;
    display: flex;
    flex-direction: column;
    background: var(--fence-bg, var(--glass-bg));
    backdrop-filter: blur(20px) saturate(1.3);
    border: 1px solid var(--fence-border, var(--glass-border));
    border-radius: var(--fence-radius, 12px);
    box-shadow:
      0 8px 32px rgba(0, 0, 0, 0.35),
      inset 0 1px 0 rgba(255, 255, 255, 0.08);
    z-index: 10;
    transition: box-shadow 0.2s, border-color 0.2s;
    overflow: visible;
  }

  .fence.drag-over {
    border-color: var(--accent);
    box-shadow:
      0 8px 32px rgba(0, 0, 0, 0.35),
      0 0 0 2px var(--accent-glow),
      inset 0 1px 0 rgba(255, 255, 255, 0.08);
  }

  .fence.is-moving {
    opacity: 0.88;
    z-index: 100;
    cursor: grabbing;
  }

  .fence.is-resizing {
    z-index: 100;
  }

  .fence-header {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 38px;
    padding: 0 8px 0 6px;
    cursor: grab;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    background: linear-gradient(
      180deg,
      rgba(255, 255, 255, 0.07) 0%,
      transparent 100%
    );
    border-radius: var(--fence-radius, 12px) var(--fence-radius, 12px) 0 0;
    flex-shrink: 0;
    touch-action: none;
  }

  .fence.collapsed .fence-header {
    border-radius: var(--fence-radius, 12px);
    border-bottom: none;
  }

  .fence-header:active {
    cursor: grabbing;
  }

  .emoji-btn {
    background: none;
    border: none;
    font-size: 16px;
    cursor: pointer;
    padding: 2px 4px;
    border-radius: 4px;
    line-height: 1;
    transition: background 0.15s;
  }

  .emoji-btn:hover {
    background: rgba(255, 255, 255, 0.12);
  }

  .fence-title {
    flex: 1;
    font-size: 12px;
    font-weight: 500;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    cursor: text;
    padding: 2px 0;
    min-width: 0;
  }

  .title-input {
    flex: 1;
    font-size: 12px;
    font-weight: 500;
    color: var(--text);
    background: rgba(0, 0, 0, 0.3);
    border: 1px solid var(--accent);
    border-radius: 4px;
    padding: 2px 6px;
    outline: none;
    min-width: 0;
    font-family: inherit;
  }

  .icon-count {
    font-size: 10px;
    color: var(--text-dim);
    background: rgba(255, 255, 255, 0.08);
    padding: 1px 6px;
    border-radius: 8px;
    flex-shrink: 0;
  }

  .collapse-btn {
    background: none;
    border: none;
    color: var(--text-dim);
    font-size: 14px;
    cursor: pointer;
    padding: 2px 4px;
    border-radius: 4px;
    line-height: 1;
    flex-shrink: 0;
    transition: background 0.15s, color 0.15s;
  }

  .collapse-btn:hover {
    background: rgba(255, 255, 255, 0.12);
    color: var(--text);
  }

  .emoji-picker {
    position: absolute;
    top: 40px;
    left: 6px;
    z-index: 200;
    display: grid;
    grid-template-columns: repeat(8, 1fr);
    gap: 2px;
    padding: 8px;
    background: rgba(20, 24, 38, 0.92);
    backdrop-filter: blur(24px);
    border: 1px solid var(--glass-border);
    border-radius: 10px;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.5);
    animation: picker-in 0.12s ease-out;
  }

  @keyframes picker-in {
    from { opacity: 0; transform: translateY(-4px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .emoji-option {
    background: none;
    border: none;
    font-size: 18px;
    padding: 4px;
    border-radius: 6px;
    cursor: pointer;
    line-height: 1;
    transition: background 0.1s;
  }

  .emoji-option:hover {
    background: rgba(91, 141, 239, 0.25);
  }

  .fence-body {
    overflow: hidden;
    transition: height 0.25s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .fence-icons {
    display: grid;
    grid-template-columns: repeat(auto-fill, 80px);
    gap: 2px;
    padding: 8px;
    align-content: start;
    height: 100%;
    overflow-y: auto;
    overflow-x: hidden;
  }

  .fence-icons::-webkit-scrollbar {
    width: 4px;
  }

  .fence-icons::-webkit-scrollbar-track {
    background: transparent;
  }

  .fence-icons::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.15);
    border-radius: 2px;
  }

  .fence-icon {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 6px 4px;
    border-radius: 8px;
    border: none;
    background: transparent;
    cursor: pointer;
    color: var(--text);
    transition: background 0.15s;
    width: 80px;
  }

  .fence-icon:hover {
    background: rgba(255, 255, 255, 0.12);
  }

  .fence-icon:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
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
    max-width: 72px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: #fff;
    font-weight: 400;
    text-shadow:
      0 0 2px rgba(0, 0, 0, 0.9),
      0 0 4px rgba(0, 0, 0, 0.7);
    pointer-events: none;
  }

  .resize-handle {
    position: absolute;
    right: 0;
    bottom: 0;
    width: 16px;
    height: 16px;
    cursor: nwse-resize;
    touch-action: none;
    z-index: 20;
  }

  .resize-handle::after {
    content: "";
    position: absolute;
    right: 4px;
    bottom: 4px;
    width: 8px;
    height: 8px;
    border-right: 2px solid rgba(255, 255, 255, 0.25);
    border-bottom: 2px solid rgba(255, 255, 255, 0.25);
    border-radius: 0 0 3px 0;
    transition: border-color 0.15s;
  }

  .resize-handle:hover::after {
    border-color: var(--accent);
  }

  .fence-list {
    display: flex;
    flex-direction: column;
    gap: 1px;
    padding: 4px;
    height: 100%;
    overflow-y: auto;
  }

  .fence-list::-webkit-scrollbar {
    width: 4px;
  }

  .fence-list::-webkit-scrollbar-track {
    background: transparent;
  }

  .fence-list::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.15);
    border-radius: 2px;
  }

  .list-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 8px;
    border-radius: 4px;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: 12px;
    cursor: pointer;
    text-align: left;
    width: 100%;
    transition: background 0.1s;
  }

  .list-item:hover {
    background: rgba(255, 255, 255, 0.1);
  }

  .list-icon {
    width: 24px;
    height: 24px;
    flex-shrink: 0;
    object-fit: contain;
    pointer-events: none;
  }

  .list-icon-placeholder {
    width: 24px;
    height: 24px;
    flex-shrink: 0;
    display: grid;
    place-items: center;
    font-size: 14px;
  }

  .list-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .portal-path-bar {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 0 8px;
    height: 22px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    background: rgba(0, 0, 0, 0.15);
    flex-shrink: 0;
  }

  .portal-path {
    flex: 1;
    font-size: 10px;
    color: var(--text-dim);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    min-width: 0;
  }

  .portal-action {
    background: none;
    border: none;
    color: var(--text-dim);
    font-size: 14px;
    cursor: pointer;
    padding: 0 2px;
    border-radius: 3px;
    line-height: 1;
    flex-shrink: 0;
    transition: color 0.15s, background 0.15s;
  }

  .portal-action:hover {
    color: var(--text);
    background: rgba(255, 255, 255, 0.1);
  }

  .portal-loading {
    display: grid;
    place-items: center;
    height: 100%;
    color: var(--text-dim);
    font-size: 12px;
  }
</style>
