<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount, onDestroy } from "svelte";
  import { initGlobalShortcuts, unregisterAll } from "../lib/shortcuts";
  import type { DesktopIcon, FenceData, FenceStyle, PageData } from "../lib/types";
  import Fence from "./Fence.svelte";
  import ContextMenu from "./ContextMenu.svelte";
  import DesktopMenu from "./DesktopMenu.svelte";
  import FenceMenu from "./FenceMenu.svelte";
  import Settings from "./Settings.svelte";
  import SceneManager from "./SceneManager.svelte";
  import SearchBar from "./SearchBar.svelte";
  import ArchiveDialog from "./ArchiveDialog.svelte";
  import Onboarding from "./Onboarding.svelte";
  import RulesEditor from "./RulesEditor.svelte";
  import FenceStyleEditor from "./FenceStyleEditor.svelte";
  import { settings, loadSettings } from "../lib/settings.svelte";
  import { t, initLocale } from "../lib/i18n.svelte";

  interface ArchivePreview {
    moves: Array<{ original_path: string; archived_path: string; file_name: string }>;
    target_dir: string;
    file_count: number;
  }

  let gridW = $derived(settings.iconSize + 32);
  let gridH = $derived(settings.iconSize + 52);
  const DRAG_THRESHOLD = 5;
  const DEFAULT_FENCE_W = 320;
  const DEFAULT_FENCE_H = 300;

  let icons = $state<DesktopIcon[]>([]);
  let fences = $state<FenceData[]>([]);
  let loading = $state(true);
  let error = $state("");
  let fencesHidden = $state(false);

  // Selection state
  let selectedPaths = $state<Set<string>>(new Set());
  let boxSelecting = $state(false);
  let boxStartX = $state(0);
  let boxStartY = $state(0);
  let boxCurrentX = $state(0);
  let boxCurrentY = $state(0);

  // Icon drag state
  let draggingIcon = $state<string | null>(null);
  let dragOffsetX = 0;
  let dragOffsetY = 0;
  let dragStartX = 0;
  let dragStartY = 0;
  let didDrag = false;
  let dragOverFence = $state<string | null>(null);
  let batchDragOffsets = $state<Map<string, { dx: number; dy: number }>>(new Map());

  let draggedIconData = $derived(
    draggingIcon ? (icons.find((ic) => ic.path === draggingIcon) ?? null) : null
  );

  // Free icons: no fence_id
  let freeIcons = $derived(icons.filter((ic) => !ic.fence_id));

  // Context menu state (icon right-click)
  let ctxVisible = $state(false);
  let ctxX = $state(0);
  let ctxY = $state(0);
  let ctxPath = $state("");
  let ctxName = $state("");

  // Desktop menu state (empty space right-click)
  let deskMenuVisible = $state(false);
  let deskMenuX = $state(0);
  let deskMenuY = $state(0);

  // Settings panel
  let settingsVisible = $state(false);

  // Rules editor
  let rulesEditorVisible = $state(false);

  // Fence style editor
  let styleEditorVisible = $state(false);
  let styleEditorFenceId = $state("");
  let styleEditorX = $state(0);
  let styleEditorY = $state(0);

  // Scene manager
  let sceneManagerVisible = $state(false);

  // Search bar
  let searchVisible = $state(false);

  // Onboarding
  let onboardingVisible = $state(false);

  // Archive dialog state
  let archiveDialogVisible = $state(false);
  let archivePreview = $state<ArchivePreview | null>(null);
  let archiveTargetFenceId = $state("");

  // Fence context menu state
  let fenceMenuVisible = $state(false);
  let fenceMenuX = $state(0);
  let fenceMenuY = $state(0);
  let fenceMenuTargetId = $state("");

  // Per-fence rename trigger tokens
  let fenceRenameTokens = $state<Record<string, number>>({});

  // Pages state
  let currentPage = $state(0);
  let totalPages = $state(1);
  let pages = $state<PageData[]>([]);
  let slideDir = $state<"left" | "right" | null>(null);
  let edgeHoldTimer: ReturnType<typeof setTimeout> | null = null;
  let edgeIndicatorVisible = $state(false);
  let edgeIndicatorSide = $state<"left" | "right">("right");

  onDestroy(() => {
    unregisterAll().catch(() => {});
  });

  onMount(async () => {
    initLocale();
    await loadSettings();

    initGlobalShortcuts({
      onToggleSearch: () => { searchVisible = !searchVisible; },
      onToggleFences: () => { fencesHidden = !fencesHidden; },
    }).catch((e) => console.error("Global shortcuts failed to register:", e));

    try {
      const [loadedIcons, loadedPagesRaw] = await Promise.all([
        invoke<DesktopIcon[]>("get_desktop_icons"),
        invoke<PageData[]>("load_pages").catch(() => []),
      ]);
      icons = loadedIcons;

      if (loadedPagesRaw.length > 0) {
        pages = loadedPagesRaw;
        totalPages = loadedPagesRaw.length;
        loadPageData(loadedPagesRaw[0]);
      } else {
        // Migration: load from legacy fences.json
        const loadedFences = await invoke<FenceData[]>("load_fence_layout").catch(() => []);
        if (loadedFences.length > 0) {
          const fenceMap = new Map<string, string>();
          for (const f of loadedFences) {
            for (const p of f.icon_paths) {
              fenceMap.set(p, f.id);
            }
          }
          icons = icons.map((ic) => ({
            ...ic,
            fence_id: fenceMap.get(ic.path) ?? null,
          }));
          fences = loadedFences;
          pages = [buildCurrentPageData()];
        } else {
          pages = [{ fences: [] }];
          onboardingVisible = true;
        }
        totalPages = 1;
      }

      if (fences.length === 0 && !onboardingVisible) {
        onboardingVisible = true;
      }

      // Auto-sort unassigned icons via rules
      try {
        const rules = await invoke<any[]>("load_rules");
        if (rules.length > 0) {
          let changed = false;
          for (let i = 0; i < icons.length; i++) {
            if (icons[i].fence_id) continue;
            const filename = icons[i].name;
            const target = await invoke<string | null>("match_rules", { filename, rules });
            if (target && fences.some((f) => f.id === target)) {
              icons[i] = { ...icons[i], fence_id: target };
              changed = true;
            }
          }
          if (changed) {
            icons = [...icons];
            await saveAll();
          }
        }
      } catch (e) {
        console.error("Rules auto-sort failed:", e);
      }
    } catch (e) {
      error = String(e);
      console.error("Failed to load desktop icons:", e);
    } finally {
      loading = false;
    }
  });

  function snapToGrid(val: number, cell: number): number {
    return Math.round(val / cell) * cell;
  }

  function generateId(): string {
    return "fence_" + Date.now().toString(36) + Math.random().toString(36).slice(2, 6);
  }

  // ── Pages ──

  function buildCurrentPageData(): PageData {
    return {
      fences: buildFencesForSave(),
      iconPositions: icons.map((ic) => ({ path: ic.path, x: ic.x, y: ic.y })),
    };
  }

  function loadPageData(page: PageData) {
    if (page.fences.length > 0) {
      fences = page.fences;
      const fenceMap = new Map<string, string>();
      for (const f of page.fences) {
        for (const p of f.icon_paths) {
          fenceMap.set(p, f.id);
        }
      }
      icons = icons.map((ic) => ({
        ...ic,
        fence_id: fenceMap.get(ic.path) ?? null,
        ...(page.iconPositions?.find((p) => p.path === ic.path) ?? {}),
      }));
    } else {
      fences = [];
      icons = icons.map((ic) => ({ ...ic, fence_id: null }));
    }
  }

  async function switchToPage(targetIndex: number) {
    if (targetIndex < 0 || targetIndex >= totalPages || targetIndex === currentPage) return;
    pages[currentPage] = buildCurrentPageData();
    pages = [...pages];
    currentPage = targetIndex;
    loadPageData(pages[targetIndex]);
    slideDir = targetIndex > currentPage ? "left" : "right";
    setTimeout(() => { slideDir = null; }, 300);
    await saveAllPages();
  }

  async function addPage() {
    pages[currentPage] = buildCurrentPageData();
    const newPage: PageData = { fences: [] };
    pages = [...pages, newPage];
    totalPages = pages.length;
    currentPage = totalPages - 1;
    loadPageData(newPage);
    await saveAllPages();
  }

  async function deletePage() {
    if (totalPages <= 1) return;
    pages.splice(currentPage, 1);
    pages = [...pages];
    totalPages = pages.length;
    if (currentPage >= totalPages) currentPage = totalPages - 1;
    loadPageData(pages[currentPage]);
    await saveAllPages();
  }

  async function saveAllPages() {
    pages[currentPage] = buildCurrentPageData();
    try {
      await invoke("save_pages", { pages });
    } catch (err) {
      console.error("Failed to save pages:", err);
    }
  }

  function handleEdgeDetection(e: PointerEvent) {
    const threshold = 2;
    const vw = window.innerWidth;
    if (e.clientX >= vw - threshold && currentPage < totalPages - 1) {
      if (!edgeHoldTimer) {
        edgeIndicatorSide = "right";
        edgeIndicatorVisible = true;
        edgeHoldTimer = setTimeout(() => {
          edgeIndicatorVisible = false;
          edgeHoldTimer = null;
          switchToPage(currentPage + 1);
        }, 500);
      }
    } else if (e.clientX <= threshold && currentPage > 0) {
      if (!edgeHoldTimer) {
        edgeIndicatorSide = "left";
        edgeIndicatorVisible = true;
        edgeHoldTimer = setTimeout(() => {
          edgeIndicatorVisible = false;
          edgeHoldTimer = null;
          switchToPage(currentPage - 1);
        }, 500);
      }
    } else if (edgeHoldTimer) {
      clearTimeout(edgeHoldTimer);
      edgeHoldTimer = null;
      edgeIndicatorVisible = false;
    }
  }

  // ── Icon drag ──

  function onIconPointerDown(e: PointerEvent, icon: DesktopIcon) {
    if (e.button !== 0) return;
    e.preventDefault();
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);

    if (e.ctrlKey) {
      const next = new Set(selectedPaths);
      if (next.has(icon.path)) next.delete(icon.path);
      else next.add(icon.path);
      selectedPaths = next;
    } else if (!selectedPaths.has(icon.path)) {
      selectedPaths = new Set([icon.path]);
    }

    dragOffsetX = e.clientX - icon.x;
    dragOffsetY = e.clientY - icon.y;
    dragStartX = e.clientX;
    dragStartY = e.clientY;
    didDrag = false;
    draggingIcon = icon.path;

    const offsets = new Map<string, { dx: number; dy: number }>();
    if (selectedPaths.has(icon.path)) {
      for (const p of selectedPaths) {
        if (p === icon.path) continue;
        const other = icons.find((ic) => ic.path === p && !ic.fence_id);
        if (other) offsets.set(p, { dx: other.x - icon.x, dy: other.y - icon.y });
      }
    }
    batchDragOffsets = offsets;
  }

  function onPointerMove(e: PointerEvent) {
    handleEdgeDetection(e);
    if (boxSelecting) {
      boxCurrentX = e.clientX;
      boxCurrentY = e.clientY;
      return;
    }
    if (!draggingIcon) return;
    const dx = e.clientX - dragStartX;
    const dy = e.clientY - dragStartY;
    if (!didDrag && Math.sqrt(dx * dx + dy * dy) > DRAG_THRESHOLD) {
      didDrag = true;
    }
    if (didDrag) {
      const newX = e.clientX - dragOffsetX;
      const newY = e.clientY - dragOffsetY;
      icons = icons.map((ic) => {
        if (ic.path === draggingIcon) return { ...ic, x: newX, y: newY };
        const off = batchDragOffsets.get(ic.path);
        if (off) return { ...ic, x: newX + off.dx, y: newY + off.dy };
        return ic;
      });
      dragOverFence = hitTestFence(e.clientX, e.clientY);
    }
  }

  async function onPointerUp(e: PointerEvent) {
    if (boxSelecting) {
      boxSelecting = false;
      const x1 = Math.min(boxStartX, boxCurrentX);
      const y1 = Math.min(boxStartY, boxCurrentY);
      const x2 = Math.max(boxStartX, boxCurrentX);
      const y2 = Math.max(boxStartY, boxCurrentY);
      const hit = new Set<string>();
      for (const ic of freeIcons) {
        const cx = ic.x + 40;
        const cy = ic.y + 30;
        if (cx >= x1 && cx <= x2 && cy >= y1 && cy <= y2) hit.add(ic.path);
      }
      selectedPaths = hit;
      return;
    }

    if (!draggingIcon) return;
    const path = draggingIcon;
    const targetFenceId = dragOverFence;
    const offsets = batchDragOffsets;
    draggingIcon = null;
    dragOverFence = null;
    batchDragOffsets = new Map();

    if (!didDrag) return;

    const batchPaths = new Set([path, ...offsets.keys()]);

    if (targetFenceId) {
      icons = icons.map((ic) =>
        batchPaths.has(ic.path)
          ? { ...ic, fence_id: targetFenceId, x: 0, y: 0 }
          : ic
      );
      // If target fence has tabs, add dropped icons to the active tab
      const targetFence = fences.find(f => f.id === targetFenceId);
      if (targetFence?.tabs && targetFence.tabs.length > 0) {
        const activeTabId = targetFence.activeTab ?? targetFence.tabs[0]?.id;
        if (activeTabId) {
          fences = fences.map(f => {
            if (f.id !== targetFenceId) return f;
            return {
              ...f,
              tabs: f.tabs?.map(tab =>
                tab.id === activeTabId
                  ? { ...tab, icon_paths: [...new Set([...tab.icon_paths, ...batchPaths])] }
                  : tab
              )
            };
          });
        }
      }
    } else {
      const snappedX = snapToGrid(e.clientX - dragOffsetX, gridW);
      const snappedY = snapToGrid(e.clientY - dragOffsetY, gridH);
      icons = icons.map((ic) => {
        if (ic.path === path) {
          return { ...ic, x: snappedX, y: snappedY, fence_id: null };
        }
        const off = offsets.get(ic.path);
        if (off) {
          return {
            ...ic,
            x: snapToGrid(snappedX + off.dx, gridW),
            y: snapToGrid(snappedY + off.dy, gridH),
            fence_id: null,
          };
        }
        return ic;
      });
    }

    await saveAll();
  }

  function hitTestFence(mx: number, my: number): string | null {
    for (const f of fences) {
      if (f.collapsed) continue;
      if (
        mx >= f.x &&
        mx <= f.x + f.width &&
        my >= f.y &&
        my <= f.y + f.height
      ) {
        return f.id;
      }
    }
    return null;
  }

  // ── Snap/magnetic alignment ──

  const SNAP_DISTANCE = 12;

  function snapFencePosition(id: string, rawX: number, rawY: number): { x: number; y: number } {
    let x = rawX;
    let y = rawY;
    const moving = fences.find((f) => f.id === id);
    if (!moving) return { x, y };

    const vw = window.innerWidth;
    const vh = window.innerHeight;
    const right = x + moving.width;
    const bottom = y + moving.height;

    // Snap to screen edges
    if (Math.abs(x) < SNAP_DISTANCE) x = 0;
    if (Math.abs(y) < SNAP_DISTANCE) y = 0;
    if (Math.abs(right - vw) < SNAP_DISTANCE) x = vw - moving.width;
    if (Math.abs(bottom - vh) < SNAP_DISTANCE) y = vh - moving.height;

    // Snap to other fences
    for (const other of fences) {
      if (other.id === id) continue;
      const oRight = other.x + other.width;
      const oBottom = other.y + other.height;

      // Horizontal snaps
      if (Math.abs(x - oRight) < SNAP_DISTANCE) x = oRight;
      if (Math.abs(right - other.x) < SNAP_DISTANCE) x = other.x - moving.width;
      if (Math.abs(x - other.x) < SNAP_DISTANCE) x = other.x;
      if (Math.abs(right - oRight) < SNAP_DISTANCE) x = oRight - moving.width;

      // Vertical snaps
      if (Math.abs(y - oBottom) < SNAP_DISTANCE) y = oBottom;
      if (Math.abs(bottom - other.y) < SNAP_DISTANCE) y = other.y - moving.height;
      if (Math.abs(y - other.y) < SNAP_DISTANCE) y = other.y;
      if (Math.abs(bottom - oBottom) < SNAP_DISTANCE) y = oBottom - moving.height;
    }

    return { x, y };
  }

  // ── Fence callbacks ──

  function handleFenceMove(id: string, rawX: number, rawY: number) {
    const { x, y } = snapFencePosition(id, rawX, rawY);
    fences = fences.map((f) => (f.id === id ? { ...f, x, y } : f));
  }

  function handleFenceResize(id: string, w: number, h: number) {
    fences = fences.map((f) =>
      f.id === id ? { ...f, width: w, height: h } : f
    );
  }

  function handleFenceRename(id: string, title: string) {
    fences = fences.map((f) => (f.id === id ? { ...f, title } : f));
    saveFences();
  }

  function handleFenceEmoji(id: string, emoji: string) {
    fences = fences.map((f) => (f.id === id ? { ...f, emoji } : f));
    saveFences();
  }

  function handleFenceCollapse(id: string) {
    fences = fences.map((f) =>
      f.id === id ? { ...f, collapsed: !f.collapsed } : f
    );
    saveFences();
  }

  function handleFenceDelete(id: string) {
    // Remove fence, release icons back to desktop
    icons = icons.map((ic) =>
      ic.fence_id === id ? { ...ic, fence_id: null } : ic
    );
    fences = fences.filter((f) => f.id !== id);
    saveAll();
  }

  function handleFenceMenuShow(e: MouseEvent, id: string) {
    fenceMenuX = e.clientX;
    fenceMenuY = e.clientY;
    fenceMenuTargetId = id;
    fenceMenuVisible = true;
  }

  function handleFenceMenuAction(id: string, fenceId: string) {
    switch (id) {
      case "rename":
        fenceRenameTokens = {
          ...fenceRenameTokens,
          [fenceId]: (fenceRenameTokens[fenceId] ?? 0) + 1,
        };
        break;
      case "sort_name": {
        const others = icons.filter((ic) => ic.fence_id !== fenceId);
        const sorted = icons
          .filter((ic) => ic.fence_id === fenceId)
          .sort((a, b) => a.name.localeCompare(b.name));
        icons = [...others, ...sorted];
        saveAll();
        break;
      }
      case "sort_type": {
        const others = icons.filter((ic) => ic.fence_id !== fenceId);
        const sorted = icons
          .filter((ic) => ic.fence_id === fenceId)
          .sort((a, b) => {
            const extA = a.name.split(".").pop()?.toLowerCase() ?? "";
            const extB = b.name.split(".").pop()?.toLowerCase() ?? "";
            return extA.localeCompare(extB) || a.name.localeCompare(b.name);
          });
        icons = [...others, ...sorted];
        saveAll();
        break;
      }
      case "toggle_view":
        fences = fences.map(f =>
          f.id === fenceId
            ? { ...f, viewMode: (f.viewMode === "list" ? "grid" : "list") }
            : f
        );
        saveFences();
        break;
      case "add_tab": {
        const fence = fences.find(f => f.id === fenceId);
        if (!fence) break;
        if (!fence.tabs || fence.tabs.length === 0) {
          const firstTabId = generateId();
          const currentIconPaths = icons.filter(ic => ic.fence_id === fenceId).map(ic => ic.path);
          fences = fences.map(f =>
            f.id === fenceId
              ? { ...f, tabs: [{ id: firstTabId, name: t.newTab, icon_paths: currentIconPaths }], activeTab: firstTabId }
              : f
          );
        } else {
          const newTabId = generateId();
          fences = fences.map(f =>
            f.id === fenceId
              ? { ...f, tabs: [...(f.tabs ?? []), { id: newTabId, name: t.newTab, icon_paths: [] }], activeTab: newTabId }
              : f
          );
        }
        saveFences();
        break;
      }
      case "appearance":
        styleEditorFenceId = fenceId;
        styleEditorX = fenceMenuX;
        styleEditorY = fenceMenuY;
        styleEditorVisible = true;
        break;
      case "auto_color":
        handleAutoColor(fenceId);
        break;
      case "archive":
        handleArchivePreview(fenceId);
        break;
      case "delete":
        handleFenceDelete(fenceId);
        break;
    }
  }

  async function handleAutoColor(fenceId: string) {
    const { computeFenceDominantColor } = await import("../lib/color");
    const fenceIcons = icons.filter((ic) => ic.fence_id === fenceId);
    const iconDataList = fenceIcons.map((ic) => ic.icon_data);
    const bgColor = await computeFenceDominantColor(iconDataList);
    fences = fences.map((f) =>
      f.id === fenceId
        ? { ...f, style: { ...f.style, bgColor } }
        : f
    );
    saveFences();
  }

  async function handleArchivePreview(fenceId: string) {
    const fence = fences.find((f) => f.id === fenceId);
    if (!fence) return;
    const fenceIcons = icons.filter((ic) => ic.fence_id === fenceId);
    const paths = fenceIcons.map((ic) => ic.path);
    if (paths.length === 0) return;
    try {
      const preview = await invoke<ArchivePreview>("preview_archive", {
        fenceTitle: fence.title,
        iconPaths: paths,
      });
      if (preview.file_count === 0) return;
      archivePreview = preview;
      archiveTargetFenceId = fenceId;
      archiveDialogVisible = true;
    } catch (e) {
      console.error("Archive preview failed:", e);
    }
  }

  async function handleArchiveConfirm() {
    if (!archivePreview || !archiveTargetFenceId) return;
    const fence = fences.find((f) => f.id === archiveTargetFenceId);
    if (!fence) return;
    const fenceIcons = icons.filter((ic) => ic.fence_id === archiveTargetFenceId);
    const paths = fenceIcons.map((ic) => ic.path);
    try {
      await invoke("execute_archive", {
        fenceTitle: fence.title,
        iconPaths: paths,
      });
      const archivedPaths = new Set(archivePreview.moves.map((m) => m.original_path));
      icons = icons.filter((ic) => !archivedPaths.has(ic.path));
      await saveAll();
    } catch (e) {
      console.error("Archive failed:", e);
    } finally {
      archiveDialogVisible = false;
      archivePreview = null;
      archiveTargetFenceId = "";
    }
  }

  // ── Icon interactions within fences ──

  function handleFenceIconDragStart(e: PointerEvent, icon: DesktopIcon) {
    // Remove icon from any tab's icon_paths when dragging out
    fences = fences.map(f => ({
      ...f,
      tabs: f.tabs?.map(tab => ({
        ...tab,
        icon_paths: tab.icon_paths.filter(p => p !== icon.path)
      }))
    }));
    icons = icons.map((ic) =>
      ic.path === icon.path
        ? { ...ic, fence_id: null, x: e.clientX - 40, y: e.clientY - 24 }
        : ic
    );
    dragOffsetX = 40;
    dragOffsetY = 24;
    dragStartX = e.clientX;
    dragStartY = e.clientY;
    didDrag = true;
    draggingIcon = icon.path;
  }

  function handleFenceIconClick(_icon: DesktopIcon) {
    // Selection logic can be added later
  }

  async function handleFenceIconDblClick(icon: DesktopIcon) {
    try {
      await invoke("open_item", { path: icon.path });
    } catch (e) {
      console.error("Failed to open:", e);
    }
  }

  function handleFenceIconContext(e: MouseEvent, icon: DesktopIcon) {
    e.preventDefault();
    ctxX = e.clientX;
    ctxY = e.clientY;
    ctxPath = icon.path;
    ctxName = icon.name;
    ctxVisible = true;
  }

  // ── Free icon interactions ──

  async function handleDoubleClick(icon: DesktopIcon) {
    if (didDrag) return;
    try {
      await invoke("open_item", { path: icon.path });
    } catch (e) {
      console.error("Failed to open:", e);
    }
  }

  function handleIconContext(e: MouseEvent, icon: DesktopIcon) {
    e.preventDefault();
    ctxX = e.clientX;
    ctxY = e.clientY;
    ctxPath = icon.path;
    ctxName = icon.name;
    ctxVisible = true;
  }

  // ── Desktop empty-space interactions ──

  function onDesktopPointerDown(e: PointerEvent) {
    if (e.button !== 0) return;
    const target = e.target as HTMLElement;
    if (target.closest("[data-fence-id]") || target.closest(".desktop-icon")) return;
    if (!e.ctrlKey) selectedPaths = new Set();
    boxSelecting = true;
    boxStartX = e.clientX;
    boxStartY = e.clientY;
    boxCurrentX = e.clientX;
    boxCurrentY = e.clientY;
  }

  function handleDesktopContext(e: MouseEvent) {
    if ((e.target as HTMLElement).closest("[data-fence-id]")) return;
    if ((e.target as HTMLElement).closest(".desktop-icon")) return;
    e.preventDefault();
    deskMenuX = e.clientX;
    deskMenuY = e.clientY;
    deskMenuVisible = true;
  }

  function handleDesktopDblClick(e: MouseEvent) {
    if ((e.target as HTMLElement).closest("[data-fence-id]")) return;
    if ((e.target as HTMLElement).closest(".desktop-icon")) return;
    fencesHidden = !fencesHidden;
  }

  function handleDesktopMenuAction(id: string) {
    switch (id) {
      case "new_fence":
        createFence(deskMenuX, deskMenuY);
        break;
      case "new_portal":
        createPortalFence(deskMenuX, deskMenuY);
        break;
      case "show_all_fences":
        fences = fences.map((f) => ({ ...f, collapsed: false }));
        saveFences();
        break;
      case "hide_all_fences":
        fences = fences.map((f) => ({ ...f, collapsed: true }));
        saveFences();
        break;
      case "sort_by_name":
        icons = [...icons].sort((a, b) => a.name.localeCompare(b.name));
        saveAll();
        break;
      case "settings":
        settingsVisible = true;
        break;
      case "scenes":
        sceneManagerVisible = true;
        break;
      case "rules":
        rulesEditorVisible = true;
        break;
      case "next_page":
        switchToPage(currentPage + 1);
        break;
      case "prev_page":
        switchToPage(currentPage - 1);
        break;
      case "add_page":
        addPage();
        break;
      case "delete_page":
        deletePage();
        break;
    }
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  function handleSceneLoad(scene: any) {
    const positions = scene.icon_positions as Array<{ path: string; x: number; y: number }>;
    const posMap = new Map(positions.map((p) => [p.path, p]));
    icons = icons.map((ic) => {
      const pos = posMap.get(ic.path) as { path: string; x: number; y: number } | undefined;
      return pos ? { ...ic, x: pos.x, y: pos.y, fence_id: null } : ic;
    });

    if (scene.fences.length > 0) {
      fences = scene.fences;
      const fenceMap = new Map<string, string>();
      for (const f of fences) {
        for (const p of (f as FenceData).icon_paths) {
          fenceMap.set(p, f.id);
        }
      }
      icons = icons.map((ic) => ({
        ...ic,
        fence_id: fenceMap.get(ic.path) ?? ic.fence_id ?? null,
      }));
    }

    saveAll();
  }

  function createFence(x: number, y: number) {
    const newFence: FenceData = {
      id: generateId(),
      title: t.newFenceTitle,
      emoji: "📁",
      x,
      y,
      width: DEFAULT_FENCE_W,
      height: DEFAULT_FENCE_H,
      collapsed: false,
      icon_paths: [],
    };
    fences = [...fences, newFence];
    saveFences();
  }

  function createPortalFence(x: number, y: number) {
    const folderPath = prompt(t.enterFolderPath);
    if (!folderPath) return;
    const name = folderPath.split(/[\\/]/).pop() || "Portal";
    const newFence: FenceData = {
      id: generateId(),
      title: name,
      emoji: "🔗",
      x,
      y,
      width: DEFAULT_FENCE_W,
      height: DEFAULT_FENCE_H,
      collapsed: false,
      icon_paths: [],
      portalPath: folderPath,
    };
    fences = [...fences, newFence];
    saveFences();
  }

  function handleOnboardingFences(newFences: FenceData[]) {
    fences = [...fences, ...newFences];
    saveFences();
  }

  function handleIconDeleted(path: string) {
    icons = icons.filter((ic) => ic.path !== path);
    saveAll();
  }

  // ── Persistence ──

  function buildFencesForSave(): FenceData[] {
    return fences.map((f) => ({
      ...f,
      icon_paths: icons
        .filter((ic) => ic.fence_id === f.id)
        .map((ic) => ic.path),
    }));
  }

  async function saveFences() {
    try {
      await invoke("save_fence_layout", { fences: buildFencesForSave() });
    } catch (err) {
      console.error("Failed to save fences:", err);
    }
  }

  async function savePositions() {
    const positions = icons.map((ic) => ({
      path: ic.path,
      x: ic.x,
      y: ic.y,
    }));
    try {
      await invoke("save_icon_positions", { positions });
    } catch (err) {
      console.error("Failed to save positions:", err);
    }
  }

  async function saveAll() {
    await Promise.all([savePositions(), saveFences(), saveAllPages()]);
  }

  function iconsForFence(fenceId: string): DesktopIcon[] {
    const fence = fences.find(f => f.id === fenceId);
    if (!fence?.tabs || fence.tabs.length === 0) {
      return icons.filter((ic) => ic.fence_id === fenceId);
    }
    const activeTab = fence.tabs.find(t => t.id === fence.activeTab) ?? fence.tabs[0];
    if (!activeTab) return icons.filter(ic => ic.fence_id === fenceId);
    const activeTabPaths = new Set(activeTab.icon_paths);
    return icons.filter(ic => ic.fence_id === fenceId && activeTabPaths.has(ic.path));
  }

  // ── Tab handlers ──

  function handleFenceTabChange(fenceId: string, tabId: string) {
    fences = fences.map(f => f.id === fenceId ? { ...f, activeTab: tabId } : f);
    saveFences();
  }

  function handleFenceTabAdd(fenceId: string) {
    const newTabId = generateId();
    fences = fences.map(f =>
      f.id === fenceId
        ? { ...f, tabs: [...(f.tabs ?? []), { id: newTabId, name: t.newTab, icon_paths: [] }], activeTab: newTabId }
        : f
    );
    saveFences();
  }

  function handleFenceTabRename(fenceId: string, tabId: string, name: string) {
    fences = fences.map(f =>
      f.id === fenceId
        ? { ...f, tabs: f.tabs?.map(tab => tab.id === tabId ? { ...tab, name } : tab) }
        : f
    );
    saveFences();
  }

  function handleFenceTabDelete(fenceId: string, tabId: string) {
    const fence = fences.find(f => f.id === fenceId);
    if (!fence?.tabs) return;
    const deletedTab = fence.tabs.find(t => t.id === tabId);
    const remainingTabs = fence.tabs.filter(t => t.id !== tabId);
    const newActiveTab = fence.activeTab === tabId ? remainingTabs[0]?.id : fence.activeTab;
    if (remainingTabs.length === 0) {
      fences = fences.map(f => f.id === fenceId ? { ...f, tabs: undefined, activeTab: undefined } : f);
    } else {
      const deletedPaths = deletedTab?.icon_paths ?? [];
      const targetTabId = newActiveTab ?? remainingTabs[0].id;
      fences = fences.map(f => {
        if (f.id !== fenceId) return f;
        return {
          ...f,
          activeTab: newActiveTab,
          tabs: remainingTabs.map(tab =>
            tab.id === targetTabId && deletedPaths.length > 0
              ? { ...tab, icon_paths: [...new Set([...tab.icon_paths, ...deletedPaths])] }
              : tab
          )
        };
      });
    }
    saveFences();
  }

  function handleFenceStyleChange(fenceId: string, style: FenceStyle) {
    fences = fences.map((f) => f.id === fenceId ? { ...f, style: Object.keys(style).length ? style : undefined } : f);
    saveFences();
  }
</script>

<svelte:window onkeydown={(e) => {
  if (e.ctrlKey && e.key === "ArrowRight") { e.preventDefault(); switchToPage(currentPage + 1); }
  if (e.ctrlKey && e.key === "ArrowLeft") { e.preventDefault(); switchToPage(currentPage - 1); }
}} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="desktop-overlay"
  class:anim-off={settings.animationLevel === 'off'}
  class:anim-basic={settings.animationLevel === 'basic'}
  onpointerdown={onDesktopPointerDown}
  onpointermove={onPointerMove}
  onpointerup={onPointerUp}
  oncontextmenu={handleDesktopContext}
  ondblclick={handleDesktopDblClick}
>
  {#if loading}
    <div class="status-pill">
      <span class="spinner">❄</span>
      <span>{t.loading}</span>
    </div>
  {:else if error}
    <div class="status-pill error">
      <span>Error: {error}</span>
    </div>
  {:else}
    <!-- Fences (hidden via double-click toggle) -->
    {#if !fencesHidden}
    {#each fences as fence (fence.id)}
      <Fence
        {fence}
        icons={iconsForFence(fence.id)}
        {dragOverFence}
        onmove={handleFenceMove}
        onresize={handleFenceResize}
        onrename={handleFenceRename}
        onemoji={handleFenceEmoji}
        oncollapse={handleFenceCollapse}
        onfencemenu={handleFenceMenuShow}
        oniconclick={handleFenceIconClick}
        onicondblclick={handleFenceIconDblClick}
        oniconcontextmenu={handleFenceIconContext}
        onicondragstart={handleFenceIconDragStart}
        onsave={saveFences}
        renameToken={fenceRenameTokens[fence.id] ?? 0}
        ontabchange={handleFenceTabChange}
        ontabadd={handleFenceTabAdd}
        ontabrename={handleFenceTabRename}
        ontabdelete={handleFenceTabDelete}
      />
    {/each}
    {/if}

    <!-- Free icons (not in any fence) — also hidden when fences toggled -->
    {#if !fencesHidden}
    {#each freeIcons as icon (icon.path)}
      <button
        class="desktop-icon"
        class:is-dragging={icon.path === draggingIcon && didDrag}
        class:selected={selectedPaths.has(icon.path)}
        style="left: {icon.x}px; top: {icon.y}px"
        onpointerdown={(e) => onIconPointerDown(e, icon)}
        ondblclick={() => handleDoubleClick(icon)}
        oncontextmenu={(e) => handleIconContext(e, icon)}
        title={icon.name}
      >
        {#if icon.icon_data}
          <img
            class="icon-image"
            src="data:image/png;base64,{icon.icon_data}"
            alt={icon.name}
            draggable="false"
            style="width: {settings.iconSize}px; height: {settings.iconSize}px"
          />
        {:else}
          <div class="icon-placeholder" style="width: {settings.iconSize}px; height: {settings.iconSize}px">📄</div>
        {/if}
        {#if !settings.hideIconLabels}<span class="icon-label">{icon.name}</span>{/if}
      </button>
    {/each}
    {/if}

    {#if boxSelecting}
      <div
        class="selection-box"
        style="left:{Math.min(boxStartX, boxCurrentX)}px; top:{Math.min(boxStartY, boxCurrentY)}px; width:{Math.abs(boxCurrentX - boxStartX)}px; height:{Math.abs(boxCurrentY - boxStartY)}px"
      ></div>
    {/if}

    <div class="status-pill">
      {t.statusBar(fences.length, icons.length, currentPage + 1, totalPages)}
      {#if totalPages > 1}
        <span class="page-dots">
          {#each Array(totalPages) as _, i}
            <button
              class="page-dot"
              class:active={i === currentPage}
              onclick={() => switchToPage(i)}
              title="Page {i + 1}"
            ></button>
          {/each}
        </span>
      {/if}
    </div>

    <!-- Edge indicators for page switching -->
    {#if edgeIndicatorVisible}
      <div class="edge-indicator" class:left={edgeIndicatorSide === "left"} class:right={edgeIndicatorSide === "right"}>
        <span class="edge-arrow">{edgeIndicatorSide === "left" ? "◀" : "▶"}</span>
      </div>
    {/if}
  {/if}
</div>

<!-- Icon context menu -->
<ContextMenu
  bind:visible={ctxVisible}
  x={ctxX}
  y={ctxY}
  targetPath={ctxPath}
  targetName={ctxName}
  onclose={() => (ctxVisible = false)}
  ondeleted={handleIconDeleted}
/>

<!-- Fence context menu -->
<FenceMenu
  bind:visible={fenceMenuVisible}
  x={fenceMenuX}
  y={fenceMenuY}
  fenceId={fenceMenuTargetId}
  viewMode={fences.find(f => f.id === fenceMenuTargetId)?.viewMode ?? "grid"}
  onclose={() => (fenceMenuVisible = false)}
  onaction={handleFenceMenuAction}
/>

<!-- Desktop context menu -->
<DesktopMenu
  bind:visible={deskMenuVisible}
  x={deskMenuX}
  y={deskMenuY}
  onclose={() => (deskMenuVisible = false)}
  onaction={handleDesktopMenuAction}
/>

<!-- Settings panel -->
<Settings
  bind:visible={settingsVisible}
  onclose={() => (settingsVisible = false)}
/>

<!-- Search bar (Alt+Space) -->
<SearchBar
  bind:visible={searchVisible}
  onclose={() => (searchVisible = false)}
/>

<!-- Scene manager -->
<SceneManager
  bind:visible={sceneManagerVisible}
  onclose={() => (sceneManagerVisible = false)}
  onsave={saveAll}
  onload={handleSceneLoad}
/>

<!-- Rules editor -->
<RulesEditor
  bind:visible={rulesEditorVisible}
  {fences}
  onclose={() => (rulesEditorVisible = false)}
/>

<!-- Fence style editor -->
<FenceStyleEditor
  bind:visible={styleEditorVisible}
  x={styleEditorX}
  y={styleEditorY}
  style={fences.find(f => f.id === styleEditorFenceId)?.style ?? {}}
  onchange={(s) => handleFenceStyleChange(styleEditorFenceId, s)}
  onclose={() => (styleEditorVisible = false)}
/>

<!-- Archive confirmation dialog -->
<ArchiveDialog
  bind:visible={archiveDialogVisible}
  preview={archivePreview}
  onconfirm={handleArchiveConfirm}
  oncancel={() => { archiveDialogVisible = false; archivePreview = null; archiveTargetFenceId = ""; }}
/>

<!-- First-run onboarding -->
<Onboarding
  bind:visible={onboardingVisible}
  onclose={() => (onboardingVisible = false)}
  oncreatefences={handleOnboardingFences}
/>

<style>
  .desktop-overlay {
    position: fixed;
    inset: 0;
  }

  :global(.anim-off *) {
    transition: none !important;
    animation: none !important;
  }

  :global(.anim-basic *) {
    animation: none !important;
  }

  .status-pill {
    position: fixed;
    bottom: 16px;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 22px;
    background: var(--glass-bg);
    border: 1px solid var(--glass-border);
    border-radius: 14px;
    backdrop-filter: blur(20px);
    color: var(--text);
    font-size: 13px;
    white-space: nowrap;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
    z-index: 500;
  }

  .status-pill.error {
    border-color: #e04060;
    color: #ff8090;
  }

  .spinner {
    display: inline-block;
    font-size: 18px;
    animation: spin 2s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .desktop-icon {
    position: absolute;
    width: 80px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 6px 4px;
    border-radius: 8px;
    border: none;
    background: transparent;
    cursor: grab;
    color: var(--text);
    transition: background 0.15s, opacity 0.1s;
    touch-action: none;
    z-index: 5;
  }

  .desktop-icon:hover {
    background: rgba(255, 255, 255, 0.12);
  }

  .desktop-icon:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .desktop-icon:focus .icon-label,
  .desktop-icon:focus-visible .icon-label {
    white-space: normal;
    word-break: break-all;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    background: rgba(0, 0, 0, 0.4);
    border-radius: 2px;
    padding: 1px 3px;
  }

  .desktop-icon.selected {
    background: rgba(91, 141, 239, 0.25);
    outline: 2px solid var(--accent);
  }

  .desktop-icon.is-dragging {
    opacity: 0.35;
    cursor: grabbing;
  }

  .selection-box {
    position: absolute;
    border: 1px solid var(--accent);
    background: rgba(91, 141, 239, 0.15);
    z-index: 50;
    pointer-events: none;
  }

  .icon-image {
    width: 48px;
    height: 48px;
    pointer-events: none;
    image-rendering: auto;
    object-fit: contain;
  }

  .icon-placeholder {
    width: 48px;
    height: 48px;
    display: grid;
    place-items: center;
    font-size: 28px;
  }

  .icon-label {
    font-size: 11px;
    text-align: center;
    line-height: 1.25;
    max-width: 76px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: #fff;
    font-weight: 400;
    text-shadow:
      0 0 2px rgba(0, 0, 0, 0.9),
      0 0 4px rgba(0, 0, 0, 0.7),
      0 0 6px rgba(0, 0, 0, 0.4);
    pointer-events: none;
  }

  .page-dots {
    display: inline-flex;
    gap: 6px;
    align-items: center;
    margin-left: 8px;
  }

  .page-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    border: 1px solid rgba(255, 255, 255, 0.3);
    background: rgba(255, 255, 255, 0.15);
    cursor: pointer;
    padding: 0;
    transition: background 0.2s, transform 0.15s;
  }

  .page-dot:hover {
    background: rgba(255, 255, 255, 0.35);
    transform: scale(1.3);
  }

  .page-dot.active {
    background: var(--accent);
    border-color: var(--accent);
  }

  .edge-indicator {
    position: fixed;
    top: 0;
    bottom: 0;
    width: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9000;
    pointer-events: none;
    animation: edge-pulse 0.5s ease-in-out infinite alternate;
  }

  .edge-indicator.left { left: 0; }
  .edge-indicator.right { right: 0; }

  .edge-arrow {
    font-size: 24px;
    color: var(--accent);
    text-shadow: 0 0 12px var(--accent-glow), 0 0 24px var(--accent-glow);
  }

  @keyframes edge-pulse {
    from { opacity: 0.4; }
    to { opacity: 1; }
  }
</style>
