<div align="center">

# Frostpane · 方寸

**A smarter Windows desktop organizer**

Fences-style translucent desktop zones · one-click file archiving · AI-powered icon classification · multilingual

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
![Stage](https://img.shields.io/badge/stage-M0_tech_validation-yellow.svg)
![Tech](https://img.shields.io/badge/Tauri_2-Rust_%2B_Svelte_5-blue.svg)

</div>

---

## What is this?

Frostpane (方寸) is a desktop organizer for Windows, built to beat [Stardock Fences](https://www.stardock.com/products/fences/) — smarter, prettier, and free at its core:

- 🪟 **Visual zones** — draw translucent fences on your desktop, drag icons to group them, instantly tidy
- 📦 **Real archiving** — one click moves desktop files into proper folders, fully undoable + audit trail
- 🤖 **AI-powered classification** (killer feature) — analyzes each shortcut/app's real purpose, searches the web for unknown ones, then proposes a smart grouping (Photoshop→Design, Steam→Games, VSCode→Dev…)
- 🖥️ **Multi-monitor memory** — remembers icon layouts per resolution/display, no more scrambled icons after plugging in a projector
- 🔍 **Full-disk search** — instant launcher for desktop + any folder, like Listary/Everything
- 🌍 **Multilingual** — follows system locale, English & Chinese built-in, extensible language packs

## Tech stack

- **Tauri 2** (Rust backend + Svelte 5 frontend)
- Windows 10/11 first, architecture ready for macOS
- Win11 Fluent glass (Mica/Acrylic) + fluid animations + themeable

## Building from source

Prerequisites: [Rust](https://rustup.rs/) 1.77+, [Node.js](https://nodejs.org/) 20+

```bash
git clone https://github.com/AYuQian/Frostpane.git
cd Frostpane
npm install
cargo tauri dev      # dev mode with hot reload
cargo tauri build    # production build
```

## Project status

🔨 **M0 — Technical validation** (in progress)

- ✅ Transparent overlay window on desktop
- ✅ Self-drawn desktop icons with high-res extraction (256px SHIL_JUMBO)
- ✅ Double-click to open (ShellExecuteW), run as admin, open file location
- ✅ Right-click context menu (quick actions + native Shell menu)
- ✅ Drag-and-drop with grid snapping + position persistence
- ⬜ Desktop-level window embedding (WorkerW)
- ⬜ Multi-monitor positioning
- ⬜ Memory & startup benchmarks

📄 [Product Requirements (PRD)](docs/PRD.md) · 🗺️ [Roadmap M0–M4](docs/planning/roadmap.md) · 🏗️ [Architecture](docs/architecture/overview.md) · 📊 [Progress](state/PROGRESS.md)

## Business model

**Open Core**: core features are Apache-2.0 open source and free forever. AI classification, cloud sync, and premium themes are paid add-ons.

## License

[Apache License 2.0](LICENSE) © Frostpane
