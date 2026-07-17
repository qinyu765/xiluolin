# T003 建立本地数据层

> **归档说明：** 本文记录特定开发阶段的背景与决策，其中的 MVP、demo、比赛或旧分支流程表述仅用于保留历史，不代表 XiLuoLin 当前的开源项目定位与协作方式。当前信息请以根目录 `README.md`、`CONTRIBUTING.md` 和 `docs/roadmap.md` 为准。

## 任务目标

为后续人格、热词、历史记录和统计能力建立可测试的本地存储基础，对应任务跟踪表中的 `T003 建立本地数据层`。

## 实际改动

- 在 `src-tauri` 新增 `data` 模块，封装 SQLite 业务表、配置默认值和最小 Tauri commands。
- 使用 `rusqlite` 建立本地 SQLite 数据层，覆盖 `personas`、`hotwords`、`history_records` 三张表。
- 使用 `tauri-plugin-store` 保存轻量配置，包含默认人格、Provider 占位、快捷键、录音模式、输出方式和自动保存开关。
- 在 Tauri 启动入口注册 `store`、`sql` 插件和数据层命令。
- 新增集成测试，覆盖建表、幂等初始化、热词写入读取、历史记录写入读取和默认配置。
- 更新 README、任务跟踪表和本任务开发文档。

## 为什么这么做

本阶段目标是先打通桌面端的本地数据基础，再把人格、热词和历史能力逐步接上前端。SQLite 负责结构化业务数据，Store 负责轻量配置，职责分离后更容易继续扩展，也更方便后续按任务拆分验证。

这里没有提前做前端管理界面，只保留后续任务需要的最小后端接口，避免把 T003 做成大而全的数据平台。

## 涉及文件

- `src-tauri/src/data.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/Cargo.toml`
- `src-tauri/capabilities/default.json`
- `src-tauri/tauri.conf.json`
- `src-tauri/tests/local_data_layer.rs`
- `docs/dev/task-tracker.md`
- `README.md`

## 工作分支与审批

- 当前工作分支：`dev`
- 是否已获批提交：否
- 是否已获批创建 `dev -> main` 的 PR：否

## 测试与验证

- `cargo test --test local_data_layer`：通过，5 个测试全部通过。
- `cargo test`：通过。
- `cargo check`：通过。
- `pnpm build`：通过，前端构建成功。

## 未完成事项

- 尚未提交 commit。
- 尚未创建 PR。

## 后续建议

继续执行 `T004` 和 `T005`，把内置人格与热词词典接到这套本地数据层上，再进入历史记录与统计展示。
