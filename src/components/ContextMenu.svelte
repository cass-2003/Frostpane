<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  interface QuickMenuItem {
    id: string;
    label: string;
    separator: boolean;
    shortcut: string;
  }

  interface Props {
    visible: boolean;
    x: number;
    y: number;
    targetPath: string;
    targetName: string;
    onclose: () => void;
    onrename?: (path: string) => void;
    ondeleted?: (path: string) => void;
  }

  let {
    visible = $bindable(false),
    x,
    y,
    targetPath,
    targetName,
    onclose,
    onrename,
    ondeleted,
  }: Props = $props();

  let menuItems = $state<QuickMenuItem[]>([]);
  let menuEl: HTMLDivElement | undefined = $state();
  let adjustedX = $state(0);
  let adjustedY = $state(0);
  let activeIndex = $state(-1);

  onMount(async () => {
    try {
      menuItems = await invoke<QuickMenuItem[]>("get_quick_menu_items");
    } catch (e) {
      console.error("Failed to load menu items:", e);
    }
  });

  $effect(() => {
    if (visible && menuEl) {
      const rect = menuEl.getBoundingClientRect();
      const vw = window.innerWidth;
      const vh = window.innerHeight;
      adjustedX = x + rect.width > vw ? vw - rect.width - 8 : x;
      adjustedY = y + rect.height > vh ? vh - rect.height - 8 : y;
      activeIndex = -1;
    }
  });

  function handleKeydown(e: KeyboardEvent) {
    if (!visible) return;

    const actionItems = menuItems.filter((m) => !m.separator);

    if (e.key === "Escape") {
      e.preventDefault();
      onclose();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      activeIndex = (activeIndex + 1) % actionItems.length;
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      activeIndex = (activeIndex - 1 + actionItems.length) % actionItems.length;
    } else if (e.key === "Enter" && activeIndex >= 0) {
      e.preventDefault();
      handleAction(actionItems[activeIndex].id);
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (menuEl && !menuEl.contains(e.target as Node)) {
      onclose();
    }
  }

  async function handleAction(id: string) {
    onclose();

    try {
      switch (id) {
        case "open":
          await invoke("open_item", { path: targetPath });
          break;
        case "run_as_admin":
          await invoke("open_item_admin", { path: targetPath });
          break;
        case "open_location":
          await invoke("open_file_location", { path: targetPath });
          break;
        case "rename":
          onrename?.(targetPath);
          break;
        case "delete":
          await invoke("delete_to_recycle_bin", { path: targetPath });
          ondeleted?.(targetPath);
          break;
        case "copy_path":
          await invoke("copy_path_to_clipboard", { path: targetPath });
          break;
        case "more_actions":
          await invoke("show_native_context_menu", {
            path: targetPath,
            screenX: Math.round(x),
            screenY: Math.round(y),
          });
          break;
      }
    } catch (e) {
      console.error(`Context menu action "${id}" failed:`, e);
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if visible}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="context-backdrop" onmousedown={handleBackdropClick}>
    <div
      class="context-menu"
      bind:this={menuEl}
      style="left: {adjustedX}px; top: {adjustedY}px"
      role="menu"
    >
      <div class="menu-header">{targetName}</div>
      {#each menuItems as item, i (item.id)}
        {#if item.separator}
          <div class="menu-separator"></div>
        {:else}
          <button
            class="menu-item"
            class:active={activeIndex ===
              menuItems.filter((m) => !m.separator).indexOf(item)}
            role="menuitem"
            onclick={() => handleAction(item.id)}
            onmouseenter={() => {
              activeIndex = menuItems
                .filter((m) => !m.separator)
                .indexOf(item);
            }}
          >
            <span class="item-label">{item.label}</span>
            {#if item.shortcut}
              <span class="item-shortcut">{item.shortcut}</span>
            {/if}
          </button>
        {/if}
      {/each}
    </div>
  </div>
{/if}

<style>
  .context-backdrop {
    position: fixed;
    inset: 0;
    z-index: 9999;
  }

  .context-menu {
    position: absolute;
    min-width: 220px;
    padding: 6px 0;
    background: var(--glass-bg);
    backdrop-filter: blur(24px) saturate(1.4);
    border: 1px solid var(--glass-border);
    border-radius: 10px;
    box-shadow:
      0 12px 40px rgba(0, 0, 0, 0.45),
      0 0 1px rgba(255, 255, 255, 0.1) inset;
    animation: menu-in 0.12s ease-out;
  }

  @keyframes menu-in {
    from {
      opacity: 0;
      transform: scale(0.96) translateY(-4px);
    }
    to {
      opacity: 1;
      transform: scale(1) translateY(0);
    }
  }

  .menu-header {
    padding: 6px 14px 4px;
    font-size: 11px;
    color: var(--text-dim);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 260px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    margin-bottom: 2px;
  }

  .menu-separator {
    height: 1px;
    margin: 4px 12px;
    background: var(--glass-border);
  }

  .menu-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 7px 14px;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: 13px;
    cursor: default;
    text-align: left;
    gap: 24px;
    transition: background 0.1s;
  }

  .menu-item:hover,
  .menu-item.active {
    background: rgba(91, 141, 239, 0.18);
  }

  .item-label {
    white-space: nowrap;
  }

  .item-shortcut {
    font-size: 11px;
    color: var(--text-dim);
    white-space: nowrap;
  }
</style>
