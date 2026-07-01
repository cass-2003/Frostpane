<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { t } from "../lib/i18n.svelte";

  interface SceneProfile {
    id: string;
    name: string;
    icon_positions: Array<{ path: string; x: number; y: number }>;
    fences: Array<Record<string, unknown>>;
  }

  interface Props {
    visible: boolean;
    onclose: () => void;
    onsave: () => void;
    onload: (scene: SceneProfile) => void;
  }

  let { visible = $bindable(false), onclose, onsave, onload }: Props = $props();

  let scenes = $state<SceneProfile[]>([]);
  let newName = $state("");
  let loading = $state(true);

  $effect(() => {
    if (visible) refreshScenes();
  });

  async function refreshScenes() {
    loading = true;
    try {
      scenes = await invoke<SceneProfile[]>("list_scenes");
    } catch (e) {
      console.error("Failed to list scenes:", e);
      scenes = [];
    } finally {
      loading = false;
    }
  }

  async function saveCurrentAsScene() {
    const name = newName.trim();
    if (!name) return;
    const id = "scene_" + Date.now().toString(36);

    const [positions, fences] = await Promise.all([
      invoke<Array<{ path: string; x: number; y: number }>>("load_icon_positions").catch(() => []),
      invoke<Array<Record<string, unknown>>>("load_fence_layout").catch(() => []),
    ]);

    const profile: SceneProfile = { id, name, icon_positions: positions, fences };
    try {
      await invoke("save_scene", { profile });
      newName = "";
      onsave();
      await refreshScenes();
    } catch (e) {
      console.error("Failed to save scene:", e);
    }
  }

  async function loadScene(scene: SceneProfile) {
    onload(scene);
    onclose();
  }

  async function deleteScene(id: string) {
    try {
      await invoke("delete_scene", { id });
      await refreshScenes();
    } catch (e) {
      console.error("Failed to delete scene:", e);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (visible && e.key === "Escape") {
      e.preventDefault();
      onclose();
    }
  }

  function handleBackdrop(e: MouseEvent) {
    if ((e.target as HTMLElement).classList.contains("scene-backdrop")) {
      onclose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if visible}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="scene-backdrop" onclick={handleBackdrop}>
    <div class="scene-panel" role="dialog" aria-label="Scene Manager">
      <div class="scene-header">
        <h2>{t.scenesTitle}</h2>
        <button class="close-btn" onclick={onclose} title="Close">✕</button>
      </div>

      <div class="scene-body">
        <div class="save-section">
          <input
            class="scene-input"
            type="text"
            bind:value={newName}
            placeholder={t.sceneName}
            onkeydown={(e) => { if (e.key === "Enter") saveCurrentAsScene(); }}
          />
          <button class="save-btn" onclick={saveCurrentAsScene} disabled={!newName.trim()}>
            {t.saveScene}
          </button>
        </div>

        <div class="scene-list">
          {#if loading}
            <div class="empty">Loading...</div>
          {:else if scenes.length === 0}
            <div class="empty">{t.noScenes}</div>
          {:else}
            {#each scenes as scene (scene.id)}
              <div class="scene-item">
                <button class="scene-name" onclick={() => loadScene(scene)}>
                  {scene.name}
                </button>
                <span class="scene-meta">
                  {scene.fences.length} fences · {scene.icon_positions.length} icons
                </span>
                <button class="delete-btn" onclick={() => deleteScene(scene.id)} title="Delete">
                  ✕
                </button>
              </div>
            {/each}
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .scene-backdrop {
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

  .scene-panel {
    width: 380px;
    max-height: 70vh;
    background: rgba(20, 24, 38, 0.92);
    backdrop-filter: blur(32px) saturate(1.4);
    border: 1px solid var(--glass-border);
    border-radius: 14px;
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.5), inset 0 1px 0 rgba(255, 255, 255, 0.06);
    overflow: hidden;
    animation: panel-in 0.2s ease-out;
  }

  @keyframes panel-in {
    from { opacity: 0; transform: scale(0.95) translateY(8px); }
    to { opacity: 1; transform: scale(1) translateY(0); }
  }

  .scene-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }

  .scene-header h2 {
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
  }

  .close-btn:hover { background: rgba(255, 255, 255, 0.1); color: var(--text); }

  .scene-body {
    padding: 16px 20px;
    overflow-y: auto;
    max-height: calc(70vh - 60px);
  }

  .save-section {
    display: flex;
    gap: 8px;
    margin-bottom: 16px;
  }

  .scene-input {
    flex: 1;
    padding: 8px 12px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 8px;
    background: rgba(0, 0, 0, 0.3);
    color: var(--text);
    font-size: 13px;
    font-family: inherit;
    outline: none;
  }

  .scene-input:focus { border-color: var(--accent); }

  .save-btn {
    padding: 8px 16px;
    border: none;
    border-radius: 8px;
    background: var(--accent);
    color: white;
    font-size: 13px;
    cursor: pointer;
    white-space: nowrap;
    transition: opacity 0.15s;
  }

  .save-btn:disabled { opacity: 0.4; cursor: default; }

  .scene-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .empty {
    text-align: center;
    color: var(--text-dim);
    font-size: 13px;
    padding: 16px 0;
  }

  .scene-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    border-radius: 8px;
    transition: background 0.15s;
  }

  .scene-item:hover { background: rgba(255, 255, 255, 0.06); }

  .scene-name {
    flex: 1;
    background: none;
    border: none;
    color: var(--text);
    font-size: 14px;
    text-align: left;
    cursor: pointer;
    padding: 0;
  }

  .scene-name:hover { color: var(--accent); }

  .scene-meta {
    font-size: 11px;
    color: var(--text-dim);
    white-space: nowrap;
  }

  .delete-btn {
    background: none;
    border: none;
    color: var(--text-dim);
    font-size: 12px;
    cursor: pointer;
    padding: 2px 6px;
    border-radius: 4px;
    opacity: 0;
    transition: opacity 0.15s, color 0.15s;
  }

  .scene-item:hover .delete-btn { opacity: 1; }
  .delete-btn:hover { color: #e04060; }
</style>
