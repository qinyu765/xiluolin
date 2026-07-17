---
name: next-dev-task
description: Use only when the user explicitly asks to choose, continue, organize, or close the next XiLuoLin development task.
---

# Next Dev Task

## Purpose

帮助用户从当前仓库状态和现行文档中确定下一项工作，并以可验证的小范围改动完成它。本 skill 不是所有开发任务的默认入口。

## Workflow

1. 读取 `AGENTS.md`，检查当前分支、工作区和最近提交。
2. 根据用户请求选择必要的现行文档；文档入口见 `docs/README.md`。
3. 若用户指定了任务，直接围绕该任务工作；未指定时，从当前代码、路线图、Issue 或用户给出的优先级中提出下一项建议。
4. 明确目标、范围和验证方式，避免同时启动无关工作。
5. 实现最小完整改动，并运行与改动范围匹配的检查。
6. 只有用户明确要求时，才更新历史 task tracker、创建任务复盘、使用分支或创建 PR。
7. 按 `AGENTS.md` 的 Git 规则发布并汇报结果。

## Selection Guidelines

- 优先处理会阻断核心语音输入链路、数据安全或正常构建的问题。
- 优先修复已有行为，再添加新功能；避免基于旧任务表机械选择已经过时的事项。
- 需求、架构和实现不一致时，以当前代码和现行文档为依据，并明确指出差异。
- 较大功能、数据迁移或依赖替换应先拆分为可独立验证的步骤。

## Verification

- 前端改动：按范围运行 `pnpm typecheck`、`pnpm build` 或相关测试。
- Rust 改动：按范围运行格式、编译和测试检查。
- 文档改动：检查链接、引用和 `git diff --check`。
- 外部 API：优先使用 mock 或本地验证，不提交真实凭据和用户数据。
- 无法执行检查时，记录具体命令、失败原因和未验证风险。

## Completion

完成时说明：

- 实际完成的目标和主要改动。
- 已运行的验证及结果。
- 未完成事项或已知风险。
- Commit 和 push 状态；只有用户要求时才补充 PR 信息或任务复盘。
