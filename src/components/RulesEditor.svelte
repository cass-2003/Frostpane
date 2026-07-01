<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import type { FenceData } from "../lib/types";
  import { t } from "../lib/i18n.svelte";

  interface SortRule {
    id: string;
    name: string;
    condition: { Extension: string } | { NameContains: string } | { NamePrefix: string };
    target_fence_id: string;
    enabled: boolean;
  }

  interface Props {
    visible: boolean;
    fences: FenceData[];
    onclose: () => void;
  }

  let { visible = $bindable(false), fences, onclose }: Props = $props();

  let rules = $state<SortRule[]>([]);
  let adding = $state(false);
  let newName = $state("");
  let newCondType = $state<"Extension" | "NameContains" | "NamePrefix">("Extension");
  let newCondValue = $state("");
  let newTarget = $state("");

  onMount(async () => {
    try {
      rules = await invoke<SortRule[]>("load_rules");
    } catch (e) {
      console.error("Failed to load rules:", e);
    }
  });

  function condLabel(type: string): string {
    switch (type) {
      case "Extension": return t.ruleExtension;
      case "NameContains": return t.ruleNameContains;
      case "NamePrefix": return t.ruleNamePrefix;
      default: return type;
    }
  }

  function condType(rule: SortRule): string {
    if ("Extension" in rule.condition) return "Extension";
    if ("NameContains" in rule.condition) return "NameContains";
    return "NamePrefix";
  }

  function condValue(rule: SortRule): string {
    if ("Extension" in rule.condition) return rule.condition.Extension;
    if ("NameContains" in rule.condition) return rule.condition.NameContains;
    return rule.condition.NamePrefix;
  }

  function fenceName(id: string): string {
    return fences.find((f) => f.id === id)?.title ?? id;
  }

  function buildCondition(type: string, value: string): SortRule["condition"] {
    switch (type) {
      case "NameContains": return { NameContains: value };
      case "NamePrefix": return { NamePrefix: value };
      default: return { Extension: value };
    }
  }

  async function save() {
    try {
      await invoke("save_rules", { rules });
    } catch (e) {
      console.error("Failed to save rules:", e);
    }
  }

  async function toggleRule(id: string) {
    rules = rules.map((r) => r.id === id ? { ...r, enabled: !r.enabled } : r);
    await save();
  }

  async function deleteRule(id: string) {
    rules = rules.filter((r) => r.id !== id);
    await save();
  }

  async function addRule() {
    if (!newName.trim() || !newCondValue.trim() || !newTarget) return;
    const rule: SortRule = {
      id: "rule_" + Date.now().toString(36) + Math.random().toString(36).slice(2, 6),
      name: newName.trim(),
      condition: buildCondition(newCondType, newCondValue.trim()),
      target_fence_id: newTarget,
      enabled: true,
    };
    rules = [...rules, rule];
    await save();
    newName = "";
    newCondValue = "";
    newCondType = "Extension";
    newTarget = "";
    adding = false;
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
    <div class="settings-panel" role="dialog" aria-label="Rules">
      <div class="settings-header">
        <h2>{t.rules}</h2>
        <button class="close-btn" onclick={onclose} title="Close">✕</button>
      </div>

      <div class="settings-body">
        {#if rules.length === 0 && !adding}
          <div class="empty">{t.noRules}</div>
        {/if}

        {#each rules as rule (rule.id)}
          <div class="rule-row">
            <button
              class="toggle-switch"
              class:active={rule.enabled}
              onclick={() => toggleRule(rule.id)}
              role="switch"
              aria-checked={rule.enabled}
            >
              <span class="toggle-knob"></span>
            </button>
            <div class="rule-info">
              <span class="rule-name">{rule.name}</span>
              <span class="rule-detail">
                {condLabel(condType(rule))}: {condValue(rule)} → {fenceName(rule.target_fence_id)}
              </span>
            </div>
            <button class="delete-btn" onclick={() => deleteRule(rule.id)} title={t.deleteRule}>✕</button>
          </div>
        {/each}

        {#if adding}
          <div class="add-form">
            <div class="form-row">
              <label class="form-label">{t.ruleName}</label>
              <input class="form-input" type="text" bind:value={newName} placeholder={t.ruleName} />
            </div>
            <div class="form-row">
              <label class="form-label">{t.ruleCondition}</label>
              <select class="form-select" bind:value={newCondType}>
                <option value="Extension">{t.ruleExtension}</option>
                <option value="NameContains">{t.ruleNameContains}</option>
                <option value="NamePrefix">{t.ruleNamePrefix}</option>
              </select>
            </div>
            <div class="form-row">
              <label class="form-label">&nbsp;</label>
              <input class="form-input" type="text" bind:value={newCondValue}
                placeholder={newCondType === "Extension" ? "pdf" : "..."}
              />
            </div>
            <div class="form-row">
              <label class="form-label">{t.ruleTarget}</label>
              <select class="form-select" bind:value={newTarget}>
                <option value="" disabled>--</option>
                {#each fences as fence (fence.id)}
                  <option value={fence.id}>{fence.emoji} {fence.title}</option>
                {/each}
              </select>
            </div>
            <div class="form-actions">
              <button class="action-btn" onclick={addRule}>{t.addRule}</button>
              <button class="action-btn cancel" onclick={() => { adding = false; }}>{t.cancel}</button>
            </div>
          </div>
        {:else}
          <div class="add-bar">
            <button class="action-btn" onclick={() => { adding = true; }}>+ {t.addRule}</button>
          </div>
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
    width: 440px;
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
    padding: 8px 20px 16px;
    overflow-y: auto;
    max-height: calc(80vh - 60px);
  }

  .empty {
    padding: 24px 0;
    text-align: center;
    color: var(--text-dim);
    font-size: 13px;
  }

  .rule-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.04);
  }

  .rule-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .rule-name {
    font-size: 13px;
    color: var(--text);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .rule-detail {
    font-size: 11px;
    color: var(--text-dim);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .delete-btn {
    background: none;
    border: none;
    color: var(--text-dim);
    font-size: 13px;
    cursor: pointer;
    padding: 4px 6px;
    border-radius: 4px;
    transition: background 0.15s, color 0.15s;
  }

  .delete-btn:hover {
    background: rgba(224, 64, 96, 0.2);
    color: #ff8090;
  }

  .toggle-switch {
    position: relative;
    width: 36px;
    min-width: 36px;
    height: 20px;
    border-radius: 10px;
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
    top: 2px;
    left: 2px;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: white;
    transition: transform 0.2s;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
  }

  .toggle-switch.active .toggle-knob {
    transform: translateX(16px);
  }

  .add-bar {
    padding: 12px 0 4px;
  }

  .add-form {
    padding: 12px 0;
    border-top: 1px solid rgba(255, 255, 255, 0.06);
  }

  .form-row {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 8px;
  }

  .form-label {
    width: 72px;
    min-width: 72px;
    font-size: 12px;
    color: var(--text-dim);
    text-align: right;
  }

  .form-input,
  .form-select {
    flex: 1;
    padding: 6px 10px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.06);
    color: var(--text);
    font-size: 13px;
    outline: none;
    transition: border-color 0.15s;
  }

  .form-input:focus,
  .form-select:focus {
    border-color: var(--accent);
  }

  .form-select option {
    background: #1a1e2e;
    color: var(--text);
  }

  .form-actions {
    display: flex;
    gap: 8px;
    padding-top: 4px;
    justify-content: flex-end;
  }

  .action-btn {
    padding: 6px 16px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.06);
    color: var(--text);
    font-size: 13px;
    cursor: pointer;
    transition: background 0.15s, border-color 0.15s;
  }

  .action-btn:hover {
    background: rgba(255, 255, 255, 0.12);
    border-color: rgba(255, 255, 255, 0.2);
  }

  .action-btn.cancel {
    color: var(--text-dim);
  }
</style>
