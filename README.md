# XiLuoLin

**简体中文** | [English](README.en.md)

XiLuoLin 是一个面向办公、写作和编程场景的开源 AI 语音输入助手。它将短语音转换为可直接使用的文本，并通过人格化整理、热词、历史记录和桌面输出减少打字、编辑与润色成本。

- **语音输入能力**：支持全局快捷键和应用内录音流程；快捷键开始时建立 CaptureSession，处理完成后向录音开始时的目标窗口投递文本
- **智能识别**：已接入智谱 GLM-ASR-2512 Provider，用于将短音频转为文本
- **人格化整理**：已接入 OpenAI Responses API，根据选定人格（Prompt 工程师、任务协作者、灵感整理师、正式消息助手）整理文本风格和结构
- **热词词典**：支持自定义专有名词、技术词、项目名，减少误识别
- **Capture 历史**：保存原始文本、整理结果、人格、输入来源、实际 Provider/模型、降级和投递方式；保留录音可试听、重新转写，原始文本可用当前人格重新整理
- **统计反馈**：展示语音协作次数、累计口述时间、生成字数、预计节省时间、常用人格
- **输出方式**：Windows 侧保存并恢复录音开始时的目标窗口；自动粘贴成功后恢复原文本或图片剪贴板，失败时保留生成文本供手动粘贴
- **就绪检查**：设置页统一展示麦克风、ASR、文本 Provider、全局快捷键和自动粘贴能力；自动粘贴不可用不阻断识别、历史保存和复制兜底

XiLuoLin 由个人发起并持续维护，欢迎社区通过 Issue、Discussion 和 Pull Request 参与。

## 产品方向

- 当前阶段：MVP 核心模块、质量门禁、凭据安全、可靠投递、输入就绪检查和可追溯 Capture 历史已实现，继续进行 Windows 桌面端真实场景验证
- 开发基线：`main`（常规任务直接在 `main` 上完成验证、提交和推送）
- 已完成代码层能力：Tauri v2 + React 基础骨架、本地数据层、内置人格与自定义人格、热词词典、智谱 ASR Provider、OpenAI 文本整理 Provider、短音频处理流程、历史记录、统计卡片、录音模块、全局快捷键注册、复制与自动粘贴模块、错误提示、左侧导航、人格管理页、热词页和设置页、快捷键录音事件监听和自动输出
- 当前界面：采用左侧导航结构，包含首页、人格、热词、设置四个页面；首页当前聚焦当前人格问候、快捷键提示、统计卡片和时间分段历史记录
- 当前限制：首页的 `QuickStartCard` 录音 / 上传入口在代码中被注释隐藏；应用设置统一由设置页承载
- 待完成验证：真实 API Key 下的完整语音 smoke test、Windows 普通与提升权限窗口的自动粘贴、目标窗口关闭后的降级、首页可见输入入口
- 前端 UI 方向：采用 Tailwind CSS + shadcn/ui，参考 Notion 风格的桌面效率工具界面
XiLuoLin 关注“从说出来到真正可用”的完整输入体验：

- **语音采集**：麦克风录音、短音频处理、全局快捷键和录音状态提示。
- **语音识别**：通过可配置 ASR Provider 将音频转换为原始文本。
- **人格化整理**：根据 Prompt 工程师、任务协作者、灵感整理师、正式消息助手或自定义人格整理结构与语气。
- **热词词典**：维护项目名、人名和技术词，降低专有名词误识别带来的编辑成本。
- **桌面输出**：支持剪贴板和自动粘贴等输出方式，并在能力受限时提供降级路径。
- **本地数据**：保存历史、人格、热词、设置和个人效率统计。
- **开放扩展**：保持 Provider 和业务模块边界，为更多云服务、本地模型与跨平台适配留出清晰接口。

## 当前状态

项目处于持续开发阶段，核心模块已经具备，但仍需围绕可靠性、跨平台验证、易用性和发布流程继续完善。

已实现的主要能力：

- Tauri v2 + React + TypeScript 桌面应用骨架
- SQLite 本地数据层与系统凭据库存储
- 内置人格、自定义人格和默认人格
- 热词词典、历史记录和统计卡片
- 智谱 GLM-ASR-2512 Provider
- OpenAI Responses API 文本整理 Provider
- 录音、全局快捷键、录音指示器和短音频处理流程
- 剪贴板、自动粘贴及错误提示
- 首页、人格、热词和设置页面
- TypeScript、前端构建、Rust 格式、编译和测试质量检查

当前重点：

- 验证不同操作系统上的麦克风、快捷键、凭据库和跨应用输出行为
- 完善首页语音入口与录音状态体验
- 建立稳定的版本发布、安装包和兼容性说明
- 增强 Provider 可配置性、失败恢复和自动化测试
- 持续改善贡献者文档、Issue 管理和技术决策记录

更完整的产品边界和技术设计见：

- [需求分析](docs/requirements-analysis.md)
- [方案设计](docs/solution-design.md)
- [使用与验证指南](docs/usage-guide.md)
- [项目路线图](docs/roadmap.md)

## 技术栈

- 桌面框架：Tauri v2
- 前端：React 19、TypeScript、Vite
- UI：Tailwind CSS、shadcn/ui、Radix UI
- 本地存储：SQLite、Tauri Store、系统凭据库
- 音频：cpal、hound
- 外部服务：可配置 ASR 与文本处理 Provider

## 环境要求

- Node.js 20+
- pnpm 10+
- Rust stable 工具链
- Windows：Microsoft Visual Studio C++ Build Tools、WebView2 Runtime
- macOS / Windows：麦克风权限；自动输入可能还需要辅助功能或输入监控权限

## 本地开发

```bash
git clone https://github.com/qinyu765/xiluolin.git
cd xiluolin
pnpm install --frozen-lockfile
pnpm check
pnpm tauri dev
```

常用命令：

| 命令 | 作用 |
|---|---|
| `pnpm dev` | 启动前端开发服务 |
| `pnpm typecheck` | 执行 TypeScript 类型检查 |
| `pnpm build` | 类型检查并构建前端 |
| `pnpm check:rust` | 执行 Rust 格式、编译和测试检查 |
| `pnpm check` | 执行完整前端与 Rust 质量检查 |
| `pnpm tauri dev` | 启动桌面应用开发模式 |

GitHub Actions 会在 `main` push 和面向 `main` 的 Pull Request 上运行质量检查。涉及录音、快捷键、凭据或输出能力的变更仍需在桌面环境中手动验证。

## 配置与使用

1. 启动应用并进入“设置”。
2. 配置智谱 GLM-ASR-2512 或其他受支持 ASR 服务。
3. 配置 OpenAI Responses API 或兼容的文本处理服务。
4. 选择麦克风、快捷键和输出方式。
5. 选择一个内置人格，或创建自定义人格。
6. 添加需要重点识别的项目名、人名和技术词。
7. 在目标输入框中使用全局快捷键完成语音输入。

真实服务演示仍需在本机配置 API Key 和麦克风权限后执行 smoke test；首页可见录音 / 上传入口当前隐藏，全局快捷键是主要输入入口。快捷键触发时状态窗会依次显示录音、识别、整理、输入和完成状态，且不会主动获取键盘焦点。
详细步骤、验证路径和错误场景见 [使用与验证指南](docs/usage-guide.md)。

## 隐私与安全

- 音频只发送给用户主动配置的 ASR Provider。
- 原始识别文本只发送给用户主动配置的文本处理 Provider。
- API Key 保存在 Windows Credential Manager、macOS Keychain 或系统原生凭据库中。
- 历史记录、人格、热词和统计数据默认保存在本地 SQLite，不上传到项目服务器。
- 应用生成的临时录音会在处理成功或失败后清理；用户选择的外部音频不会被清理逻辑删除。
- 日志不应记录 API Key、用户完整文本或完整录音路径。

使用第三方 Provider 前，请自行阅读其隐私政策、数据保留规则和服务条款。安全问题请按照 [SECURITY.md](SECURITY.md) 报告。

## 参与贡献

欢迎以下类型的贡献：

- Bug 报告、复现案例和跨平台兼容性反馈
- 产品建议、交互改进和可访问性优化
- Provider、录音、快捷键、输出与本地存储能力增强
- 测试、文档、翻译和发布流程改进

开始前请阅读 [CONTRIBUTING.md](CONTRIBUTING.md) 和 [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)。建议先通过 Issue 对齐较大的功能或架构改动，再从短生命周期分支向 `main` 提交 Pull Request。

当项目定位、功能状态、安装使用、隐私、兼容性或贡献方式发生变化时，请在同一个 Pull Request 中同步更新中文 [README.md](README.md) 和英文 [README.en.md](README.en.md)。

## 项目治理

- `main` 是稳定开发基线，所有变更通过分支和 Pull Request 合入。
- 路线图用于表达方向，不承诺固定交付日期。
- 维护者会根据用户价值、可靠性、隐私风险、维护成本和架构一致性评估提案。
- 历史开发记录保留在 `docs/dev/`，其中的比赛、demo 或 MVP 表述仅代表当时背景，不再定义当前项目方向。

## 许可证

- 音频发送给用户配置的智谱 GLM-ASR-2512 服务
- 原始识别文本发送给用户配置的文本处理 Provider
- API Key 保存在 Windows Credential Manager 或 macOS Keychain；`settings.json` 只保存非敏感配置
- 旧版 `settings.json` 中的明文 API Key 会在首次成功写入系统凭据库后自动清理
- 应用生成的录音默认在处理后删除；只有用户开启“保留原始录音”、自动历史开启且历史写入成功时才保留
- 用户自行选择的外部音频不会被录音清理逻辑删除；删除历史或执行“清理全部录音”会同步解除应用录音关联
- 日志不记录 API Key 片段、用户文本或完整录音路径
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
  - `tauri-plugin-store`：Rust 侧保存默认人格、快捷键、Provider 地址等非敏感轻量配置。
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
  - `keyring`：通过 Windows Credential Manager、macOS Keychain 或系统原生安全存储保存 API Key。
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

- `public/indicator.html`：录音状态窗口资源，由 Vite 在开发和生产构建中统一提供；窗口在应用启动时预创建并保持不可聚焦。
本项目基于 [MIT License](LICENSE) 开源。
