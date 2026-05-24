---
name: rust
description: Expert in Rust development with focus on safety, performance, async programming, and systems code.
---

# Rust

Use this skill when working on Rust code, especially backend, async, or systems-level logic.

## Core Principles

- Prefer safety and clarity over cleverness.
- Use `Result` and `Option` properly.
- Keep functions small and focused.
- Use ownership and borrowing to avoid unnecessary cloning.

## Async

- Prefer `tokio` for async work when the project already uses it.
- Manage cancellation and shared state explicitly.
- Test async code with `#[tokio::test]` where appropriate.

## Error Handling

- Return meaningful errors.
- Propagate recoverable failures with `?`.
- Define custom error types when the domain needs them.

## Performance

- Avoid unnecessary allocations.
- Prefer iterators and references over copies where readable.
- Profile before optimizing.

## Testing

- Write unit tests for helpers and pure logic.
- Add integration tests for cross-module behavior.
- Keep test cases small and deterministic.
