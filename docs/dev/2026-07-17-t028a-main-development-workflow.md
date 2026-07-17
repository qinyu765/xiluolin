# T028A 切换 main 直接开发流程

## 任务目标

将仓库的日常开发基线从 `dev` 调整为 `main`。在修改协作规范前，先把 `dev` 上尚未合入的文本处理指令改动通过 PR 同步到 `main`，确保切换后不会遗漏代码。

## 实际改动

- 创建 PR #21，将 `dev` 上的 `6011166` 通过 rebase merge 合入 `main`。
- 将 `AGENTS.md` 和 `CLAUDE.md` 的分支规则改为常规任务直接在 `main` 开发。
- 同步两份 `next-dev-task` skill，改为提交前同步 `origin/main`，验证后直接提交并推送 `main`。
- 更新 README 的开发基线说明。
- 更新任务跟踪表顶部的当前工作方式，并保留历史任务的原始分支记录。

## 为什么这么做

项目后续不再使用持久 `dev` 分支承载日常开发。直接以 `main` 为基线可以减少长期分支同步和重复合并成本。为降低直接开发的风险，每次改动仍必须保持小而完整，执行验证，并在提交和推送前获得用户明确确认。

PR 不再是常规任务的强制步骤，仅在用户明确要求、外部贡献或高风险改动需要隔离审查时使用。

## 涉及文件

- `AGENTS.md`
- `CLAUDE.md`
- `.agents/skills/next-dev-task/SKILL.md`
- `.claude/skills/next-dev-task/SKILL.md`
- `README.md`
- `docs/dev/task-tracker.md`
- `docs/dev/2026-07-17-t029-main-development-workflow.md`

## 测试与验证

- PR #21 已于 2026-07-17 通过 rebase merge 合入 `main`。
- `pnpm build` 在合入前验证通过。
- Rust 测试和 `cargo check` 被既有问题阻断：缺少 `src-tauri/icons/icon.png`，且 `WebviewWindowBuilder::transparent` 在当前 macOS 构建目标不可用。
- `cargo fmt --check` 发现仓库历史 Rust 文件存在大量格式差异，本任务不执行全仓格式化。
- 文档变更使用引用搜索和 `git diff --check` 验证，不修改历史任务事实记录。

## 执行复盘

### 遇到的问题

1. 当前 `dev` 比 `main` 多一条尚未合入的提交，需要在工作流切换前先同步。
2. Rust 首次验证暴露了图标资源缺失和 macOS 窗口 API 兼容问题，导致测试目标无法完成编译。
3. 旧规则同时存在于根目录协作规范和两份工具 skill 中，必须同步修改，避免不同 Agent 读取到冲突规则。

### 解决流程

1. 检查远端分支和 PR 状态，确认差异只有 `6011166`。
2. 创建 PR #21，并按用户指定使用 rebase merge 合入 `main`。
3. 更新本地 `main` 后修改所有当前有效的工作流文档。
4. 保留历史任务文档和历史任务表行中的 `dev` 信息，只更新当前规则入口。

### 经验总结

- 调整分支策略时，应先同步旧开发分支上的未合入改动。
- 工作流规则需要同时更新人类文档和 Agent skill，避免自动化执行继续采用旧流程。
- 直接在 `main` 开发时，更需要小提交、提交前验证和明确的用户确认。

## 未完成事项

- 远端 `dev` 分支暂时保留，未执行删除。
- Rust 图标资源和 macOS 透明窗口兼容问题不属于本任务，后续应单独修复。

## 后续建议

下一项开发任务开始前先执行 `git switch main` 和 `git pull --ff-only origin main`。后续常规任务直接在 `main` 完成，提交和推送前继续展示改动与验证结果并获得用户确认。
