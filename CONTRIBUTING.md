# 参与 XiLuoLin 开发

感谢你愿意参与 XiLuoLin。项目欢迎代码、测试、文档、设计、翻译、问题复现和产品讨论等多种形式的贡献。

## 开始之前

- 阅读中文 [README.md](README.md)、英文 [README.en.md](README.en.md)、[路线图](docs/roadmap.md) 和相关技术文档。
- 搜索现有 Issue，避免重复提交。
- 对较大功能、依赖替换、数据迁移或架构调整，先创建 Issue 说明使用场景和方案。
- 不要在公开 Issue、PR、日志或截图中提交 API Key、私人录音、用户文本或其他敏感数据。

## 报告问题

一个可处理的 Bug 报告应包含：

- 操作系统、版本和 CPU 架构
- XiLuoLin 版本或 commit
- 可复现步骤
- 预期行为与实际行为
- 已运行的检查命令
- 脱敏后的日志、截图或最小复现

安全漏洞请遵循 [SECURITY.md](SECURITY.md)，不要创建公开漏洞详情。

## 开发流程

1. Fork 并克隆仓库，或在有写入权限时直接克隆。
2. 从最新 `main` 创建短生命周期分支：

   ```bash
   git switch main
   git pull --ff-only origin main
   git switch -c feat/简短主题
   ```

3. 保持改动聚焦，不混入无关重构或格式化。
4. 添加或更新必要的测试和文档。
5. 运行检查：

   ```bash
   pnpm install --frozen-lockfile
   pnpm check
   ```

6. 涉及桌面能力时，运行 `pnpm tauri dev` 并记录手动验证环境与步骤。
7. 提交 Pull Request 到 `main`。

推荐分支前缀：`feat/`、`fix/`、`docs/`、`test/`、`refactor/`、`chore/`。

## Commit 规范

Commit 使用 `type: 中文描述` 格式，例如：

- `feat: 新增自定义 ASR Provider 配置`
- `fix: 修复快捷键停止录音后状态未重置`
- `docs: 补充 macOS 权限配置说明`
- `test: 增加历史记录迁移测试`

一个 commit 应表达一个连贯改动，不要提交构建产物、真实密钥、临时录音或本地缓存。

## Pull Request 要求

PR 描述至少应包含：

- 问题或使用场景
- 解决方案与关键取舍
- 用户可见变化
- 自动化检查结果
- 手动验证步骤与环境
- 隐私、兼容性或数据迁移影响
- 关联 Issue

UI 改动请附截图或录屏。尚未完成的工作请使用 Draft PR。维护者可能要求缩小范围、补充测试、调整设计或拆分后再合入。

## 代码与文档原则

- 优先保证正确性、可读性、隐私和可维护性。
- 延续现有模块边界和代码风格，避免无需求依据的抽象。
- 新依赖必须说明用途、替代方案和维护成本。
- Provider 相关改动应避免将业务逻辑绑定到单一服务商。
- 用户可见行为、配置方式和已知限制发生变化时，必须同步文档。
- 项目定位、功能状态、安装使用、隐私、兼容性或贡献方式发生变化时，必须在同一个 PR 中同步更新 `README.md` 和 `README.en.md`。
- 历史记录可以保留原始背景，但当前文档不得再把项目描述为比赛项目、一次性 demo 或最小 MVP。

## 行为准则

参与项目即表示你同意遵守 [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)。
