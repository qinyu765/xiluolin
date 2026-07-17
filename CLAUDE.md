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

This repository contains XiLuoLin, an independently maintained open-source AI voice input assistant for office work, writing, and programming. It is a long-term product and engineering project, not a hackathon submission, throwaway demo, or minimum MVP.

The product helps users turn speech into directly usable text through speech recognition, scenario-aware rewriting, hotwords, history, and desktop output workflows.

## Communication Language

- Use Chinese for user-facing conversation, documentation, code comments, Issue content, PR descriptions, and Git commit messages.
- Keep code, API names, technical terms, URLs, paths, and third-party library names in English.

## Priority Scenarios

- Developers use voice to organize prompts, requirements, debugging notes, and instructions for Agent tools.
- Team members convert spoken task instructions or collaboration messages into clear, structured text.
- Creators capture ideas by voice and turn them into titles, key points, drafts, or task lists.

## Open-Source Project Direction

- Treat XiLuoLin as a maintainable desktop product and community project with an evolving roadmap.
- Optimize for real user value, reliability, privacy, accessibility, documentation quality, and sustainable maintenance.
- Keep product and architecture decisions understandable to external contributors.
- Prefer incremental, reviewable improvements over competition-oriented feature stacking or one-off presentation work.
- Record important decisions, limitations, setup requirements, and compatibility notes in `README.md`, `CONTRIBUTING.md`, or `docs/`.

## Branch And PR Workflow

- `main` is the protected, releasable baseline and should remain runnable.
- Do not develop directly on `main`. Use short-lived branches such as `feat/...`, `fix/...`, `docs/...`, or `chore/...`.
- External contributors should fork the repository when needed, create a focused branch, and open a PR targeting `main`.
- Keep each change small, coherent, verified, and independently revertible.
- Before committing or pushing, show the changed files, verification result, and proposed commit message, then obtain explicit user approval.
- Every PR should explain the problem, solution, verification method, user-visible impact, and related Issue when applicable.
- Use draft PRs for work in progress and request review only when the documented checks pass.

## Commit Rules

- Git commit messages must use `type: Chinese description`.
- `type` must be lowercase English. The description must be Chinese.
- Keep commit scope clear. One commit should represent one coherent change.
- Do not mix unrelated changes in one commit.
- Do not commit `.env`, real API keys, temporary recordings, build artifacts, or local caches.
- Common commit types include `feat`, `fix`, `docs`, `refactor`, `style`, `test`, and `chore`.

## Development Principles

- Build for sustained daily use rather than only proving a minimum loop.
- Keep the product focused on voice-to-usable-text workflows; evaluate broader features against the roadmap and user value.
- Preserve Provider abstractions so cloud services and future local models can evolve independently.
- Prefer existing ASR services or open-source models over training a speech model inside this repository.
- Treat privacy, local data ownership, failure recovery, and cross-platform behavior as product requirements.
- Every new dependency must have a clear purpose and be documented in the README or relevant technical document.
- Avoid speculative complexity, but do not use “MVP” as a reason to leave known reliability, security, or maintainability gaps undocumented.

## Quality And Verification

- Every functional change must explain how it was verified.
- Run relevant frontend and Rust checks before PR review; use `pnpm check` when the complete toolchain is available.
- If automated coverage is unavailable, document reproducible manual verification steps in the PR.
- Bug fixes should include the reproduction path, root cause, and post-fix verification.
- Changes affecting audio, global shortcuts, clipboard output, credentials, or external providers require platform and privacy impact notes.

## Documentation Requirements

- `README.md` (Chinese) and `README.en.md` (English) explain project positioning, current status, core features, setup, usage, roadmap, privacy boundary, and contribution entry points. Keep both files synchronized when shared information changes.
- `CONTRIBUTING.md` is the source of truth for community contribution workflow.
- Important product decisions, architecture changes, compatibility constraints, and scope boundaries belong in Issues or `docs/`.
- Historical development records may keep their original context, but they must be clearly marked as archived and must not define the current project direction.
