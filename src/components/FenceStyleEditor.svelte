<script lang="ts">
  import type { FenceStyle } from "../lib/types";
  import { t } from "../lib/i18n.svelte";

  interface Props {
    visible: boolean;
    x: number;
    y: number;
    style: FenceStyle;
    onchange: (style: FenceStyle) => void;
    onclose: () => void;
  }

  let { visible = $bindable(false), x, y, style, onchange, onclose }: Props = $props();

  const PRESETS = [
    { label: "Glass",    color: undefined },
    { label: "Navy",     color: "rgba(15, 30, 70, 0.65)" },
    { label: "Purple",   color: "rgba(55, 20, 80, 0.65)" },
    { label: "Teal",     color: "rgba(10, 60, 65, 0.65)" },
    { label: "Crimson",  color: "rgba(80, 15, 30, 0.65)" },
    { label: "Obsidian", color: "rgba(8, 8, 15, 0.82)" },
  ] as const;

  const PREVIEW_COLORS: Record<string, string> = {
    "Navy":     "rgba(15, 30, 70, 0.9)",
    "Purple":   "rgba(55, 20, 80, 0.9)",
    "Teal":     "rgba(10, 60, 65, 0.9)",
    "Crimson":  "rgba(80, 15, 30, 0.9)",
    "Obsidian": "rgba(8, 8, 15, 0.95)",
  };

  let panelEl: HTMLDivElement | undefined = $state();
  let adjustedX = $state(0);
  let adjustedY = $state(0);

  $effect(() => {
    if (visible && panelEl) {
      const rect = panelEl.getBoundingClientRect();
      const vw = window.innerWidth;
      const vh = window.innerHeight;
      adjustedX = x + rect.width > vw ? vw - rect.width - 8 : x;
      adjustedY = y + rect.height > vh ? vh - rect.height - 8 : y;
    }
  });

  // Transparency: 0 = opaque (opacity 1), 100 = fully transparent (opacity 0)
  let transparencyPct = $derived(Math.round((1 - (style.opacity ?? 1)) * 100));
  let radiusPx = $derived(style.borderRadius ?? 12);

  function setTransparency(val: number) {
    onchange({ ...style, opacity: parseFloat((1 - val / 100).toFixed(2)) });
  }

  function setRadius(val: number) {
    onchange({ ...style, borderRadius: val });
  }

  function setPreset(color: string | undefined) {
    if (color === undefined) {
      const { bgColor: _, ...rest } = style;
      onchange(rest);
    } else {
      onchange({ ...style, bgColor: color });
    }
  }

  function resetAll() {
    onchange({});
  }

  function handleBackdropClick(e: MouseEvent) {
    if (panelEl && !panelEl.contains(e.target as Node)) onclose();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!visible) return;
    if (e.key === "Escape") { e.preventDefault(); onclose(); }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if visible}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="backdrop" onmousedown={handleBackdropClick}>
    <div
      class="panel"
      bind:this={panelEl}
      style="left:{adjustedX}px; top:{adjustedY}px"
    >
      <div class="panel-header">
        <span class="panel-title">{t.fenceStyle}</span>
        <button class="close-btn" onclick={onclose} title="Close">✕</button>
      </div>

      <div class="section-label">Color</div>
      <div class="swatches">
        {#each PRESETS as preset}
          <button
            class="swatch"
            class:active={
              preset.color === undefined
                ? !style.bgColor
                : style.bgColor === preset.color
            }
            style={preset.color ? `background:${PREVIEW_COLORS[preset.label] ?? preset.color}` : ""}
            title={preset.label}
            onclick={() => setPreset(preset.color)}
          >
            {#if preset.color === undefined}
              <span class="glass-icon">◇</span>
            {/if}
          </button>
        {/each}
      </div>

      <div class="section-label">
        {t.transparency}
        <span class="value-badge">{transparencyPct}%</span>
      </div>
      <input
        class="slider"
        type="range"
        min="0"
        max="95"
        step="1"
        value={transparencyPct}
        oninput={(e) => setTransparency(Number((e.currentTarget as HTMLInputElement).value))}
      />

      <div class="section-label">
        {t.borderRadius}
        <span class="value-badge">{radiusPx}px</span>
      </div>
      <input
        class="slider"
        type="range"
        min="0"
        max="24"
        step="1"
        value={radiusPx}
        oninput={(e) => setRadius(Number((e.currentTarget as HTMLInputElement).value))}
      />

      <div class="footer">
        <button class="reset-btn" onclick={resetAll}>{t.resetStyle}</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    z-index: 9998;
  }

  .panel {
    position: absolute;
    width: 224px;
    padding: 0 0 10px;
    background: rgba(18, 22, 36, 0.94);
    backdrop-filter: blur(28px) saturate(1.4);
    border: 1px solid rgba(255, 255, 255, 0.14);
    border-radius: 12px;
    box-shadow:
      0 16px 48px rgba(0, 0, 0, 0.55),
      0 0 1px rgba(255, 255, 255, 0.08) inset;
    animation: panel-in 0.14s ease-out;
    z-index: 9999;
  }

  @keyframes panel-in {
    from { opacity: 0; transform: scale(0.95) translateY(-4px); }
    to   { opacity: 1; transform: scale(1) translateY(0); }
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px 8px;
    border-bottom: 1px solid rgba(255, 255, 255, 0.07);
    margin-bottom: 10px;
  }

  .panel-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--text);
    letter-spacing: 0.02em;
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--text-dim);
    font-size: 12px;
    cursor: pointer;
    padding: 2px 4px;
    border-radius: 4px;
    line-height: 1;
    transition: background 0.1s, color 0.1s;
  }

  .close-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text);
  }

  .section-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 10px;
    font-weight: 600;
    color: var(--text-dim);
    letter-spacing: 0.06em;
    text-transform: uppercase;
    padding: 0 12px 5px;
  }

  .value-badge {
    font-size: 10px;
    font-weight: 500;
    color: var(--accent, #5b8def);
    text-transform: none;
    letter-spacing: 0;
  }

  .swatches {
    display: flex;
    gap: 6px;
    padding: 0 12px 12px;
    flex-wrap: wrap;
  }

  .swatch {
    width: 28px;
    height: 28px;
    border-radius: 7px;
    border: 2px solid transparent;
    cursor: pointer;
    background: rgba(255, 255, 255, 0.08);
    display: grid;
    place-items: center;
    transition: border-color 0.12s, transform 0.1s;
    flex-shrink: 0;
  }

  .swatch:hover {
    transform: scale(1.1);
    border-color: rgba(255, 255, 255, 0.3);
  }

  .swatch.active {
    border-color: var(--accent, #5b8def);
    box-shadow: 0 0 0 1px var(--accent, #5b8def);
  }

  .glass-icon {
    font-size: 14px;
    color: rgba(255, 255, 255, 0.5);
    line-height: 1;
  }

  .slider {
    display: block;
    width: calc(100% - 24px);
    margin: 0 12px 12px;
    -webkit-appearance: none;
    appearance: none;
    height: 4px;
    background: rgba(255, 255, 255, 0.12);
    border-radius: 2px;
    outline: none;
    cursor: pointer;
  }

  .slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--accent, #5b8def);
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.4);
    cursor: pointer;
    transition: transform 0.1s;
  }

  .slider::-webkit-slider-thumb:hover {
    transform: scale(1.15);
  }

  .footer {
    display: flex;
    justify-content: flex-end;
    padding: 4px 12px 0;
  }

  .reset-btn {
    background: none;
    border: 1px solid rgba(255, 255, 255, 0.15);
    color: var(--text-dim);
    font-size: 11px;
    padding: 4px 10px;
    border-radius: 6px;
    cursor: pointer;
    transition: background 0.1s, color 0.1s, border-color 0.1s;
  }

  .reset-btn:hover {
    background: rgba(255, 255, 255, 0.08);
    color: var(--text);
    border-color: rgba(255, 255, 255, 0.25);
  }
</style>
