<script lang="ts">
  import { t } from "../lib/i18n.svelte";

  interface MenuItem {
    id: string;
    label: string;
    separator?: boolean;
  }

  interface Props {
    visible: boolean;
    x: number;
    y: number;
    onclose: () => void;
    onaction: (id: string) => void;
  }

  let { visible = $bindable(false), x, y, onclose, onaction }: Props =
    $props();

  let menuEl: HTMLDivElement | undefined = $state();
  let adjustedX = $state(0);
  let adjustedY = $state(0);
  let activeIndex = $state(-1);

  const items: MenuItem[] = $derived([
    { id: "new_fence", label: t.newFence },
    { id: "new_portal", label: t.newPortal },
    { id: "sep1", label: "", separator: true },
    { id: "show_all_fences", label: t.showAllFences },
    { id: "hide_all_fences", label: t.hideAllFences },
    { id: "sep2", label: "", separator: true },
    { id: "sort_by_name", label: t.sortByName },
    { id: "sep3", label: "", separator: true },
    { id: "prev_page", label: t.prevPage },
    { id: "next_page", label: t.nextPage },
    { id: "add_page", label: t.addPage },
    { id: "delete_page", label: t.deletePage },
    { id: "sep4", label: "", separator: true },
    { id: "scenes", label: "Scenes..." },
    { id: "rules", label: t.rules },
    { id: "settings", label: t.settings },
  ]);

  const actionItems = $derived(items.filter((m) => !m.separator));

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
      onaction(actionItems[activeIndex].id);
      onclose();
    }
  }

  function handleBackdropClick(e: MouseEvent) {
    if (menuEl && !menuEl.contains(e.target as Node)) {
      onclose();
    }
  }

  function handleAction(id: string) {
    onaction(id);
    onclose();
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
      <div class="menu-header">Frostpane</div>
      {#each items as item (item.id)}
        {#if item.separator}
          <div class="menu-separator"></div>
        {:else}
          <button
            class="menu-item"
            class:active={activeIndex === actionItems.indexOf(item)}
            role="menuitem"
            onclick={() => handleAction(item.id)}
            onmouseenter={() => {
              activeIndex = actionItems.indexOf(item);
            }}
          >
            <span class="item-label">{item.label}</span>
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
    z-index: 9998;
  }

  .context-menu {
    position: absolute;
    min-width: 200px;
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
    width: 100%;
    padding: 7px 14px;
    border: none;
    background: transparent;
    color: var(--text);
    font-size: 13px;
    cursor: default;
    text-align: left;
    transition: background 0.1s;
  }

  .menu-item:hover,
  .menu-item.active {
    background: rgba(91, 141, 239, 0.18);
  }

  .item-label {
    white-space: nowrap;
  }
</style>
