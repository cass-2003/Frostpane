<div align="center">

# Frostpane · 方寸

**更智能的 Windows 桌面整理工具**

Fences 式半透明视觉分区 · 一键真实归档 · AI 智能识别分区 · 多语言

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
![Stage](https://img.shields.io/badge/阶段-M0_技术验证-yellow.svg)
![Tech](https://img.shields.io/badge/Tauri_2-Rust_%2B_Svelte_5-blue.svg)

</div>

---

## 这是什么？

Frostpane（方寸）是一款 Windows 桌面整理工具，对标 [Stardock Fences](https://www.stardock.com/products/fences/)，更智能、更好看、核心免费：

- 🪟 **视觉分区** — 在桌面画半透明栅栏，拖图标分组，桌面瞬间清爽
- 📦 **真实归档** — 一键把桌面文件移入对应文件夹，全程可撤销 + 可审计
- 🤖 **AI 智能识别分区**（杀手锏）— 逐个分析桌面软件的真实用途，不认识的联网搜索，深度思考后给出分类方案（Photoshop→设计、Steam→游戏、VSCode→开发……）
- 🖥️ **多屏记忆** — 记住每个分辨率/显示器的图标布局，接投屏/换分辨率不再乱跳
- 🔍 **全盘搜索** — 全局快捷键唤起，秒搜桌面与任意文件夹
- 🌍 **多语言** — 跟随系统语言，中英文内置，可扩展任意语言包

## 技术栈

- **Tauri 2**（Rust 后端 + Svelte 5 前端）
- Windows 10/11 优先，架构预留 macOS
- Win11 Fluent 毛玻璃 + 高级动效 + 可换主题

## 从源码构建

环境要求：[Rust](https://rustup.rs/) 1.77+、[Node.js](https://nodejs.org/) 20+

```bash
git clone https://github.com/cass-2003/Frostpane.git
cd Frostpane
npm install
cargo tauri dev      # 开发模式（热更新）
cargo tauri build    # 生产构建
```

## 项目状态

🔨 **M0 · 技术验证**（进行中）

- ✅ 透明覆盖窗口，壁纸可见
- ✅ 桌面图标自绘，256px 高清提取（SHIL_JUMBO）
- ✅ 双击打开（ShellExecuteW）、管理员运行、打开文件位置
- ✅ 右键菜单（快捷操作 + 原生 Shell 菜单）
- ✅ 拖拽移动 + 网格吸附 + 位置持久化
- ⬜ 桌面层级嵌入（WorkerW）
- ⬜ 多显示器定位
- ⬜ 内存 / 启动速度测量

📄 [产品需求文档 PRD](docs/PRD.md) · 🗺️ [路线图 M0–M4](docs/planning/roadmap.md) · 🏗️ [架构概览](docs/architecture/overview.md) · 📊 [进度看板](state/PROGRESS.md)

## 商业模式

**Open Core**：核心功能 Apache-2.0 开源免费，AI 识别 / 云同步 / 高级主题作增值付费。

## License

[Apache License 2.0](LICENSE) © Frostpane
