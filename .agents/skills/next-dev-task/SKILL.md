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

## Execution Pitfalls To Check

Run these checks when the task touches dependencies, external APIs, build tooling, commit, push, or PR creation:

- For a new Rust dependency, expect the first `cargo` command to need network access. If crates.io access fails because of sandbox or TLS/network restrictions, rerun the same command with escalation instead of changing code.
- For frontend verification on Windows, `node_modules` can be unreadable inside the sandbox and cause false failures such as missing `picomatch/index.js` or `tsc` not found. First confirm with `Test-Path` or `Get-ChildItem`; then run `pnpm build` and `pnpm exec tsc --noEmit` with escalation if the source files are unchanged and the error is dependency-read related.
- For HTTP client libraries, verify the installed crate API from local registry source before assuming examples from memory. In this project `ureq` 3.x multipart lives under `ureq::unversioned::multipart`.
- For provider tasks, use mock HTTP tests for request shape and local validation. Real API smoke tests are recommended before merge when credentials and sample files are available, but may be deferred if the task document records the deferral and no secrets are committed.
- Before creating a PR, read `git remote -v` and use the actual GitHub owner/repo. Do not infer owner or repo from the local folder path.
- After committing on `dev`, check `git status --short --branch`. If `dev` is ahead of `origin/dev`, push `dev` before creating the `dev -> main` PR.
- After PR creation, update `docs/dev/task-tracker.md` PR column with the PR number or URL.

## Task Document Template

Use this structure for every completed task document:

```markdown
# <Task ID> <Task Title>

## 任务目标

## 实际改动

## 为什么这么做

## 涉及文件

## 测试与验证

## 执行复盘

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
- Task tracker PR column was updated after PR creation.
- Task development document exists.
- Verification result is documented.
- Known execution pitfalls and deferred real-service checks are documented.
- No unrelated files were changed.
- README or dependency documentation was updated when dependencies changed.
- Commit and PR text are ready for the user to review.
