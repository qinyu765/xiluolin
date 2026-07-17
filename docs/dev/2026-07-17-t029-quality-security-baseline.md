# T029 建立前后端质量检查基线

## 任务目标

修复仓库现有的 TypeScript 和 Rust 编译阻塞，建立本地统一检查命令与 Windows GitHub Actions，使后续直接在 `main` 开发时具备明确、可重复的质量门禁。

## 实际改动

- 删除没有运行时入口且与当前配置模型不一致的 `AppSettingsDialog`。
- 删除前端遗留的 `recording_mode` 配置字段和无效的 `setHasNonModifierKey` 调用。
- 建立类型安全的 Lucide 人格图标注册表，替代对整个模块命名空间的动态索引。
- 新增 `typecheck`、`build:frontend`、`check:rust` 和 `check` 脚本，并让 `pnpm build` 强制先执行类型检查。
- 新增 Windows GitHub Actions，覆盖依赖安装、TypeScript、Vite、Rust 格式、编译和测试。
- 从现有 ICO 生成 Tauri 编译所需的 PNG 图标。
- 在 macOS 上不启用需要 `macos-private-api` 的透明窗口设置，保持 Windows 透明指示器行为不变。
- 将已经与当前数据模型漂移的 Rust 测试更新为当前热词、配置和 ASR 接口。
- 对现有 Rust 源码执行一次全仓格式化，建立后续 `cargo fmt --check` 基线。

## 为什么这么做

此前 `pnpm build` 只执行 Vite 构建，即使 TypeScript 存在错误也会成功；Rust 检查则被缺失图标和 macOS API 兼容问题阻断，测试还引用已经删除的数据字段。仓库也没有 CI，无法阻止不可编译改动进入主分支。

本任务将所有检查统一到 `pnpm check`，并在 Windows runner 上执行同等门禁，为直接基于 `main` 的开发方式提供最低质量保障。

## 涉及文件

- `package.json`
- `.github/workflows/ci.yml`
- `src/lib/persona-icons.ts`
- `src-tauri/src/indicator.rs`
- `src-tauri/tests/`
- `README.md`
- `docs/dev/local-asr-implementation-plan.md`

此外删除 `src/components/dialogs/AppSettingsDialog.tsx`，并格式化 `src-tauri/src/` 下现有 Rust 文件。

## 测试与验证

本地 macOS 已执行：

```bash
pnpm install --frozen-lockfile
pnpm check
```

验证结果：

- TypeScript 类型检查通过。
- Vite production build 通过。
- `cargo fmt --check` 通过。
- `cargo check` 通过。
- `cargo test` 通过，共 21 个集成测试通过。
- Windows GitHub Actions 已添加，首次远端运行需要提交并推送到 `main` 后确认。

## 执行复盘

### 遇到的问题

1. `pnpm build` 原本不会执行 TypeScript 检查，隐藏了 6 个类型错误。
2. Tauri 编译需要 `src-tauri/icons/icon.png`，但仓库只有 ICO。
3. `WebviewWindowBuilder::transparent` 在 macOS 上需要 `macos-private-api` feature。
4. Rust 测试仍使用旧热词和配置字段，导致库代码能通过 `cargo check`，但测试无法编译。
5. 仓库历史 Rust 文件未建立统一 rustfmt 基线。

### 解决流程

1. 修复前端类型错误并让正式构建依赖类型检查。
2. 从现有 ICO 生成 PNG，不引入新的视觉设计。
3. 使用条件编译仅在非 macOS 平台启用透明窗口，避免启用私有 API。
4. 按当前数据模型更新测试，不恢复已删除字段。
5. 执行一次全仓 Rust 格式化，然后通过统一检查命令复验。

### 经验总结

- 构建命令必须包含类型检查，否则不能作为质量门禁。
- `cargo check` 不会保证所有集成测试能够编译，CI 必须同时执行 `cargo test`。
- 跨平台 Tauri API 需要在实际目标平台或条件编译下验证。

## 未完成事项

- Windows CI 尚未远端运行，需推送 `main` 后确认。
- macOS 指示器当前不启用窗口透明效果；如未来需要透明效果，应单独评估 `macos-private-api` 的分发影响。

## 后续建议

进入 T030，加固 API Key 本地存储、敏感日志和录音临时文件生命周期。
