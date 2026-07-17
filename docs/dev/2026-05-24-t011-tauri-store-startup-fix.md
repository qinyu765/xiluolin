# T011 修复 Tauri Store 插件启动崩溃

> **归档说明：** 本文记录特定开发阶段的背景与决策，其中的 MVP、demo、比赛或旧分支流程表述仅用于保留历史，不代表 XiLuoLin 当前的开源项目定位与协作方式。当前信息请以根目录 `README.md`、`CONTRIBUTING.md` 和 `docs/roadmap.md` 为准。

## 任务目标

修复桌面应用执行 `pnpm tauri dev` 时在 Tauri Store 插件初始化阶段崩溃的问题，确保开发模式可以正常打开 XiLuoLin 窗口。

## 实际改动

移除 `src-tauri/tauri.conf.json` 中无效的 `plugins.store` 空对象配置。

## 为什么这么做

当前 Rust 侧已经通过 `tauri_plugin_store::Builder::default().build()` 注册 Store 插件。`tauri-plugin-store` 2.4.x 不需要在 `tauri.conf.json` 中提供空对象配置；保留 `"store": {}` 会导致插件初始化时反序列化失败，报错为 `invalid type: map, expected unit`。

## 涉及文件

- `src-tauri/tauri.conf.json`
- `docs/dev/task-tracker.md`

## 测试与验证

- `cargo check`
- `pnpm build`
- `pnpm tauri dev`

`pnpm tauri dev` 在短跑验证中没有复现 `plugins.store` 启动即崩溃，进程保持运行直到验证超时后手动结束。

## 执行复盘

该问题属于配置契约不匹配。修复只调整 Tauri 配置，不改动 Rust 插件注册逻辑，也不引入新依赖。

## 未完成事项

无。

## 后续建议

后续新增或升级 Tauri 插件时，先确认插件是否需要 `tauri.conf.json` 配置项，避免把空对象当作默认配置写入。
