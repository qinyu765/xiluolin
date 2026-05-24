# 开发任务跟踪表

> 本文件是项目开发任务状态的唯一事实源。执行下一个任务前必须先读取本文件；任务状态变化后必须更新本文件。
> 当前工作方式：直接在 `dev` 上开发；阶段性成果通过 `dev -> main` 的 PR 交付。需要 commit 或 PR 时，必须先向用户申请并获得明确同意。

## 状态说明

| 状态 | 含义 |
|---|---|
| Todo | 尚未开始 |
| Doing | 正在执行。任意时间最多只能有一个任务处于此状态 |
| Review | 已完成实现和验证，等待审查、提交或 PR |
| Done | 已合入或经用户明确确认完成 |
| Blocked | 被明确问题阻塞，备注中必须说明阻塞原因 |

## 任务表

| ID | 优先级 | 任务 | 目标 | 状态 | 分支 | PR | 验证方式 | 开发文档 | 备注 |
|---|---|---|---|---|---|---|---|---|---|
| T001 | P0 | 开发流程基础设施 | 创建执行 skill、任务跟踪表、dev 文档模板，并明确分支、测试、commit、PR、文档规则 | Review | dev | 待创建 | 检查文件存在、skill frontmatter、任务表列和 T001 文档 | [2026-05-24-t001-dev-workflow-infrastructure.md](./2026-05-24-t001-dev-workflow-infrastructure.md) | 当前工作方式已切换为直接在 `dev` 上开发 |
| T002 | P0 | 初始化 Tauri + React + pnpm 项目 | 直接在 `dev` 上初始化 Tauri v2、React、TypeScript、pnpm，并补充 README 运行说明 | Review | dev | 待创建 | `pnpm install`、`pnpm tauri dev` 或对应初始化验证 | [2026-05-24-t002-init-tauri-react-pnpm.md](./2026-05-24-t002-init-tauri-react-pnpm.md) | Rust / C++ Build Tools 已补齐；`pnpm build` 与 `pnpm tauri dev` 已验证通过 |
| T003 | P0 | 建立本地数据层 | 直接在 `dev` 上建立人格、热词、历史记录数据模型，使用 SQLite 保存业务数据，使用 Tauri Store 保存配置 | Review | dev | 待创建 | `cargo test --test local_data_layer`、`cargo check`、`pnpm build` | [2026-05-24-t003-local-data-layer.md](./2026-05-24-t003-local-data-layer.md) | 不提交真实 API Key；本地 SQLite + Store 数据层已建立 |
| T004 | P1 | 实现内置人格和默认人格选择 | 直接在 `dev` 上内置 Prompt 工程师、任务协作者、灵感整理师、正式消息助手，并支持选择默认人格 | Review | dev | 待创建 | `cargo test --test local_data_layer`、`cargo check`、`pnpm build` | [2026-05-24-t004-builtin-persona-default-selection.md](./2026-05-24-t004-builtin-persona-default-selection.md) | 前端依赖目录已恢复；首轮不实现自定义人格编辑 |
| T004A | P1 | 建立 shadcn/ui + Tailwind 前端基础 | 直接在 `dev` 上引入 Tailwind CSS 与 shadcn/ui，配置路径别名、基础设计 token 和首批 UI 组件，并迁移默认人格选择面板 | Review | dev | 待创建 | `pnpm build`、`pnpm exec tsc --noEmit`、`cargo check`、`cargo test --test local_data_layer` | [2026-05-24-t004a-shadcn-tailwind-ui-foundation.md](./2026-05-24-t004a-shadcn-tailwind-ui-foundation.md) | 已建立 UI 基础；HP design-md 不完整注入，仅吸收克制蓝白工具风格 |
| T005 | P1 | 实现热词词典 | 直接在 `dev` 上支持新增、编辑、删除、启用或停用热词，并在文本整理时作为上下文输入 | Review | dev | 待创建 | `cargo test --test local_data_layer`、`cargo check`、`pnpm exec tsc --noEmit`、`pnpm build` | [2026-05-24-t005-hotword-dictionary.md](./2026-05-24-t005-hotword-dictionary.md) | 不做批量导入导出；前端构建和类型检查在沙盒外验证通过 |
| T006 | P0 | 实现智谱 GLM-ASR-2512 Provider | 直接在 `dev` 上支持配置 API Key、Base URL、模型名，输入短音频文件并输出原始识别文本 | Review | dev | [#8](https://github.com/qinyu765/xiluolin/pull/8) | 假配置错误验证；真实本地配置端到端验证 | [2026-05-24-t006-zhipu-asr-provider.md](./2026-05-24-t006-zhipu-asr-provider.md) | ASR 失败不写入历史；真实 API smoke test 延后到本地配置后执行 |
| T007 | P0 | 实现 OpenAI Responses API 文本整理 | 直接在 `dev` 上输入原始识别文本、当前人格 prompt、热词上下文，输出整理后的可用文本 | Review | dev | [#9](https://github.com/qinyu765/xiluolin/pull/9) | 假配置错误验证；真实本地配置端到端验证 | [2026-05-24-t007-openai-responses-text-polish.md](./2026-05-24-t007-openai-responses-text-polish.md) | 整理失败时保留原文 |
| T008 | P0 | 实现主流程 | 直接在 `dev` 上支持短音频录音或上传，串联 ASR、人格化整理、展示结果和复制 | Review | dev | [#10](https://github.com/qinyu765/xiluolin/pull/10) | `cargo test --test voice_input_pipeline`、`cargo check`、`pnpm exec tsc --noEmit`、`pnpm build` | [2026-05-24-t008-main-voice-flow.md](./2026-05-24-t008-main-voice-flow.md) | 当前先实现短音频上传主流程；真实服务端到端验证待本地配置 API Key 和样例音频后执行 |
| T011 | P2 | 修复 Tauri Store 插件启动崩溃 | 修复 `tauri.conf.json` 中 `plugins.store` 的无效配置，确保桌面应用可以正常启动 | Review | dev | 待创建 | `cargo check`、`pnpm build`、`pnpm tauri dev` 启动检查 | [2026-05-24-t011-tauri-store-startup-fix.md](./2026-05-24-t011-tauri-store-startup-fix.md) | 根因是 `tauri-plugin-store` 2.4.x 不接受空对象配置 |
| T009 | P1 | 实现历史记录和统计卡片 | 直接在 `dev` 上保存输入历史并展示协作次数、累计口述时间、生成字数、预计节省时间、常用人格 | Todo | dev | 待创建 | 历史保存和统计更新验证 | 待补充 | 节省时间按每分钟 80 个中文字估算 |
| T010 | P1 | 补齐演示和评审文档 | 直接在 `dev` 上完善 README，新增 demo 脚本或 pitch 草稿，明确第三方服务用途和隐私边界 | Todo | dev | 待创建 | 按 README 从零复现核心流程 | 待补充 | 面向评委演示 |
