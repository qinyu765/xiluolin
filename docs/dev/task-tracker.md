# 开发任务跟踪表

> **归档状态：** 本文件是早期开发阶段的任务状态快照，不再是当前工作的唯一事实源，也不要求 Agent 在开发前读取或在每次改动后更新。当前规则见 [`../../AGENTS.md`](../../AGENTS.md)，当前方向见 [`../roadmap.md`](../roadmap.md)。

## 历史 PR 阶段记录

> 下表记录 2026-07-17 之前采用 `dev -> main` 流程时的阶段划分，仅用于保留项目历史，不再作为当前任务的强制交付规则。

| PR 阶段 | 包含任务 | 提交时机 | PR 目标 |
|---|---|---|---|
| **PR-1: 基础设施和项目初始化** | T001, T002 | T002 完成后 | 建立开发流程、初始化 Tauri + React 项目骨架 |
| **PR-2: 数据层和内置人格** | T003, T004, T004A | T004A 完成后 | 建立本地数据层、内置人格、UI 基础 |
| **PR-3: 热词和服务集成** | T005, T006, T007, T011 | T007 完成后 | 实现热词词典、ASR Provider、文本整理 Provider、修复启动崩溃 |
| **PR-4: 主流程和历史统计** | T008, T009, T012 | T012 完成后 | 实现语音输入主流程、历史记录、统计卡片、录音模块 |
| **PR-5: 全局快捷键和输出模块** | T013, T014 | T014 完成后 | 实现全局快捷键、复制和自动粘贴 |
| **PR-6: 完整前端界面** | T015, T016, T017, T018, T019, T020 | T020 完成后 | 实现主界面、历史页、人格页、热词页、统计页、设置页 |
| **PR-7: 错误处理和端到端验证** | T021, T022 | T022 完成后 | 实现错误处理和兜底、完成端到端验证和调优 |
| **PR-8: 演示和评审文档** | T010 | T010 完成后 | 补齐 README、demo 脚本、pitch 草稿 |

## 末期交付规则（历史）

1. 从最新 `main` 创建 `feat/`、`fix/`、`docs/`、`test/`、`refactor/` 或 `chore/` 短生命周期分支。
2. 每个任务保持范围聚焦，并同步必要的测试、用户文档和技术说明。
3. 提交、推送或创建 PR 前，展示改动文件、验证结果和建议提交信息，并获得用户明确确认。
4. Pull Request 必须以 `main` 为目标分支，说明问题、方案、验证结果、用户影响以及隐私或兼容性风险。
5. 历史任务行中的 `dev` 分支和旧 PR 信息保持不变。

## 状态说明

| 状态 | 含义 |
|---|---|
| Todo | 尚未开始 |
| Doing | 正在执行。任意时间最多只能有一个任务处于此状态 |
| Review | 已完成实现和验证，等待审查、提交、推送或 PR |
| Done | PR 已合入 `main`，或历史任务已按当时流程完成 |
| Blocked | 被明确问题阻塞，备注中必须说明阻塞原因 |

## 任务表

| ID | 优先级 | 任务 | 目标 | 状态 | 分支 | PR | 验证方式 | 开发文档 | 备注 |
|---|---|---|---|---|---|---|---|---|---|
| T001 | P0 | 开发流程基础设施 | 创建执行 skill、任务跟踪表、dev 文档模板，并明确分支、测试、commit、PR、文档规则 | Done | dev | 待创建 | 检查文件存在、skill frontmatter、任务表列和 T001 文档 | [2026-05-24-t001-dev-workflow-infrastructure.md](./2026-05-24-t001-dev-workflow-infrastructure.md) | 当前工作方式已切换为直接在 `dev` 上开发 |
| T002 | P0 | 初始化 Tauri + React + pnpm 项目 | 直接在 `dev` 上初始化 Tauri v2、React、TypeScript、pnpm，并补充 README 运行说明 | Done | dev | 待创建 | `pnpm install`、`pnpm tauri dev` 或对应初始化验证 | [2026-05-24-t002-init-tauri-react-pnpm.md](./2026-05-24-t002-init-tauri-react-pnpm.md) | Rust / C++ Build Tools 已补齐；`pnpm build` 与 `pnpm tauri dev` 已验证通过 |
| T003 | P0 | 建立本地数据层 | 直接在 `dev` 上建立人格、热词、历史记录数据模型，使用 SQLite 保存业务数据，使用 Tauri Store 保存配置 | Done | dev | 待创建 | `cargo test --test local_data_layer`、`cargo check`、`pnpm build` | [2026-05-24-t003-local-data-layer.md](./2026-05-24-t003-local-data-layer.md) | 不提交真实 API Key；本地 SQLite + Store 数据层已建立 |
| T004 | P1 | 实现内置人格和默认人格选择 | 直接在 `dev` 上内置 Prompt 工程师、任务协作者、灵感整理师、正式消息助手，并支持选择默认人格 | Done | dev | 待创建 | `cargo test --test local_data_layer`、`cargo check`、`pnpm build` | [2026-05-24-t004-builtin-persona-default-selection.md](./2026-05-24-t004-builtin-persona-default-selection.md) | 前端依赖目录已恢复；首轮不实现自定义人格编辑 |
| T004A | P1 | 建立 shadcn/ui + Tailwind 前端基础 | 直接在 `dev` 上引入 Tailwind CSS 与 shadcn/ui，配置路径别名、基础设计 token 和首批 UI 组件，并迁移默认人格选择面板 | Done | dev | 待创建 | `pnpm build`、`pnpm exec tsc --noEmit`、`cargo check`、`cargo test --test local_data_layer` | [2026-05-24-t004a-shadcn-tailwind-ui-foundation.md](./2026-05-24-t004a-shadcn-tailwind-ui-foundation.md) | 已建立 UI 基础；HP design-md 不完整注入，仅吸收克制蓝白工具风格 |
| T005 | P1 | 实现热词词典 | 直接在 `dev` 上支持新增、编辑、删除、启用或停用热词，并在文本整理时作为上下文输入 | Done | dev | 待创建 | `cargo test --test local_data_layer`、`cargo check`、`pnpm exec tsc --noEmit`、`pnpm build` | [2026-05-24-t005-hotword-dictionary.md](./2026-05-24-t005-hotword-dictionary.md) | 不做批量导入导出；前端构建和类型检查在沙盒外验证通过 |
| T006 | P0 | 实现智谱 GLM-ASR-2512 Provider | 直接在 `dev` 上支持配置 API Key、Base URL、模型名，输入短音频文件并输出原始识别文本 | Done | dev | [#8](https://github.com/qinyu765/xiluolin/pull/8) | 假配置错误验证；真实本地配置端到端验证 | [2026-05-24-t006-zhipu-asr-provider.md](./2026-05-24-t006-zhipu-asr-provider.md) | ASR 失败不写入历史；真实 API smoke test 延后到本地配置后执行 |
| T007 | P0 | 实现 OpenAI Responses API 文本整理 | 直接在 `dev` 上输入原始识别文本、当前人格 prompt、热词上下文，输出整理后的可用文本 | Done | dev | [#9](https://github.com/qinyu765/xiluolin/pull/9) | 假配置错误验证；真实本地配置端到端验证 | [2026-05-24-t007-openai-responses-text-polish.md](./2026-05-24-t007-openai-responses-text-polish.md) | 整理失败时保留原文 |
| T008 | P0 | 实现主流程 | 直接在 `dev` 上支持短音频录音或上传，串联 ASR、人格化整理、展示结果和复制 | Done | dev | [#10](https://github.com/qinyu765/xiluolin/pull/10) | `cargo test --test voice_input_pipeline`、`cargo check`、`pnpm exec tsc --noEmit`、`pnpm build` | [2026-05-24-t008-main-voice-flow.md](./2026-05-24-t008-main-voice-flow.md) | 当前先实现短音频上传主流程；真实服务端到端验证待本地配置 API Key 和样例音频后执行 |
| T011 | P2 | 修复 Tauri Store 插件启动崩溃 | 修复 `tauri.conf.json` 中 `plugins.store` 的无效配置，确保桌面应用可以正常启动 | Done | dev | [#11](https://github.com/qinyu765/xiluolin/pull/11) | `cargo check`、`pnpm build`、`pnpm tauri dev` 启动检查 | [2026-05-24-t011-tauri-store-startup-fix.md](./2026-05-24-t011-tauri-store-startup-fix.md) | 根因是 `tauri-plugin-store` 2.4.x 不接受空对象配置 |
| T009 | P1 | 实现历史记录和统计卡片 | 直接在 `dev` 上保存输入历史并展示协作次数、累计口述时间、生成字数、预计节省时间、常用人格 | Done | dev | 待创建 | `cargo test --test local_data_layer`、`cargo check`、`pnpm exec tsc --noEmit`、`pnpm build` | [2026-05-24-t009-history-statistics.md](./2026-05-24-t009-history-statistics.md) | 节省时间按每分钟 80 个中文字估算；前端展示最近 10 条历史 |
| T012 | P0 | 实现录音模块 | 直接在 `dev` 上实现麦克风录音开始、停止、音频文件生成、录音时长统计和状态通知 | Done | dev | 待创建 | `cargo check`、`pnpm build`、手动录音测试生成音频文件 | [2026-05-25-t012-recording-module.md](./2026-05-25-t012-recording-module.md) | 支持短音频输入；检查麦克风权限；通知前端录音中、处理中、失败状态 |
| T013 | P0 | 实现全局快捷键模块 | 直接在 `dev` 上注册全局快捷键，支持长按录音和切换式录音两种模式 | Done | dev | 待创建 | `cargo check`、`pnpm build`、手动测试快捷键触发录音 | [2026-05-25-t013-global-hotkey.md](./2026-05-25-t013-global-hotkey.md) | Rust 侧已 emit 录音事件；前端监听和处理链路仍待接线 |
| T014 | P1 | 实现输出模块（复制和自动粘贴） | 直接在 `dev` 上实现文本复制到剪贴板和模拟粘贴功能，自动粘贴失败时保留复制兜底 | Done | dev | 待创建 | `cargo check`、`pnpm build`、手动测试复制和粘贴到不同应用 | [2026-05-25-t014-output-module.md](./2026-05-25-t014-output-module.md) | 采用"直接键盘注入+剪贴板兜底"架构；编译验证通过；真实场景测试待 T015 完成后执行 |
| T015 | P0 | 实现主界面 | 直接在 `dev` 上实现主界面，包含当前人格选择、录音按钮、录音状态、原始识别文本、整理后文本、复制按钮、输出按钮 | Done | dev | 待创建 | `pnpm exec tsc --noEmit`、`pnpm build`、`pnpm tauri dev` 手动测试主流程 | [2026-05-25-t015-main-ui.md](./2026-05-25-t015-main-ui.md) | `QuickStartCard` 已实现但当前隐藏；首页可见语音输入入口待恢复或替代 |
| T016 | P1 | 实现历史页 | 直接在 `dev` 上为历史记录添加复制功能，每条历史记录右上角显示复制按钮 | Done | dev | 待创建 | `pnpm exec tsc --noEmit`、`pnpm build`、`pnpm tauri dev` 手动测试复制功能 | [2026-05-25-t016-history-page.md](./2026-05-25-t016-history-page.md) | 已完成编译验证；复用 T009 已有的历史记录展示；真实复制测试待手动验证 |
| T017 | P1 | 实现人格页 | 直接在 `dev` 上实现人格页，展示内置人格列表、自定义人格列表、新建人格、编辑自定义人格、设置默认人格 | Done | dev | 待创建 | `pnpm exec tsc --noEmit`、`pnpm build`、`pnpm tauri dev` 手动测试人格管理 | [2026-05-25-t017-persona-page.md](./2026-05-25-t017-persona-page.md) | 已完成编译验证；支持用户自定义人格配置；真实场景测试待手动验证 |
| T018 | P1 | 实现热词页 | 直接在 `dev` 上实现热词页，展示热词列表、新增热词、编辑热词、启用或停用热词 | Done | dev | 待创建 | `pnpm exec tsc --noEmit`、`pnpm build`、`pnpm tauri dev` 手动测试热词管理 | [2026-05-25-t018-hotword-page.md](./2026-05-25-t018-hotword-page.md) | 热词管理界面已在 T005 中完整实现；编译验证通过；手动测试待执行 |
| T019 | P1 | 实现统计页或首页卡片 | 直接在 `dev` 上实现统计页或首页卡片，展示语音协作次数、累计口述时间、口述生成字数、预计节省时间、常用人格 | Done | dev | 待创建 | `pnpm exec tsc --noEmit`、`pnpm build`、`pnpm tauri dev` 手动测试统计卡片 | [2026-05-25-t019-statistics-page.md](./2026-05-25-t019-statistics-page.md) | 统计卡片已在 T009 中完整实现；编译验证通过；手动测试待执行 |
| T020 | P0 | 实现设置页 | 直接在 `dev` 上实现设置页，包含智谱 GLM-ASR-2512 配置、OpenAI 文本模型配置、快捷键配置、录音模式配置、输出方式配置 | Done | dev | 待创建 | `pnpm exec tsc --noEmit`、`pnpm build`、`pnpm tauri dev` 手动测试设置保存和读取 | [2026-05-25-t020-settings-page.md](./2026-05-25-t020-settings-page.md) | 新增应用设置卡片；ASR 和 OpenAI 配置已在 T006/T007 实现；编译验证通过 |
| T021 | P0 | 实现错误处理和兜底 | 直接在 `dev` 上覆盖未配置 API Key、麦克风权限缺失、录音失败、ASR 调用失败、快速文本模型调用失败、自动粘贴失败、数据库写入失败等场景 | Done | dev | 待创建 | `cargo check`、`pnpm build`、手动测试各类失败场景 | [2026-05-25-t021-error-handling.md](./2026-05-25-t021-error-handling.md) | 已完成编译验证；真实场景测试待手动执行 |
| T022 | P0 | 端到端验证和调优 | 直接在 `dev` 上完成首次启动配置、短语音输入、人格切换、热词生效、历史保存、统计更新的完整闭环验证 | Done | dev | 待创建 | 按 solution-design.md 第 11 节验收路径逐项验证 | [2026-05-25-t022-e2e-verification.md](./2026-05-25-t022-e2e-verification.md) | 确保 MVP 可演示、可解释、可复现 |
| T010 | P1 | 补齐演示和评审文档 | 直接在 `dev` 上完善 README，新增 demo 脚本或 pitch 草稿，明确第三方服务用途和隐私边界 | Done | dev | 待创建 | 文档一致性检查；按 README 复核复现路径 | [2026-05-25-t010-demo-docs-sync.md](./2026-05-25-t010-demo-docs-sync.md) | 已同步当前 MVP 和 UI 重构状态；首页语音入口隐藏、快捷键事件未接线、录音指示器待验证已记录 |
| T023 | P0 | 实现导航框架 | 直接在 `dev` 上建立左侧 Tab 导航栏和页面路由系统，为 UI 重构奠定基础 | Done | dev | 待创建 | `pnpm exec tsc --noEmit`、`pnpm build`、`cargo check` | [2026-05-25-t023-navigation-framework.md](./2026-05-25-t023-navigation-framework.md) | 采用状态管理实现路由；当前设置入口统一在左侧导航 |
| T024 | P0 | 首页重构 | 直接在 `dev` 上重构首页内容，移除技术标记，添加时间分段历史记录展示和删除功能 | Done | dev | 待创建 | `pnpm exec tsc --noEmit`、`pnpm build`、`cargo check` | [2026-05-25-t024-home-page-refactor.md](./2026-05-25-t024-home-page-refactor.md) | 历史记录按今天/昨天/具体日期分组；新增删除历史记录命令 |
| T025 | P1 | 人格页整合 | 直接在 `dev` 上整合人格管理和默认人格选择功能到人格页 | Done | dev | 待创建 | `pnpm exec tsc --noEmit`、`pnpm build` | [2026-05-25-t025-persona-page-integration.md](./2026-05-25-t025-persona-page-integration.md) | 合并 T017 人格管理卡片到人格页 |
| T026 | P1 | 热词页优化 | 直接在 `dev` 上优化热词页说明文案，明确热词功能定义 | Done | dev | 待创建 | `pnpm exec tsc --noEmit`、`pnpm build` | [2026-05-25-t026-hotword-page-optimization.md](./2026-05-25-t026-hotword-page-optimization.md) | 优化字段说明和示例 |
| T027 | P1 | 设置页重构 | 直接在 `dev` 上重构设置页，添加分类 Tab，移动 ASR 和 OpenAI 配置到设置页 | Done | dev | 待创建 | `pnpm exec tsc --noEmit`、`pnpm build` | [2026-05-25-t027-settings-page-refactor.md](./2026-05-25-t027-settings-page-refactor.md) | 当前实际实现为顶部水平 Tab：通用、模型配置 |
| T028 | P0 | UI 重构端到端验证 | 直接在 `dev` 上验证 UI 重构后的完整流程，确保所有功能正常 | Done | dev | 待创建 | 按 T028 文档验收清单逐项验证 | [2026-05-25-t028-ui-e2e-verification.md](./2026-05-25-t028-ui-e2e-verification.md) | 用户确认所有验证项已完成 |
| T028A | P0 | 切换 main 直接开发流程 | 将 `dev` 未合入改动通过 PR #21 rebase 合入 `main`，并把当前协作规范统一调整为直接基于 `main` 开发 | Done | main | 不适用（main 直提） | 文档引用检查、规则一致性检查、`git diff --check` | [2026-07-17-t028a-main-development-workflow.md](./2026-07-17-t028a-main-development-workflow.md) | PR #21 已合入；工作流文档已推送到 `main` |
| T029 | P0 | 建立前后端质量检查基线 | 修复现有 TypeScript 与 Rust 编译阻塞，建立统一检查脚本和 Windows CI | Done | main | 不适用（main 直提） | `pnpm install --frozen-lockfile`、`pnpm check`、Windows CI | [2026-07-17-t029-quality-security-baseline.md](./2026-07-17-t029-quality-security-baseline.md) | 本地完整检查和 Windows CI 均通过 |
| T030 | P0 | 加固本地凭据和隐私数据处理 | 将 API Key 迁移到系统凭据库，清理敏感日志，并限制和清理应用录音临时文件 | Done | main | 不适用（main 直提） | `pnpm check`、凭据迁移测试、录音路径与清理测试、Windows CI | [2026-07-17-t030-credential-privacy-hardening.md](./2026-07-17-t030-credential-privacy-hardening.md) | 本地完整检查和 Windows CI 均通过 |
| T031 | P0 | 建立跨应用 CaptureSession 和可靠文本投递 | 为快捷键录音建立会话状态、目标窗口快照、剪贴板恢复和多状态指示器 | Review | feat/capture-session-delivery | 待创建 | `pnpm check`、CaptureSession 单元测试、Windows CI、Windows 手动跨应用验证 | [2026-07-17-t031-capture-session-delivery.md](./2026-07-17-t031-capture-session-delivery.md) | 本地完整检查通过；等待 Windows CI 和桌面端手动验证 |
| T032 | P0 | 增加语音输入就绪检查 | 检查麦克风、Provider 配置、快捷键注册和自动粘贴能力，并在设置页展示可操作提示 | Review | feat/input-readiness | 待创建 | `pnpm check`、就绪计算单元测试、Windows CI、设置页手动检查 | [2026-07-17-t032-input-readiness.md](./2026-07-17-t032-input-readiness.md) | 本地检查通过；等待 Windows CI 和设置页真实环境验证 |
| T033 | P1 | 增加 Capture 历史和可选录音保留 | 扩展历史快照、默认删除录音，并为显式保留的录音提供试听、重处理基础和清理能力 | Review | feat/capture-retention | 待创建 | `pnpm check`、数据库迁移测试、录音留存与清理测试、Windows CI | [2026-07-17-t033-capture-retention.md](./2026-07-17-t033-capture-retention.md) | 本地检查通过；等待 Windows CI 和真实录音留存验证 |
| T034 | P1 | 增加 Capture 试听和重新处理 | 为已保留录音提供试听、重新转写，并允许历史原文重新执行人格整理 | Review | feat/capture-reprocessing | 待创建 | `pnpm check`、重新处理测试、Windows CI、桌面端手动验证 | [2026-07-17-t034-capture-reprocessing.md](./2026-07-17-t034-capture-reprocessing.md) | 本地检查通过；等待 Windows CI 和真实 Provider 验证 |
| T035 | P1 | 接入本地 ASR 与显式云端降级 | 使用 whisper-rs 提供离线转写、模型管理和可选云端回退，并记录实际 Provider | Review | feat/local-asr | 待创建 | `pnpm check`、本地 ASR 单元测试、Windows CI、模型下载与离线手动验证 | [2026-07-17-t035-local-asr.md](./2026-07-17-t035-local-asr.md) | 本地检查和真实离线 smoke test 通过；等待 Windows CI |
