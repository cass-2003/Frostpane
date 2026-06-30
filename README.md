<div align="center">

# Frostpane · 方寸

**更智能的 Windows 桌面整理工具**
Fences 式半透明视觉分区 · 一键真实归档 · AI 智能识别分区 · 多语言

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
![Stage](https://img.shields.io/badge/stage-inception-orange.svg)

</div>

---

## 这是什么

Frostpane（方寸）是一款桌面整理工具，对标 [Stardock Fences](https://www.stardock.com/products/fences/)，但更智能、更好用：

- 🪟 **视觉分区**：在桌面画半透明分区（栅栏），拖图标分组，桌面瞬间清爽
- 📦 **真实归档**：一键把桌面文件移入对应文件夹，全程可撤销 + 可审计
- 🤖 **AI 智能识别分区**（杀手锏）：逐个分析桌面软件的真实用途，不认识的联网搜索，深度思考后给出按用途的分类方案（Photoshop→设计、Steam→游戏、VSCode→开发……）
- 🖥️ **多屏记忆**：记住每个分辨率/显示器的图标布局，接投屏/换分辨率不再乱跳
- 🔍 **快速搜索**：一键搜桌面与常用文件夹
- 🌍 **多语言**：默认英文，架构支持任意语言包

## 技术栈

- **Tauri**（Rust 后端 + Web 前端）
- Win10/11 优先，架构预留 macOS
- 视觉：Win11 Fluent 毛玻璃 + 高级动效 + 可换主题

## 商业模式

**Open Core**：核心功能 Apache-2.0 开源免费，AI 识别 / 云同步 / 高级主题作增值付费。

## 项目状态

🚧 **inception（规划阶段）** — 当前只有文档，未开始编码。

- 📄 [产品需求文档 PRD](docs/PRD.md)
- 🗺️ [路线图 M0–M4](docs/planning/roadmap.md)
- 🏗️ [架构概览](docs/architecture/overview.md)
- 📊 [进度看板](state/PROGRESS.md)

## License

[Apache License 2.0](LICENSE) © Frostpane
