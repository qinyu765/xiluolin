---
name: next-dev-task
description: Use when continuing this repository's development work, choosing the next task, closing a task, preparing a commit, or preparing a pull request.
---

# Next Dev Task

## Purpose

This skill keeps development task execution auditable for the Qiniu Hackathon voice input assistant.

`docs/dev/task-tracker.md` is the single source of truth for task state. Read it before choosing work, and update it before claiming a task is ready for review.

## Required Workflow

1. Read `AGENTS.md`, `docs/requirements-analysis.md`, `docs/solution-design.md`, and `docs/dev/task-tracker.md`.
2. Confirm the current branch. Work on `dev`; do not create routine feature branches.
3. Select the next task:
   - If one task is `Doing`, continue it.
   - Otherwise choose the first `Todo` task by tracker order.
   - Do not start a second `Doing` task.
4. Restate the task goal, success criteria, working branch, and verification command or manual check.
5. Implement only the selected task. Keep changes surgical and aligned with the voice input assistant scope.
6. Run the relevant verification:
   - Automated tests or build when available.
   - Manual verification when no automated check exists.
   - If verification cannot run, document the exact blocker.
7. Create or update one task document under `docs/dev/YYYY-MM-DD-<task-id>-<title>.md`.
8. Update `docs/dev/task-tracker.md`:
   - `Doing` when starting.
   - `Review` when implementation and verification are ready for review.
   - `Done` only after the PR is merged or the user explicitly confirms completion.
   - `Blocked` with a concrete blocker when work cannot continue.
9. Before any commit or PR, stop and ask for explicit user approval. Show:
   - Changed files.
   - Verification result.
   - Suggested commit message using `type: 中文描述`.
   - PR description containing feature description, implementation approach, and verification method.
10. Only after approval:
   - Create the commit on `dev`.
   - If this change is being prepared as a stable milestone, create or update the PR from `dev` into `main`.

## Task Document Template

Use this structure for every completed task document:

```markdown
# <Task ID> <Task Title>

## 任务目标

## 实际改动

## 为什么这么做

## 涉及文件

## 测试与验证

## 未完成事项

## 后续建议
```

## Git And PR Rules

- Do not develop directly on `main`.
- Use `dev` as the working branch for routine development.
- Prefer one task per PR.
- PRs should target `main`.
- Commit messages must use `type: 中文描述`.
- Do not commit `.env`, real API keys, temporary recordings, build artifacts, or local caches.
- Keep PR descriptions non-empty and matched to actual changes.
- Do not run commit or PR commands before explicit user approval.

## Completion Checklist

Before saying a task is ready for review, verify all items:

- Task tracker state was updated.
- Task development document exists.
- Verification result is documented.
- No unrelated files were changed.
- README or dependency documentation was updated when dependencies changed.
- Commit and PR text are ready for the user to review.
