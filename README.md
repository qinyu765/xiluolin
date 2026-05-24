# XiLuoLin

XiLuoLin 是一个面向办公、写作和编程场景的 AI 语音输入助手。当前仓库已完成 Tauri + React + TypeScript + pnpm 的项目骨架初始化，后续会在这个基础上继续实现语音输入、人格化整理、历史记录和统计能力。

## 项目状态

- 当前阶段：Tauri v2 + React 基础骨架
- 工作分支：`dev`
- 目标：先跑通桌面应用工程结构，再逐步接入语音输入流程

## 依赖

- Node.js 20+
- pnpm 10+
- Rust 工具链（通过 `rustup` 安装）
- Microsoft Visual Studio C++ Build Tools
- WebView2 Runtime

## 本地运行

```bash
pnpm install
pnpm build
pnpm tauri dev
```

## 说明

- 当前仓库不包含 `.env`、真实 API Key 或录音临时文件。
- 本次初始化只建立桌面应用骨架，不包含 ASR、OpenAI、SQLite 或业务流程。
- 第三方依赖用途：
  - `@tauri-apps/api`：前端调用桌面端能力。
  - `@tauri-apps/cli`：Tauri 构建和开发命令。
  - `react` / `react-dom`：前端界面。
  - `vite`：前端开发和构建。
  - `typescript`：类型检查。

## 原创说明

本项目以语音输入助手为主题，当前仓库中的产品结构、任务拆分和骨架页面均为本项目自建内容。后续若接入第三方服务，会在文档中明确来源和用途。
