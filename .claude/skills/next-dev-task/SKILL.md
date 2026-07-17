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
2. Confirm the current branch. Work directly on `main`; before editing, run a fast-forward-only pull from `origin/main`.
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
8. Before the task commit, update `docs/dev/task-tracker.md`:
   - `Doing` when starting.
   - `Review` when implementation and verification are ready for review.
   - `Done` only after the change is pushed to `main`, a PR is merged, or the user explicitly confirms completion.
   - `Blocked` with a concrete blocker when work cannot continue.
   - Use `不适用（main 直提）` in the PR column for direct-to-`main` tasks; use `待创建` only when a PR is planned but does not exist yet.
9. After verification, publish the task without pausing for approval unless the user explicitly requests review-only or dry-run behavior. Before publishing, summarize:
   - Changed files.
   - Verification result.
   - Commit message using `type: 中文描述`.
   - PR description containing feature description, implementation approach, and verification method.
10. Publish through the current short-lived branch:
    - Create a focused commit containing the implementation, task document, and task tracker state change.
    - Push the branch and create or update a PR targeting `main`.
    - Do not create a follow-up commit solely to add the PR number or URL to `docs/dev/task-tracker.md`; report the URL in the final response.
    - Stop before commit or push only when the user explicitly asks not to publish, requests a dry run, or the working tree contains unrelated changes whose ownership is unclear.

## Execution Pitfalls To Check

Run these checks when the task touches dependencies, external APIs, build tooling, commit, push, or PR creation:

- On Windows in this repository, the sandbox can produce false permission or dependency-read failures. Run these commands with escalation directly instead of first trying them inside the sandbox:
  - `pnpm build`
  - `pnpm exec tsc --noEmit`
  - `pnpm tauri dev`
  - `git add`
  - `git commit`
  - `git push`
  - `git remote set-url`
  - `Stop-Process` when stopping this project's `xiluolin`, `cargo`, or Tauri development processes.
- Use direct escalation for the commands above because `pnpm` may need reliable `node_modules` access, `pnpm tauri dev` opens a desktop GUI, Git write commands may need `.git` lock/config writes, `git push` needs network access, and stopping development processes may require access outside the sandbox.
- For commands not listed above, run normally first. If the failure is clearly caused by permission, network, GUI, or sandbox file access, rerun the same command with escalation and keep the command scope unchanged.
- For a new Rust dependency, expect the first `cargo` command to need network access. If crates.io access fails because of sandbox or TLS/network restrictions, rerun the same command with escalation instead of changing code.
- For frontend verification on Windows, `node_modules` can be unreadable inside the sandbox and cause false failures such as missing `picomatch/index.js` or `tsc` not found. First confirm with `Test-Path` or `Get-ChildItem`; then run `pnpm build` and `pnpm exec tsc --noEmit` with escalation if the source files are unchanged and the error is dependency-read related.
- For HTTP client libraries, verify the installed crate API from local registry source before assuming examples from memory. In this project `ureq` 3.x multipart lives under `ureq::unversioned::multipart`.
- For provider tasks, use mock HTTP tests for request shape and local validation. Real API smoke tests are recommended before merge when credentials and sample files are available, but may be deferred if the task document records the deferral and no secrets are committed.
- Before creating a PR, read `git remote -v` and use the actual GitHub owner/repo. Do not infer owner or repo from the local folder path.
- After committing, check `git status --short --branch`; push the current short-lived branch and confirm it tracks the expected remote branch.
- For PR creation, prefer using GitHub MCP tool (`mcp__github__create_pull_request`) over `gh` CLI, as `gh` may not be installed. Extract owner and repo from `git remote -v` output.
- After PR creation, do not make a second commit only to update the tracker PR column. Put the PR URL in the final response and, when useful, the PR description.

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

- Use `main` as the stable baseline and start each task from an up-to-date short-lived branch.
- Pull `origin/main` with `--ff-only` before creating the task branch.
- Keep commits small, coherent, verified, and independently revertible.
- Push the task branch and open a PR targeting `main` after verification; do not pause for approval unless the user explicitly requests review-only or dry-run behavior.
- Commit messages must use `type: 中文描述`.
- Do not commit `.env`, real API keys, temporary recordings, build artifacts, or local caches.
- Keep PR descriptions non-empty and matched to actual changes.

## Completion Checklist

Before saying a task is ready for review, verify all items:

- Task tracker state was updated.
- Task tracker state, verification method, and task document were committed together with the implementation.
- For direct-to-`main` work, the pushed commit hash was reported; when a PR is used, its URL was reported without requiring a PR-link-only commit.
- Task development document exists.
- Verification result is documented.
- Known execution pitfalls and deferred real-service checks are documented.
- No unrelated files were changed.
- README or dependency documentation was updated when dependencies changed.
- Commit and PR text are ready for the user to review.
