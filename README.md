# XiLuoLin

XiLuoLin 是一个面向办公、写作和编程场景的 AI 语音输入助手。通过语音输入、智能识别、人格化整理，帮助用户快速将口述内容转化为可直接使用的文本，减少打字、编辑和润色成本。

## 核心功能

- **语音输入**：支持麦克风录音和全局快捷键触发（长按模式 / 切换模式）
- **智能识别**：接入智谱 GLM-ASR-2512，将语音转为文本
- **人格化整理**：基于 OpenAI Responses API，根据选定人格（Prompt 工程师、任务协作者、灵感整理师、正式消息助手）整理文本风格和结构
- **热词词典**：支持自定义专有名词、技术词、项目名，减少误识别
- **历史记录**：保存每次输入的原始识别文本、整理后文本、使用人格、录音时长
- **统计反馈**：展示语音协作次数、累计口述时间、生成字数、预计节省时间、常用人格
- **输出方式**：支持复制到剪贴板或自动粘贴到当前输入位置

## 项目状态

- 当前阶段：MVP 开发中
- 工作分支：`dev`
- 已完成：Tauri v2 + React 基础骨架、本地数据层、智谱 ASR Provider、OpenAI 文本整理 Provider
- 进行中：录音模块、全局快捷键、主界面、历史页、人格页、热词页、统计页、设置页
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
  - `class-variance-authority`：管理 shadcn/ui 组件变体。
  - `clsx`：组合条件 class。
  - `lucide-react`：提供界面图标。
  - `react` / `react-dom`：前端界面。
  - `tailwind-merge`：合并 Tailwind class，避免样式冲突。
  - `tailwindcss`：编译生成项目实际使用到的样式。
  - `vite`：前端开发和构建。
  - `typescript`：类型检查。
  - `tw-animate-css`：提供 shadcn/ui 推荐的动画工具样式。
  - `tauri-plugin-store`：Rust 侧保存默认人格、快捷键和输出方式等轻量配置。
  - `tauri-plugin-sql`：Tauri 官方 SQLite 插件，已注册到桌面端，为后续前端数据访问预留接口。
  - `rusqlite`：Rust 侧直接管理本地业务表。
  - `ureq`：Rust 侧调用智谱 GLM-ASR-2512 `audio/transcriptions` 接口和 OpenAI Responses API，发送短音频 multipart 请求与文本整理 JSON 请求。
  - `@types/node`：为 Vite 配置中的 Node API 提供类型定义。

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
