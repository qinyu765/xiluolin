---
name: testing-tauri-apps
description: Guides testing for Tauri apps, including mocked unit tests, Tauri API mocking, end-to-end testing, and CI coverage.
---

# Testing Tauri Applications

Use this skill when adding or improving tests for a Tauri app.

## Cover

- Frontend unit tests with mocked Tauri APIs
- IPC and command mocking
- End-to-end testing with WebDriver or browser automation
- CI integration for Linux and Windows

## Main Workflow

1. Prefer unit tests for UI logic and command handling.
2. Mock Tauri APIs instead of calling native code directly in tests.
3. Add end-to-end tests for critical desktop flows.
4. Run tests in CI with platform-specific setup when needed.

## Mock Testing

- Use `@tauri-apps/api/mocks` for frontend tests.
- Reset mocks between tests.
- Verify that commands are invoked with the expected payload.

## E2E Testing

- Use Tauri-compatible WebDriver tooling for real app flows.
- Keep tests independent and focused on one user path.
- Use explicit waits for startup and UI readiness.

## CI Tips

- Build the app before running E2E tests.
- Use `xvfb` on Linux when needed.
- Match the platform driver version on Windows.

## Notes

- For Tauri v2, test the frontend and the Rust backend at the seam where commands meet UI.
- For a voice-input app, prioritize tests for recording state, transcription flow, result rendering, and copy/paste behavior.
