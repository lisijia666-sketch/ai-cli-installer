# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

AI 工具安装器 — 基于 Tauri v2 的桌面应用，用于一键安装和管理 AI 编程 CLI 工具。

## 常用命令

```bash
# 开发（前端热重载 + Rust 后端）
pnpm tauri dev

# 仅前端开发
pnpm dev

# 构建生产版本（生成 .exe 安装包）
pnpm tauri build

# 安装依赖
pnpm install
```

构建产物路径：`src-tauri/target/release/bundle/nsis/`

## 技术栈

- **前端**: React 18 + TypeScript + Tailwind CSS + Vite
- **后端**: Rust + Tauri v2
- **包管理**: pnpm
- **图标**: lucide-react

## 架构

### 前后端通信

前端通过 `@tauri-apps/api` 的 `invoke()` 调用 Rust 后端命令。所有后端命令定义在 `src-tauri/src/commands.rs`，在 `src-tauri/src/lib.rs` 的 `invoke_handler` 中注册。

### 后端命令（commands.rs）

四个核心 Tauri 命令，均为 `async` + `spawn_blocking` 避免阻塞 UI：
- `check_tool(tool)` — 检测工具是否已安装及版本
- `install_tool(tool)` — 安装/升级工具
- `uninstall_tool(tool)` — 卸载工具
- `open_tool(tool)` — 打开已安装的工具

### 工具分类

| 类型 | 工具 | 安装方式 |
|------|------|----------|
| npm 包 | Claude Code, Codex, Gemini CLI, OpenCode, OpenClaw, MiMo Code | `npm install -g` |
| 原生应用 | Hermes | GitHub 脚本安装 |
| 桌面应用 | CC-Switch | 下载 .msi 安装 |

npm 包类工具使用 `install_npm_package()` / `uninstall_npm_package()` 通用函数。
非 npm 工具（Hermes、CC-Switch）有独立的 `install_*` / `uninstall_*` 函数。

### 前端结构

- `src/components/Installer.tsx` — 唯一的业务组件，包含工具列表、状态管理、安装逻辑
- `src/App.tsx` — 入口，直接渲染 Installer
- 工具定义在 `initialSteps` 数组中，每个工具有 `id`、`name`、`description`、`icon`

### Windows 特殊处理

所有 `std::process::Command` 调用在 Windows 上使用 `cmd /C` 包装，并添加 `CREATE_NO_WINDOW` (0x08000000) 标志避免弹出控制台窗口。

## 添加新工具

1. **后端** (`commands.rs`): 添加 `check_*` 函数，然后在 `check_tool`、`install_tool`、`uninstall_tool`、`open_tool` 的 match 中注册
2. **前端** (`Installer.tsx`): 在 `initialSteps` 数组中添加条目

## 主题

深色主题，自定义颜色：
- `surface-0` (#0d0d0d) 到 `surface-4` (#383838) — 背景层次
- `accent` (#d4a574) — 主色调（暖橙色）
