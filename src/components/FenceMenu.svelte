<script lang="ts">
  import { t } from "../lib/i18n.svelte";

  interface Props {
    visible: boolean;
    x: number;
    y: number;
    fenceId: string;
    onclose: () => void;
    onaction: (id: string, fenceId: string) => void;
  }

  let { visible = $bindable(false), x, y, fenceId, onclose, onaction }: Props = $props();

  let menuEl: HTMLDivElement | undefined = $state();
  let adjustedX = $state(0);
  let adjustedY = $state(0);

  $effect(() => {
    if (visible && menuEl) {
      const rect = menuEl.getBoundingClientRect();
      const vw = window.innerWidth;
      const vh = window.innerHeight;
      adjustedX = x + rect.width > vw ? vw - rect.width - 8 : x;
      adjustedY = y + rect.height > vh ? vh - rect.height - 8 : y;
    }
  });

  function handleBackdropClick(e: MouseEvent) {
    if (menuEl && !menuEl.contains(e.target as Node)) {
      onclose();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!visible) return;
    if (e.key === "Escape") {
      e.preventDefault();
      onclose();
    }
  }

  function act(id: string) {
    onclose();
    onaction(id, fenceId);
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
      <button class="menu-item" role="menuitem" onclick={() => act("rename")}>
        <span class="item-label">{t.rename}</span>
      </button>
      <div class="menu-separator"></div>
      <button class="menu-item" role="menuitem" onclick={() => act("sort_name")}>
        <span class="item-label">{t.sortName}</span>
      </button>
      <button class="menu-item" role="menuitem" onclick={() => act("sort_type")}>
        <span class="item-label">{t.sortType}</span>
      </button>
      <div class="menu-separator"></div>
      <button class="menu-item" role="menuitem" onclick={() => act("appearance")}>
        <span class="item-label">{t.appearance}</span>
      </button>
      <div class="menu-separator"></div>
      <button class="menu-item danger" role="menuitem" onclick={() => act("delete")}>
        <span class="item-label">{t.deleteFence}</span>
      </button>
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

  .menu-item:hover {
    background: rgba(91, 141, 239, 0.18);
  }

  .menu-item.danger {
    color: #e06060;
  }

  .menu-item.danger:hover {
    background: rgba(224, 80, 80, 0.15);
  }

  .item-label {
    white-space: nowrap;
  }
</style>
