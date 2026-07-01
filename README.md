<div align="center">

# Frostpane · 方寸

**更智能的 Windows 桌面整理工具**

Fences 式半透明视觉分区 · 一键真实归档 · 规则自动归类 · Portal 文件夹映射 · 多标签页分区 · 桌面分页 · 全盘搜索 · 多语言

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
![Stage](https://img.shields.io/badge/阶段-M1_功能原型-green.svg)
![Tech](https://img.shields.io/badge/Tauri_2-Rust_%2B_Svelte_5-blue.svg)

</div>

---

## 这是什么？

Frostpane（方寸）是一款 Windows 桌面整理工具，对标 [Stardock Fences](https://www.stardock.com/products/fences/)，更智能、更好看、核心免费：

- 🪟 **视觉分区** — 在桌面画半透明毛玻璃栅栏，拖图标分组，桌面瞬间清爽
- 📦 **真实归档** — 一键把分区内文件移入真实文件夹，全程可撤销 + 审计日志
- 📂 **Portal 分区** — 映射任意文件夹到桌面分区，实时显示文件夹内容
- 🔄 **规则引擎** — 按文件扩展名/名称前缀/名称包含自动归入指定分区
- 🎨 **Magic Fences** — 从图标自动采样主色调，分区背景色与内容视觉和谐
- 🔍 **全盘搜索** — Alt+Space 唤起，异步遍历桌面/文档/下载/Program Files
- 🎭 **场景系统** — 保存/加载完整桌面布局，工作/娱乐一键切换
- 🌍 **多语言** — 跟随系统语言，中英文内置
- 🧹 **坏快捷方式检测** — 扫描失效 .lnk 快捷方式，一键清理

## 功能一览

### 分区管理
- 毛玻璃栅栏，拖入/拖出图标，移动/缩放/折叠/重命名
- Emoji 图标选择器（24 种预设）
- 网格视图 / 列表视图切换
- 分区独立样式（颜色预设/透明度/圆角滑条）
- 磁性吸附对齐（屏幕边缘 + 分区间 12px 阈值）
- 框选（Rubber Band）+ Ctrl 多选 + 批量拖拽
- 双击桌面空白 → 隐藏/显示所有分区

### Portal 分区（文件夹映射）
- 绑定任意文件夹路径，实时显示文件夹内容
- 双击打开文件，支持网格/列表视图
- 刷新按钮 + 在资源管理器中打开

### 规则引擎
- 按文件扩展名（如 `*.pdf → 📁 文档`）
- 按名称包含（如 `项目A → 💼 项目A`）
- 按名称前缀匹配
- 启动时自动归类未分配图标

### 真实归档
- 预览归档操作（文件数/目标路径）
- 执行后生成 JSON 审计日志
- 按操作 ID 一键撤销

### 搜索
- Alt+Space 全局唤起毛玻璃搜索栏
- 300ms 防抖，方向键导航，Enter 打开
- 异步遍历 Desktop/Documents/Downloads/Program Files/Start Menu

### 个性化
- 图标大小：32/48/64/96px
- 动画等级：关闭/基础/完整
- 隐藏图标名称（极简模式）
- Magic Fences 自动取色（从图标 PNG 采样主色调）
- 分区独立配色（6 种预设 + 自定义透明度/圆角）
- 开机自启（可关）

### 系统集成
- 系统托盘图标（Show/Hide、隐藏真实桌面图标、Settings、Exit）
- 关闭窗口 → 最小化到托盘（不退出）
- 单实例控制（防重复打开，自动聚焦已有窗口）
- 右键菜单：打开/管理员运行/打开位置/重命名/删除/复制路径/原生 Shell IContextMenu
- 首次启动引导（三步：欢迎 → 选模板 → 操作提示）
- 启动自动备份（保留最近 10 个快照）

## 技术栈

| 层 | 技术 |
|---|---|
| 框架 | Tauri 2.11（Rust + Web） |
| 前端 | Svelte 5（runes）+ TypeScript + Vite |
| 图标 | Win32 SHIL_JUMBO 256px + PNG 编码 + base64 |
| 右键 | Win32 IContextMenu COM + 快捷菜单 |
| 搜索 | walkdir + tokio 异步 |
| 存储 | JSON 配置 + SQLite（预留 AI 学习） |
| 平台 | Windows 10/11，架构预留 macOS |

## 从源码构建

环境要求：[Rust](https://rustup.rs/) 1.77+、[Node.js](https://nodejs.org/) 20+

```bash
git clone https://github.com/cass-2003/Frostpane.git
cd Frostpane
npm install
cargo tauri dev      # 开发模式（热更新）
cargo tauri build    # 生产构建（MSI + 绿色版）
```

## 项目状态

**M1 · 功能原型**（进行中）

| 功能 | 状态 |
|---|---|
| 透明覆盖窗口 + 壁纸可见 | ✅ |
| 256px 高清图标提取（SHIL_JUMBO） | ✅ |
| 双击打开 / 管理员运行 / 打开位置 | ✅ |
| 右键菜单（快捷 + 原生 Shell） | ✅ |
| 拖拽移动 + 网格吸附 + 持久化 | ✅ |
| 视觉分区（Fence） | ✅ |
| 磁性吸附对齐 | ✅ |
| 框选 + Ctrl 多选 + 批量拖拽 | ✅ |
| 场景系统（保存/加载布局） | ✅ |
| i18n 中英双语 | ✅ |
| 系统托盘 + 关闭到托盘 | ✅ |
| 首次引导教程 | ✅ |
| 全盘搜索（Alt+Space） | ✅ |
| 真实归档 + 撤销 + 审计日志 | ✅ |
| 列表视图切换 | ✅ |
| 设置面板（开机自启/图标大小/动画/语言） | ✅ |
| 隐藏图标名称 | ✅ |
| 单实例控制 | ✅ |
| 坏快捷方式检测 + 清理 | ✅ |
| 分区独立样式（颜色/透明度/圆角） | ✅ |
| 规则引擎（自动归入分区） | ✅ |
| 分区样式编辑器（预设+滑条） | ✅ |
| Magic Fences 自动取色 | ✅ |
| Portal 分区（文件夹映射） | ✅ |
| Desktop Pages 桌面分页 | 🚧 |
| 分区多标签页 | 🚧 |
| 全局快捷键（系统级） | 🚧 |
| AI 智能分类 | ⬜ |
| 多显示器布局记忆 | ⬜ |
| Chameleon 融入壁纸 | ⬜ |
| 自动更新 | ⬜ |

## 商业模式

**Open Core**：核心功能 Apache-2.0 开源免费，AI 识别 / 云同步 / 高级主题作增值付费。

## License

[Apache License 2.0](LICENSE) © Frostpane
