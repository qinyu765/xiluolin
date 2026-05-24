# T002 初始化 Tauri + React + pnpm 项目

## 任务目标

直接在 `dev` 分支上初始化 Tauri v2、React、TypeScript、pnpm 项目骨架，并补充 README 运行说明和本机开发依赖说明。

## 实际改动

- 新增 Vite + React + TypeScript 前端骨架。
- 新增 Tauri v2 后端骨架，包括 `src-tauri` 配置、Rust 入口和默认 capability。
- 新增 `package.json` 和 `pnpm-lock.yaml`，使用 pnpm 管理依赖。
- 新增 `.npmrc`，使用 `node-linker=hoisted` 降低 Windows 与 pnpm 链接树的兼容风险。
- 新增 `scripts/build.mjs` 和 `scripts/dev.mjs`，通过 Vite API 启动构建和开发服务。
- 更新 README，说明项目定位、依赖、运行命令、第三方依赖用途和原创范围。

## 为什么这么做

Tauri + React 是方案设计中确定的桌面助手技术路线。T002 只建立可运行工程结构，为后续录音、Provider、本地数据层和主流程开发提供稳定入口。

本机当前还没有完整 Rust/Tauri 开发环境，因此本次同时检查了 Rust、C++ Build Tools 和 WebView2 状态。WebView2 Runtime 已存在；Rust 与 C++ Build Tools 仍需要在宿主机完成安装后才能验证 Tauri 原生端运行。

## 涉及文件

- `package.json`
- `pnpm-lock.yaml`
- `index.html`
- `src/main.tsx`
- `src/styles.css`
- `scripts/build.mjs`
- `scripts/dev.mjs`
- `src-tauri/`
- `.npmrc`
- `README.md`
- `docs/dev/task-tracker.md`

## 测试与验证

验证命令：

```bash
pnpm install
pnpm build
rustc -V
cargo -V
pnpm tauri dev
```

实际结果：

- `pnpm install`：通过。首次在沙箱网络下失败，放行网络后安装成功并生成 `pnpm-lock.yaml`。
- `pnpm build`：通过。
- `rustc -V`：通过，`rustc 1.95.0 (59807616e 2026-04-14)`。
- `cargo -V`：通过，`cargo 1.95.0 (f2d3ce0bd 2026-03-21)`。
- `pnpm tauri dev`：通过，已进入 Tauri 开发模式并打开桌面窗口。

## 未完成事项

- 尚未创建 PR。
- 当前仅完成基础骨架，后续仍需继续 T003 本地数据层开发。

## 后续建议

继续 T003 本地数据层开发，优先建立人格、热词、历史记录的数据模型和本地存储层。
