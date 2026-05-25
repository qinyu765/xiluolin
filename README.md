# XiLuoLin
b站演示视频：https://www.bilibili.com/video/BV1mEGo66EBB/?vd_source=12fed1699cf3108d48cd967e825a59e1

XiLuoLin 是一个面向办公、写作和编程场景的 AI 语音输入助手。通过语音输入、智能识别、人格化整理，帮助用户快速将口述内容转化为可直接使用的文本，减少打字、编辑和润色成本。

## 核心功能

- **语音输入能力**：Rust 侧已实现麦克风录音命令、短音频处理命令、全局快捷键注册和录音指示器窗口；当前首页暂未展示录音 / 上传入口，快捷键事件到前端处理链路仍待联调
- **智能识别**：已接入智谱 GLM-ASR-2512 Provider，用于将短音频转为文本
- **人格化整理**：已接入 OpenAI Responses API，根据选定人格（Prompt 工程师、任务协作者、灵感整理师、正式消息助手）整理文本风格和结构
- **热词词典**：支持自定义专有名词、技术词、项目名，减少误识别
- **历史记录**：保存每次输入的原始识别文本、整理后文本、使用人格、录音时长
- **统计反馈**：展示语音协作次数、累计口述时间、生成字数、预计节省时间、常用人格
- **输出方式**：Rust 侧已实现复制到剪贴板和自动粘贴兜底能力，真实跨应用输出仍需桌面端手动验证

## 项目状态

- 当前阶段：MVP 核心模块已实现，快捷键录音事件已接入前端，正在进行端到端验证
- 工作分支：`dev`
- 已完成代码层能力：Tauri v2 + React 基础骨架、本地数据层、内置人格与自定义人格、热词词典、智谱 ASR Provider、OpenAI 文本整理 Provider、短音频处理流程、历史记录、统计卡片、录音模块、全局快捷键注册、复制与自动粘贴模块、错误提示、左侧导航、人格管理页、热词页和设置页、快捷键录音事件监听和自动输出
- 当前界面：采用左侧导航结构，包含首页、人格、热词、设置四个页面；首页当前聚焦当前人格问候、快捷键提示、统计卡片和时间分段历史记录
- 当前限制：首页的 `QuickStartCard` 录音 / 上传入口在代码中被注释隐藏；`AppSettingsDialog` 仍存在但当前主入口未使用
- 待完成验证：UI 重构端到端验证（T028）、真实 API Key 和样例音频下的 ASR + OpenAI smoke test、快捷键录音触发后的桌面端完整联调、首页语音输入入口恢复或替代方案确认、录音指示器窗口打包路径验证
- 前端 UI 方向：采用 Tailwind CSS + shadcn/ui，参考 Notion 风格的桌面效率工具界面

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

## 演示复现

评审或本地演示建议按以下路径验证：

1. 运行 `pnpm install` 安装依赖。
2. 运行 `pnpm build` 验证前端构建。
3. 运行 `pnpm tauri dev` 启动桌面应用。
4. 进入设置页，配置智谱 GLM-ASR-2512 和 OpenAI Responses API 的 API Key、Base URL 和模型名。
5. 在设置页配置长按模式和切换模式快捷键、麦克风设备、输出方式和自动保存历史。
6. 进入人格页，查看 4 个内置人格，按需创建自定义人格并设为默认。
7. 进入热词页，添加项目名、技术词或常见误识别词，启用后确认热词上下文更新。
8. 回到首页，查看当前人格、快捷键提示、统计卡片和历史记录。
9. 记录当前限制：首页未展示录音 / 上传入口，快捷键录音事件尚未接入前端处理流程。

当前 T028 仍在待验证状态。若需要演示真实语音输入闭环，需要先恢复首页 `QuickStartCard` 或补齐快捷键事件监听，再配置真实 API Key、麦克风权限和样例音频执行 smoke test。

## 使用场景

- **开发者**：用语音快速组织和改进 Prompt，配合 Agent 工具或 vibe coding 工作流
- **团队协作**：将口述的任务指令或协作消息转化为清晰、结构化的文本
- **创作者**：快速捕捉灵感，将口述内容转化为标题、要点、草稿或任务列表

## 隐私与安全

- 音频发送给用户配置的智谱 GLM-ASR-2512 服务
- 原始识别文本发送给用户配置的 OpenAI Responses API 服务
- API Key 保存在本地配置中，不写入项目文件，不提交到 Git
- 历史记录、人格、热词和统计数据保存在本地 SQLite，不上传云端

## 说明

- 当前仓库不包含 `.env`、真实 API Key 或录音临时文件。
- 本项目为七牛云 2026 黑客松参赛作品，选题为"语音输入法"。
- 第三方依赖用途：
  - `@radix-ui/react-dialog`：为 shadcn/ui 弹窗组件提供无障碍交互基础。
  - `@radix-ui/react-label`：为 shadcn/ui 表单标签组件提供无障碍交互基础。
  - `@radix-ui/react-select`：为 shadcn/ui 选择器组件提供键盘操作和弹层交互。
  - `@radix-ui/react-slot`：为 shadcn/ui 组件组合能力提供基础。
  - `@radix-ui/react-switch`：为 shadcn/ui 开关组件提供无障碍交互基础。
  - `@radix-ui/react-tabs`：为 shadcn/ui 标签页组件提供键盘操作基础。
  - `@tailwindcss/vite`：在 Vite 构建中接入 Tailwind CSS。
  - `@tauri-apps/api`：前端调用桌面端能力。
  - `@tauri-apps/cli`：Tauri 构建和开发命令。
  - `@vitejs/plugin-react`：为 Vite 提供 React 编译支持。
  - `class-variance-authority`：管理 shadcn/ui 组件变体。
  - `clsx`：组合条件 class。
  - `lucide-react`：提供界面图标。
  - `react` / `react-dom`：前端界面。
  - `sonner`：提供前端 toast 提示，用于错误、保存和处理结果反馈。
  - `tailwind-merge`：合并 Tailwind class，避免样式冲突。
  - `tailwindcss`：编译生成项目实际使用到的样式。
  - `vite`：前端开发和构建。
  - `typescript`：类型检查。
  - `tw-animate-css`：提供 shadcn/ui 推荐的动画工具样式。
  - `tauri`：Rust 侧桌面应用框架，负责窗口、命令和插件集成。
  - `tauri-build`：Tauri 构建脚本依赖，用于生成桌面端构建上下文。
  - `tauri-plugin-store`：Rust 侧保存默认人格、快捷键和输出方式等轻量配置。
  - `tauri-plugin-sql`：Tauri 官方 SQLite 插件，已注册到桌面端，为后续前端数据访问预留接口。
  - `tauri-plugin-global-shortcut`：注册全局快捷键，支持长按录音和切换式录音入口。
  - `tauri-plugin-opener`：提供 Tauri 默认打开外部资源能力。
  - `rusqlite`：Rust 侧直接管理本地业务表。
  - `uuid`：生成本地业务数据 ID。
  - `serde` / `serde_json`：序列化和反序列化前后端命令数据。
  - `ureq`：Rust 侧调用智谱 GLM-ASR-2512 `audio/transcriptions` 接口和 OpenAI Responses API，发送短音频 multipart 请求与文本整理 JSON 请求。
  - `reqwest`：为音频和模型服务调用保留 HTTP multipart / JSON 能力。
  - `cpal`：采集麦克风音频输入。
  - `hound`：写入 WAV 录音文件。
  - `chrono`：生成时间戳和处理本地记录时间。
  - `tokio`：支撑 Tauri 异步命令和后台任务。
  - `enigo`：模拟键盘输入，实现自动粘贴或直接输入能力。
  - `arboard`：访问系统剪贴板，作为复制和自动粘贴兜底。
  - `thiserror`：定义 Rust 侧错误类型，生成更明确的错误提示。
  - `@types/node`：为 Vite 配置中的 Node API 提供类型定义。
  - `@types/react` / `@types/react-dom`：为 React 组件和渲染入口提供 TypeScript 类型。

## 前端 UI 选型

本项目采用 shadcn/ui + Tailwind CSS 作为前端 UI 基础。shadcn/ui 组件会以源码形式进入项目，方便按语音输入助手的桌面工具场景做局部定制；Tailwind CSS 负责布局、间距、状态和设计 token。

使用约束：

- 按任务需要添加 shadcn/ui 组件，不一次性引入完整组件集合。
- Tailwind class 使用完整、可静态识别的写法，避免动态拼接 class。
- 新增依赖后必须在本 README 中补充来源和用途。
- UI 改动需要通过 `pnpm build`，并在可行时通过 `pnpm tauri dev` 做桌面端手动检查。

设计取舍：

- 当前采用 `https://getdesign.md/notion/design-md` 作为 UI 视觉参考，原因是它更贴近 productivity SaaS、knowledge management 和 workspace tools。
- 已评估 `https://getdesign.md/hp/design-md`。该风格更偏企业官网和产品目录，不完整注入到当前桌面效率工具。
- 视觉 token 记录在 `docs/design/ui-style.md`。

## 原创说明

本项目以语音输入助手为主题，当前仓库中的产品结构、任务拆分和骨架页面均为本项目自建内容。后续若接入第三方服务，会在文档中明确来源和用途。

## 项目资源说明

- `src-tauri/indicator.html`：录音指示器窗口资源；Rust 侧 `indicator` 模块在开发模式读取根目录 `indicator.html`（已删除），生产模式使用 `WebviewUrl::App("indicator.html")`，打包资源配置仍需验证。
