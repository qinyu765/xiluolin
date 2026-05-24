# XiLuoLin

XiLuoLin 是一个面向办公、写作和编程场景的 AI 语音输入助手。当前仓库已完成 Tauri + React + TypeScript + pnpm 的项目骨架初始化，后续会在这个基础上继续实现语音输入、人格化整理、历史记录和统计能力。
当前仓库也已建立本地 SQLite + Store 数据层，用于保存人格、热词、历史记录和轻量配置。

## 项目状态

- 当前阶段：Tauri v2 + React 基础骨架
- 工作分支：`dev`
- 目标：先跑通桌面应用工程结构，再逐步接入语音输入流程
- 前端 UI 方向：采用 Tailwind CSS + shadcn/ui，优先支持设置、人格选择、热词词典、历史记录和统计卡片等桌面工具界面。

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
- 本次已建立桌面应用骨架和本地数据层，但仍不包含 ASR、OpenAI 或完整语音输入业务流程。
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
