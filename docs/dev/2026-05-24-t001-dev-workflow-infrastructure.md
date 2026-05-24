# T001 开发流程基础设施

## 任务目标

创建项目级执行 skill、任务跟踪表和开发文档模板，让后续每个开发任务都能按固定流程选择、验证、记录和准备 PR。

## 实际改动

- 新增 `.agents/skills/next-dev-task/SKILL.md`，定义后续执行任务时需要读取任务表、检查分支、执行验证、更新文档和准备 commit/PR 信息。
- 新增 `docs/dev/task-tracker.md`，记录 T001 到 T010 的任务状态、分支、验证方式和开发文档。
- 新增 `docs/dev/task-doc-template.md`，固定每个任务完成后的开发记录结构。
- 新增本任务开发文档，记录 T001 的目标、改动、验证和后续建议。

## 为什么这么做

当前仓库已有需求分析和方案设计，但还没有可持续维护的开发执行入口。将执行规则封装为项目级 skill，可以让后续开发者或 agent 在“继续下一个任务”时先读取任务表，并按同一套检查清单完成测试、文档、commit 和 PR 准备。

任务表作为唯一事实源，可以避免任务状态只存在对话上下文里。每个任务单独生成开发文档，可以直接支撑 PR 描述、评审说明和黑客松过程展示。

## 涉及文件

- `.agents/skills/next-dev-task/SKILL.md`
- `docs/dev/task-tracker.md`
- `docs/dev/task-doc-template.md`
- `docs/dev/2026-05-24-t001-dev-workflow-infrastructure.md`

## 测试与验证

验证方式：

- 检查 `next-dev-task` skill 是否包含合法 frontmatter：`name` 和 `description`。
- 检查任务跟踪表是否包含固定列：`ID`、`优先级`、`任务`、`目标`、`状态`、`分支`、`PR`、`验证方式`、`开发文档`、`备注`。
- 检查任务表中最多只有一个 `Doing` 状态任务。
- 检查 T001 开发文档是否包含必需章节。

实际结果：验证通过。skill frontmatter 合法，任务表表头完整且 `Doing` 任务数量为 0，T001 开发文档包含 7 个二级标题。

## 未完成事项

- 尚未创建 PR。
- 尚未将 T001 标记为 `Done`，因为该状态应在 PR 合入或用户明确确认后更新。

## 后续建议

后续执行 `T002` 时，先调用 `next-dev-task`，从 `docs/dev/task-tracker.md` 读取下一个 `Todo` 任务，并直接在 `dev` 上初始化 Tauri + React + pnpm 项目。阶段性成果准备好后，再从 `dev` 发起合并到 `main` 的 PR。
