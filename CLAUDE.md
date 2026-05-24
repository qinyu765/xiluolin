# CLAUDE.md

Behavioral guidelines to reduce common LLM coding mistakes. Merge with project-specific instructions as needed.

**Tradeoff:** These guidelines bias toward caution over speed. For trivial tasks, use judgment.

## 1. Think Before Coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:

- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them - don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

## 2. Simplicity First

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask yourself: "Would a senior engineer say this is overcomplicated?" If yes, simplify.

## 3. Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:

- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

When your changes create orphans:

- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:

- "Add validation" → "Write tests for invalid inputs, then make them pass"
- "Fix the bug" → "Write a test that reproduces it, then make it pass"
- "Refactor X" → "Ensure tests pass before and after"

For multi-step tasks, state a brief plan:

```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria ("make it work") require constant clarification.

---

**These guidelines are working if:** fewer unnecessary changes in diffs, fewer rewrites due to overcomplication, and clarifying questions come before implementation rather than after mistakes.

---

# Project-Specific Instructions

This repository is for a Qiniu Cloud 2026 hackathon project. The selected topic is "voice input method".

The product is an AI voice input assistant for office work, writing, and programming. It helps users turn speech into directly usable text, reducing typing, editing, and polishing cost.

## Communication Language

- Use Chinese for user-facing conversation, documentation, code comments, Issue content, PR descriptions, and Git commit messages.
- Keep code, API names, technical terms, URLs, paths, and third-party library names in English.

## Priority Scenarios

- Developers use voice to quickly organize and improve prompts while working with Agent tools or vibe coding workflows.
- Team members convert spoken task instructions or collaboration messages into clearer and more structured text.
- Creators quickly capture ideas by voice and turn them into titles, key points, drafts, or task lists.

## Hackathon Evaluation Context

The project should optimize for three evaluation dimensions:

- Product completeness and innovation: reasonable product design, complete user flow, smooth interaction, and useful originality.
- Development process and quality: clear architecture, readable code, reasonable PR and commit history, robustness, and maintainability.
- Demo and communication: clear demo video and complete explanation of product value and functionality.

Project validity requirements:

- The work must stay aligned with the voice input method topic.
- Keep continuous PR and commit records during development. Do not import all work in one final batch.
- PR descriptions must not be empty and must match the actual changes.
- If third-party libraries, frameworks, open-source projects, or reused historical code are referenced, document their source and purpose.
- The README must list dependencies and clarify original work.

## Branch And PR Workflow

- The development baseline branch is `dev`.
- Daily development happens directly on `dev`.
- Do not create feature branches for routine work.
- `main` is the stable presentation branch. Do not develop features directly on `main`.
- After a stable milestone, open a PR from `dev` into `main`.
- PRs should target `main`.
- Each PR should do one thing and stay as small as practical.
- Split large features into multiple independent PRs.
- PR titles should summarize what was added or changed in one sentence.
- PR descriptions must include:
  - Feature description.
  - Implementation approach.
  - Test or verification method.
- After PR merge, the target branch must remain runnable and reproducible for judges.

## Commit Rules

- Git commit messages must use `type: Chinese description`.
- `type` must be lowercase English. The description must be Chinese.
- Keep commit scope clear. One commit should represent one coherent change.
- Do not mix unrelated changes in one commit.
- Do not commit `.env`, real API keys, temporary recordings, build artifacts, or local caches.
- Separate documentation, configuration, and feature-code commits when it helps review the development process.
- Common commit types:
  - `feat`: new feature.
  - `fix`: bug fix.
  - `docs`: documentation changes.
  - `refactor`: code refactor.
  - `style`: style or formatting changes.
  - `chore`: maintenance, dependency, or configuration changes.
- Commit examples:
  - `feat: 新增人格配置本地存储`
  - `fix: 修复录音状态未重置问题`
  - `docs: 更新语音输入流程需求文档`
  - `refactor: 重构数据层接口`
  - `style: 调整首页布局样式`
  - `chore: 更新项目依赖配置`

## Development Principles

- Prioritize a demonstrable, explainable, and reproducible minimum loop.
- Do not add features unrelated to voice input.
- Do not build a full system-level input method kernel in the MVP. Focus on a voice input assistant.
- Do not train speech models. Prefer existing ASR services or open-source models.
- Do not build meeting transcription, long-audio transcription, or a complex multi-user collaboration platform.
- Every new dependency must have a clear purpose and must be documented in the README.

## Quality And Verification

- Every functional change must explain how it was verified.
- When a test system exists, run tests and builds before PR merge.
- If no automated tests exist, document manual verification steps in the PR description.
- For bug fixes, describe the reproduction path before the fix and the verification method after the fix.

## Documentation Requirements

- README must explain project positioning, core features, run instructions, dependency notes, original work, and demo method.
- Important product decisions, competitor research, and scope boundaries should be written in Issues or `docs/`.
- Demo video scripts, pitch drafts, and interview Q&A should be placed under `docs/` or `pitch/`.
