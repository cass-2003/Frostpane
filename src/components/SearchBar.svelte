<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  interface SearchResult {
    name: string;
    path: string;
    is_dir: boolean;
    size: number;
    extension: string;
  }

  let {
    visible = $bindable(false),
    onclose,
  }: {
    visible: boolean;
    onclose: () => void;
  } = $props();

  let query = $state("");
  let results = $state<SearchResult[]>([]);
  let selectedIndex = $state(0);
  let searching = $state(false);
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  let inputEl: HTMLInputElement | undefined = $state();

  $effect(() => {
    if (visible) {
      query = "";
      results = [];
      selectedIndex = 0;
      searching = false;
      setTimeout(() => inputEl?.focus(), 50);
    }
  });

  function iconForResult(r: SearchResult): string {
    if (r.is_dir) return "\u{1F4C1}";
    const ext = r.extension.toLowerCase();
    if (["exe", "msi"].includes(ext)) return "\u{1F3AE}";
    if (["lnk", "url"].includes(ext)) return "\u{1F517}";
    if (["jpg", "jpeg", "png", "gif", "bmp", "svg", "webp", "ico"].includes(ext)) return "\u{1F5BC}";
    if (["mp3", "wav", "flac", "ogg", "m4a"].includes(ext)) return "\u{1F3B5}";
    if (["mp4", "avi", "mkv", "mov", "wmv"].includes(ext)) return "\u{1F3AC}";
    if (["zip", "rar", "7z", "tar", "gz"].includes(ext)) return "\u{1F4E6}";
    if (["pdf"].includes(ext)) return "\u{1F4D5}";
    if (["doc", "docx", "txt", "rtf", "md"].includes(ext)) return "\u{1F4DD}";
    if (["xls", "xlsx", "csv"].includes(ext)) return "\u{1F4CA}";
    if (["ppt", "pptx"].includes(ext)) return "\u{1F4CA}";
    if (["js", "ts", "py", "rs", "go", "java", "cpp", "c", "h"].includes(ext)) return "\u{1F4BB}";
    return "\u{1F4C4}";
  }

  function doSearch(q: string) {
    if (debounceTimer) clearTimeout(debounceTimer);
    if (!q.trim()) {
      results = [];
      searching = false;
      return;
    }
    searching = true;
    debounceTimer = setTimeout(async () => {
      try {
        results = await invoke<SearchResult[]>("search_files", {
          query: q.trim(),
          maxResults: 50,
        });
        selectedIndex = 0;
      } catch (e) {
        console.error("Search failed:", e);
        results = [];
      } finally {
        searching = false;
      }
    }, 300);
  }

  function onInput(e: Event) {
    const val = (e.target as HTMLInputElement).value;
    query = val;
    doSearch(val);
  }

  async function openSelected() {
    const r = results[selectedIndex];
    if (!r) return;
    try {
      await invoke("open_item", { path: r.path });
      onclose();
    } catch (e) {
      console.error("Failed to open:", e);
    }
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      e.preventDefault();
      onclose();
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      if (results.length > 0) {
        selectedIndex = (selectedIndex + 1) % results.length;
        scrollToSelected();
      }
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (results.length > 0) {
        selectedIndex = (selectedIndex - 1 + results.length) % results.length;
        scrollToSelected();
      }
    } else if (e.key === "Enter") {
      e.preventDefault();
      openSelected();
    }
  }

  function scrollToSelected() {
    const el = document.querySelector(".search-result.selected");
    el?.scrollIntoView({ block: "nearest" });
  }

  function onBackdropClick(e: MouseEvent) {
    if ((e.target as HTMLElement).classList.contains("search-backdrop")) {
      onclose();
    }
  }

  function shortenPath(p: string): string {
    const home = "C:\\Users\\";
    if (p.startsWith(home)) {
      const rest = p.slice(home.length);
      const slash = rest.indexOf("\\");
      if (slash !== -1) {
        return "~" + rest.slice(slash);
      }
    }
    return p;
  }
</script>

{#if visible}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="search-backdrop" onclick={onBackdropClick} onkeydown={onKeydown}>
    <div class="search-container">
      <div class="search-input-row">
        <span class="search-icon">{"\u{1F50D}"}</span>
        <input
          bind:this={inputEl}
          type="text"
          class="search-input"
          placeholder="Search files..."
          value={query}
          oninput={onInput}
          spellcheck="false"
          autocomplete="off"
        />
        {#if searching}
          <span class="search-spinner">{"❄"}</span>
        {/if}
      </div>

      {#if results.length > 0}
        <div class="search-results">
          {#each results.slice(0, 10) as result, i (result.path)}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
              class="search-result"
              class:selected={i === selectedIndex}
              onmouseenter={() => (selectedIndex = i)}
              ondblclick={() => openSelected()}
              onclick={() => { selectedIndex = i; openSelected(); }}
            >
              <span class="result-icon">{iconForResult(result)}</span>
              <div class="result-info">
                <span class="result-name">{result.name}</span>
                <span class="result-path">{shortenPath(result.path)}</span>
              </div>
            </div>
          {/each}
        </div>
      {:else if query.trim() && !searching}
        <div class="search-empty">No results found</div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .search-backdrop {
    position: fixed;
    inset: 0;
    z-index: 9000;
    display: flex;
    justify-content: center;
    padding-top: 18vh;
    background: rgba(0, 0, 0, 0.35);
    animation: fadeIn 0.15s ease-out;
  }

  .search-container {
    width: min(580px, 90vw);
    background: rgba(22, 26, 40, 0.82);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 16px;
    backdrop-filter: blur(32px);
    box-shadow:
      0 16px 64px rgba(0, 0, 0, 0.5),
      0 0 0 1px rgba(255, 255, 255, 0.06) inset;
    overflow: hidden;
    animation: slideDown 0.2s ease-out;
    align-self: flex-start;
  }

  .search-input-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 16px 20px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  }

  .search-icon {
    font-size: 20px;
    opacity: 0.6;
    flex-shrink: 0;
  }

  .search-input {
    flex: 1;
    background: none;
    border: none;
    outline: none;
    font-size: 18px;
    color: var(--text);
    font-family: inherit;
  }

  .search-input::placeholder {
    color: var(--text-dim);
  }

  .search-spinner {
    font-size: 16px;
    animation: spin 1.5s linear infinite;
    opacity: 0.5;
  }

  .search-results {
    max-height: 400px;
    overflow-y: auto;
    padding: 6px 0;
  }

  .search-result {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 20px;
    cursor: pointer;
    transition: background 0.1s;
  }

  .search-result:hover,
  .search-result.selected {
    background: rgba(91, 141, 239, 0.18);
  }

  .result-icon {
    font-size: 22px;
    flex-shrink: 0;
    width: 32px;
    text-align: center;
  }

  .result-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .result-name {
    font-size: 14px;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .result-path {
    font-size: 11px;
    color: var(--text-dim);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .search-empty {
    padding: 24px 20px;
    text-align: center;
    color: var(--text-dim);
    font-size: 14px;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes slideDown {
    from {
      opacity: 0;
      transform: translateY(-16px) scale(0.97);
    }
    to {
      opacity: 1;
      transform: translateY(0) scale(1);
    }
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
