# XiLuoLin

**简体中文** | [English](README.en.md)

XiLuoLin 是一个面向办公、写作和编程场景的开源 AI 语音输入助手。它将短语音转换为可直接使用的文本，并通过人格化整理、热词、历史记录和桌面输出减少打字、编辑与润色成本。

XiLuoLin 由个人发起并持续维护，欢迎社区通过 Issue、Discussion 和 Pull Request 参与。

## 产品方向

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

- [文档导航](docs/README.md)
- [需求分析](docs/requirements-analysis.md)
- [方案设计](docs/solution-design.md)
- [使用与验证指南](docs/usage-guide.md)
- [故障排查](docs/troubleshooting.md)
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

- `main` 是稳定开发基线；外部贡献通过短生命周期分支和 Pull Request 合入，仓库维护者的 Agent 工作流以 `AGENTS.md` 为准。
- 路线图用于表达方向，不承诺固定交付日期。
- 维护者会根据用户价值、可靠性、隐私风险、维护成本和架构一致性评估提案。
- 历史开发记录保留在 `docs/dev/`，其中的比赛、demo 或 MVP 表述仅代表当时背景，不再定义当前项目方向。

## 许可证

本项目基于 [MIT License](LICENSE) 开源。
