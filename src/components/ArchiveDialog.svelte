<script lang="ts">
  import { t } from "../lib/i18n.svelte";

  interface FileMove {
    original_path: string;
    archived_path: string;
    file_name: string;
  }

  interface ArchivePreview {
    moves: FileMove[];
    target_dir: string;
    file_count: number;
  }

  interface Props {
    visible: boolean;
    preview: ArchivePreview | null;
    onconfirm: () => void;
    oncancel: () => void;
  }

  let { visible = $bindable(false), preview, onconfirm, oncancel }: Props = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (!visible) return;
    if (e.key === "Escape") {
      e.preventDefault();
      oncancel();
    }
  }

  function handleBackdrop(e: MouseEvent) {
    if ((e.target as HTMLElement).classList.contains("archive-backdrop")) {
      oncancel();
    }
  }

  let shortDir = $derived(
    preview ? preview.target_dir.replace(/^.*[\\\/]Desktop[\\\/]/, "~/Desktop/") : ""
  );
</script>

<svelte:window onkeydown={handleKeydown} />

{#if visible && preview}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="archive-backdrop" onclick={handleBackdrop}>
    <div class="archive-panel" role="dialog" aria-label={t.archiveToFolder}>
      <div class="archive-header">
        <h2>{t.archiveToFolder}</h2>
        <button class="close-btn" onclick={oncancel} title="Close">&#x2715;</button>
      </div>

      <div class="archive-body">
        <p class="archive-summary">
          {t.archiveSummary(preview.file_count, shortDir)}
        </p>

        <div class="file-list">
          {#each preview.moves as move (move.original_path)}
            <div class="file-item">
              <span class="file-icon">&#x1F4C4;</span>
              <span class="file-name" title={move.original_path}>{move.file_name}</span>
            </div>
          {/each}
        </div>
      </div>

      <div class="archive-footer">
        <button class="btn btn-cancel" onclick={oncancel}>{t.cancel}</button>
        <button class="btn btn-archive" onclick={onconfirm}>{t.archiveAction}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .archive-backdrop {
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

  .archive-panel {
    width: 420px;
    max-height: 70vh;
    background: rgba(20, 24, 38, 0.92);
    backdrop-filter: blur(32px) saturate(1.4);
    border: 1px solid var(--glass-border);
    border-radius: 14px;
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.5), inset 0 1px 0 rgba(255, 255, 255, 0.06);
    overflow: hidden;
    animation: panel-in 0.2s ease-out;
    display: flex;
    flex-direction: column;
  }

  @keyframes panel-in {
    from { opacity: 0; transform: scale(0.95) translateY(8px); }
    to { opacity: 1; transform: scale(1) translateY(0); }
  }

  .archive-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  }

  .archive-header h2 {
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

  .archive-body {
    padding: 16px 20px;
    overflow-y: auto;
    flex: 1;
    min-height: 0;
  }

  .archive-summary {
    margin: 0 0 14px;
    font-size: 13px;
    color: var(--text-dim);
    line-height: 1.5;
  }

  .file-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-height: 280px;
    overflow-y: auto;
    padding-right: 4px;
  }

  .file-list::-webkit-scrollbar {
    width: 5px;
  }

  .file-list::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.15);
    border-radius: 3px;
  }

  .file-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    border-radius: 6px;
    font-size: 13px;
    color: var(--text);
  }

  .file-item:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .file-icon {
    font-size: 14px;
    flex-shrink: 0;
  }

  .file-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .archive-footer {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    padding: 14px 20px;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
  }

  .btn {
    padding: 8px 20px;
    border: none;
    border-radius: 8px;
    font-size: 13px;
    cursor: pointer;
    transition: opacity 0.15s, background 0.15s;
  }

  .btn-cancel {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text);
  }

  .btn-cancel:hover {
    background: rgba(255, 255, 255, 0.14);
  }

  .btn-archive {
    background: var(--accent);
    color: white;
    font-weight: 500;
  }

  .btn-archive:hover {
    opacity: 0.9;
  }
</style>
