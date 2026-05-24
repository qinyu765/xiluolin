---
name: setting-up-tauri-projects
description: Helps create and initialize Tauri v2 projects, including prerequisites, scaffolding, manual setup, and project structure for desktop and mobile apps.
---

# Setting Up Tauri Projects

Use this skill when starting or bootstrapping a Tauri v2 app.

## Cover

- System prerequisites for macOS, Windows, and Linux
- `create-tauri-app` and manual setup with an existing frontend
- Tauri project structure and key config files
- Common dev/build commands

## Decide First

- New project or existing project?
- Frontend stack: React, Vue, Svelte, Solid, Angular, vanilla, or Rust-based frontend?
- Desktop only or desktop + mobile?

## Main Workflow

1. Check platform prerequisites.
2. Scaffold with `create-tauri-app` or run `tauri init` in an existing frontend.
3. Confirm `tauri.conf.json`, `src-tauri/src/main.rs`, `src-tauri/src/lib.rs`, and capabilities are in place.
4. Verify dev/build commands before adding app-specific logic.

## Useful Commands

```bash
npm create tauri-app@latest
npx tauri init
npm run tauri dev
npm run tauri build
```

## Notes

- Keep frontend build output and Tauri frontend dist paths aligned.
- For React + TypeScript apps, keep the frontend under the existing web app structure and let Tauri own native capabilities.
- For mobile, pay attention to dev URL, websocket hot reload, and platform prerequisites.
