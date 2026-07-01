<script lang="ts">
  import type { FenceData } from "../lib/types";
  import { t, setLocale } from "../lib/i18n.svelte";
  import { settings } from "../lib/settings.svelte";

  interface Props {
    visible: boolean;
    onclose: () => void;
    oncreatefences: (fences: FenceData[]) => void;
  }

  let { visible = $bindable(false), onclose, oncreatefences }: Props = $props();

  let step = $state(0);
  let leaving = $state(false);

  function generateId(): string {
    return "fence_" + Date.now().toString(36) + Math.random().toString(36).slice(2, 6);
  }

  function makeFence(title: string, emoji: string, x: number, y: number): FenceData {
    return {
      id: generateId(),
      title,
      emoji,
      x,
      y,
      width: 280,
      height: 260,
      collapsed: false,
      icon_paths: [],
    };
  }

  function applyTemplate(kind: "work" | "play" | "all") {
    const gap = 300;
    const startX = 60;
    const startY = 60;
    let fences: FenceData[];

    switch (kind) {
      case "work":
        fences = [
          makeFence("💼 Work", "💼", startX, startY),
          makeFence("🔧 Tools", "🔧", startX + gap, startY),
          makeFence("📁 Files", "📁", startX + gap * 2, startY),
        ];
        break;
      case "play":
        fences = [
          makeFence("🎮 Games", "🎮", startX, startY),
          makeFence("🎵 Media", "🎵", startX + gap, startY),
          makeFence("🌐 Social", "🌐", startX + gap * 2, startY),
        ];
        break;
      case "all":
        fences = [
          makeFence("💼 Work", "💼", startX, startY),
          makeFence("🎮 Play", "🎮", startX + gap, startY),
          makeFence("🔧 Tools", "🔧", startX, startY + 280),
          makeFence("📁 Files", "📁", startX + gap, startY + 280),
        ];
        break;
    }

    oncreatefences(fences);
    step = 2;
  }

  function skip() {
    step = 2;
  }

  function finish() {
    leaving = true;
    setTimeout(() => {
      leaving = false;
      step = 0;
      onclose();
    }, 300);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (visible && e.key === "Escape") {
      e.preventDefault();
      finish();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if visible}
  <div class="onboarding-backdrop" class:leaving>
    <div class="onboarding-card" class:leaving>
      <!-- Step indicators -->
      <div class="steps-indicator">
        {#each [0, 1, 2] as i}
          <div class="step-dot" class:active={step === i} class:done={step > i}></div>
        {/each}
      </div>

      {#if step === 0}
        <!-- Welcome -->
        <div class="step-content">
          <div class="welcome-icon">❄</div>
          <h1 class="title">{t.welcomeTitle}</h1>
          <p class="desc">{t.welcomeDesc}</p>

          <div class="lang-row">
            <button
              class="lang-btn"
              class:active={settings.locale === "en"}
              onclick={() => setLocale("en")}
            >English</button>
            <button
              class="lang-btn"
              class:active={settings.locale === "zh"}
              onclick={() => setLocale("zh")}
            >中文</button>
          </div>

          <button class="primary-btn" onclick={() => (step = 1)}>
            {t.getStarted}
          </button>
        </div>

      {:else if step === 1}
        <!-- Choose template -->
        <div class="step-content">
          <h1 class="title">{t.chooseLayout}</h1>

          <div class="templates">
            <button class="template-card" onclick={() => applyTemplate("work")}>
              <span class="template-emoji">🏢</span>
              <span class="template-label">{t.templateWork}</span>
              <span class="template-detail">💼 🔧 📁</span>
            </button>

            <button class="template-card" onclick={() => applyTemplate("play")}>
              <span class="template-emoji">🎮</span>
              <span class="template-label">{t.templatePlay}</span>
              <span class="template-detail">🎮 🎵 🌐</span>
            </button>

            <button class="template-card" onclick={() => applyTemplate("all")}>
              <span class="template-emoji">⚡</span>
              <span class="template-label">{t.templateAllInOne}</span>
              <span class="template-detail">💼 🎮 🔧 📁</span>
            </button>
          </div>

          <button class="skip-link" onclick={skip}>
            {t.skipSetup}
          </button>
        </div>

      {:else}
        <!-- Tips -->
        <div class="step-content">
          <h1 class="title">✨</h1>

          <div class="tips">
            <div class="tip-card">
              <span class="tip-icon">🖱️</span>
              <span class="tip-text">{t.tipRightClick}</span>
            </div>
            <div class="tip-card">
              <span class="tip-icon">👆</span>
              <span class="tip-text">{t.tipDragIcons}</span>
            </div>
            <div class="tip-card">
              <span class="tip-icon">👁️</span>
              <span class="tip-text">{t.tipDoubleClick}</span>
            </div>
          </div>

          <button class="primary-btn" onclick={finish}>
            {t.done}
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .onboarding-backdrop {
    position: fixed;
    inset: 0;
    z-index: 20000;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.55);
    animation: ob-fade-in 0.4s ease-out;
  }

  .onboarding-backdrop.leaving {
    animation: ob-fade-out 0.3s ease-in forwards;
  }

  @keyframes ob-fade-in {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  @keyframes ob-fade-out {
    from { opacity: 1; }
    to { opacity: 0; }
  }

  .onboarding-card {
    width: 460px;
    background: rgba(18, 22, 36, 0.92);
    backdrop-filter: blur(40px) saturate(1.6);
    border: 1px solid var(--glass-border);
    border-radius: 20px;
    box-shadow:
      0 32px 80px rgba(0, 0, 0, 0.6),
      0 0 0 1px rgba(255, 255, 255, 0.04),
      inset 0 1px 0 rgba(255, 255, 255, 0.08);
    overflow: hidden;
    animation: ob-card-in 0.4s cubic-bezier(0.16, 1, 0.3, 1);
  }

  .onboarding-card.leaving {
    animation: ob-card-out 0.3s ease-in forwards;
  }

  @keyframes ob-card-in {
    from { opacity: 0; transform: scale(0.9) translateY(24px); }
    to { opacity: 1; transform: scale(1) translateY(0); }
  }

  @keyframes ob-card-out {
    from { opacity: 1; transform: scale(1) translateY(0); }
    to { opacity: 0; transform: scale(0.95) translateY(12px); }
  }

  .steps-indicator {
    display: flex;
    justify-content: center;
    gap: 8px;
    padding: 20px 0 0;
  }

  .step-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.15);
    transition: all 0.3s ease;
  }

  .step-dot.active {
    background: var(--accent);
    box-shadow: 0 0 8px var(--accent-glow);
    transform: scale(1.2);
  }

  .step-dot.done {
    background: rgba(91, 141, 239, 0.5);
  }

  .step-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 28px 36px 36px;
    animation: ob-step-in 0.35s cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes ob-step-in {
    from { opacity: 0; transform: translateX(20px); }
    to { opacity: 1; transform: translateX(0); }
  }

  .welcome-icon {
    font-size: 56px;
    margin-bottom: 12px;
    animation: ob-float 3s ease-in-out infinite;
  }

  @keyframes ob-float {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-6px); }
  }

  .title {
    font-size: 22px;
    font-weight: 700;
    color: var(--text);
    margin: 0 0 8px;
    text-align: center;
  }

  .desc {
    font-size: 14px;
    color: var(--text-dim);
    margin: 0 0 24px;
    text-align: center;
    line-height: 1.5;
  }

  .lang-row {
    display: flex;
    gap: 8px;
    margin-bottom: 28px;
  }

  .lang-btn {
    padding: 8px 20px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.04);
    color: var(--text-dim);
    font-size: 14px;
    font-family: inherit;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .lang-btn:hover {
    background: rgba(255, 255, 255, 0.08);
    border-color: rgba(255, 255, 255, 0.2);
    color: var(--text);
  }

  .lang-btn.active {
    background: rgba(91, 141, 239, 0.15);
    border-color: var(--accent);
    color: var(--accent);
  }

  .primary-btn {
    padding: 12px 40px;
    border: none;
    border-radius: 12px;
    background: var(--accent);
    color: white;
    font-size: 15px;
    font-weight: 600;
    font-family: inherit;
    cursor: pointer;
    transition: all 0.2s ease;
    box-shadow: 0 4px 16px rgba(91, 141, 239, 0.3);
  }

  .primary-btn:hover {
    background: #6d9af0;
    box-shadow: 0 6px 24px rgba(91, 141, 239, 0.45);
    transform: translateY(-1px);
  }

  .primary-btn:active {
    transform: translateY(0);
  }

  .templates {
    display: flex;
    gap: 12px;
    margin-bottom: 20px;
    width: 100%;
  }

  .template-card {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 20px 12px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 14px;
    background: rgba(255, 255, 255, 0.03);
    color: var(--text);
    cursor: pointer;
    transition: all 0.2s ease;
    font-family: inherit;
  }

  .template-card:hover {
    background: rgba(91, 141, 239, 0.1);
    border-color: var(--accent);
    transform: translateY(-2px);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.3);
  }

  .template-emoji {
    font-size: 32px;
  }

  .template-label {
    font-size: 14px;
    font-weight: 600;
  }

  .template-detail {
    font-size: 12px;
    color: var(--text-dim);
    letter-spacing: 2px;
  }

  .skip-link {
    background: none;
    border: none;
    color: var(--text-dim);
    font-size: 13px;
    font-family: inherit;
    cursor: pointer;
    padding: 8px 16px;
    border-radius: 8px;
    transition: color 0.2s;
  }

  .skip-link:hover {
    color: var(--text);
  }

  .tips {
    display: flex;
    flex-direction: column;
    gap: 10px;
    width: 100%;
    margin: 8px 0 24px;
  }

  .tip-card {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 14px 18px;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.06);
  }

  .tip-icon {
    font-size: 22px;
    flex-shrink: 0;
  }

  .tip-text {
    font-size: 14px;
    color: var(--text);
    line-height: 1.4;
  }
</style>
