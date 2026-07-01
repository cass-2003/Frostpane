<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  interface Props {
    visible: boolean;
    onclose: () => void;
  }

  let { visible = $bindable(false), onclose }: Props = $props();

  let autostart = $state(false);
  let loading = $state(true);

  onMount(async () => {
    try {
      autostart = await invoke<boolean>("is_autostart_enabled");
    } catch (e) {
      console.error("Failed to check autostart:", e);
    } finally {
      loading = false;
    }
  });

  async function toggleAutostart() {
    const newVal = !autostart;
    try {
      await invoke("set_autostart", { enabled: newVal });
      autostart = newVal;
    } catch (e) {
      console.error("Failed to set autostart:", e);
    }
  }

  function handleBackdrop(e: MouseEvent) {
    if ((e.target as HTMLElement).classList.contains("settings-backdrop")) {
      onclose();
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (visible && e.key === "Escape") {
      e.preventDefault();
      onclose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if visible}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="settings-backdrop" onclick={handleBackdrop}>
    <div class="settings-panel" role="dialog" aria-label="Settings">
      <div class="settings-header">
        <h2>Settings</h2>
        <button class="close-btn" onclick={onclose} title="Close">✕</button>
      </div>

      <div class="settings-body">
        {#if loading}
          <div class="loading">Loading...</div>
        {:else}
          <section class="settings-section">
            <h3>General</h3>
            <label class="setting-row">
              <span class="setting-label">Launch at startup</span>
              <button
                class="toggle-switch"
                class:active={autostart}
                onclick={toggleAutostart}
                role="switch"
                aria-checked={autostart}
              >
                <span class="toggle-knob"></span>
              </button>
            </label>
          </section>

          <section class="settings-section">
            <h3>About</h3>
            <div class="about-info">
              <p><strong>Frostpane</strong> v0.1.0</p>
              <p class="dim">A smarter Windows desktop organizer</p>
              <p class="dim">Apache-2.0 License</p>
            </div>
          </section>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .settings-backdrop {
    position: fixed;
    inset: 0;
    z-index: 10000;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.4);
    animation: fade-in 0.15s ease-out;
  }

  @keyframes fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .settings-panel {
    width: 400px;
    max-height: 80vh;
    background: rgba(20, 24, 38, 0.92);
    backdrop-filter: blur(32px) saturate(1.4);
    border: 1px solid var(--glass-border);
    border-radius: 14px;
    box-shadow:
      0 24px 64px rgba(0, 0, 0, 0.5),
      inset 0 1px 0 rgba(255, 255, 255, 0.06);
    overflow: hidden;
    animation: panel-in 0.2s ease-out;
  }

  @keyframes panel-in {
    from { opacity: 0; transform: scale(0.95) translateY(8px); }
    to { opacity: 1; transform: scale(1) translateY(0); }
  }

  .settings-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }

  .settings-header h2 {
    font-size: 16px;
    font-weight: 600;
    color: var(--text);
    margin: 0;
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-dim);
    font-size: 16px;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 6px;
    transition: background 0.15s, color 0.15s;
  }

  .close-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text);
  }

  .settings-body {
    padding: 8px 0;
    overflow-y: auto;
    max-height: calc(80vh - 60px);
  }

  .loading {
    padding: 24px;
    text-align: center;
    color: var(--text-dim);
  }

  .settings-section {
    padding: 8px 20px 16px;
  }

  .settings-section h3 {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-dim);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin: 0 0 10px;
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 0;
    cursor: pointer;
  }

  .setting-label {
    font-size: 14px;
    color: var(--text);
  }

  .toggle-switch {
    position: relative;
    width: 44px;
    height: 24px;
    border-radius: 12px;
    border: none;
    background: rgba(255, 255, 255, 0.15);
    cursor: pointer;
    padding: 0;
    transition: background 0.2s;
  }

  .toggle-switch.active {
    background: var(--accent);
  }

  .toggle-knob {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: white;
    transition: transform 0.2s;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
  }

  .toggle-switch.active .toggle-knob {
    transform: translateX(20px);
  }

  .about-info {
    padding: 4px 0;
  }

  .about-info p {
    margin: 4px 0;
    font-size: 13px;
    color: var(--text);
  }

  .about-info .dim {
    color: var(--text-dim);
    font-size: 12px;
  }
</style>
