# T004 实现内置人格和默认人格选择

> **归档说明：** 本文记录特定开发阶段的背景与决策，其中的 MVP、demo、比赛或旧分支流程表述仅用于保留历史，不代表 XiLuoLin 当前的开源项目定位与协作方式。当前信息请以根目录 `README.md`、`CONTRIBUTING.md` 和 `docs/roadmap.md` 为准。

## 任务目标

为 `T004 实现内置人格和默认人格选择` 交付内置人格读取、默认人格切换和前端选择入口。首轮只覆盖系统内置人格和默认选择，不实现自定义人格编辑。

## 实际改动

- 在本地数据层新增 `set_default_persona`，支持把指定人格设为唯一默认人格。
- 新增 Tauri command `set_default_persona`，切换 SQLite 默认人格标记，并同步更新 Store 中的 `default_persona_id`。
- 调整内置人格初始化逻辑：仅在人格表为空时写入内置人格，避免后续初始化覆盖用户选择。
- 在前端新增默认人格选择面板，读取内置人格列表，展示人格描述、适用场景、输出语气和输出结构。
- 新增数据层测试，覆盖默认人格从 `Prompt 工程师` 切换到 `任务协作者` 后的持久化行为。

## 为什么这么做

默认人格会影响后续 ASR 后的文本整理提示词，是主流程中必须先确定的用户偏好。本任务沿用 T003 已建立的 SQLite + Store 数据层：SQLite 保存人格数据和默认标记，Store 保存轻量配置中的默认人格 ID，保证后续业务读取任一侧都能拿到一致状态。

本次没有加入自定义人格编辑，也没有引入额外前端状态管理，保持任务范围集中在“内置人格可选、默认值可保存”。

## 涉及文件

- `src-tauri/src/data.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/tests/local_data_layer.rs`
- `src/main.tsx`
- `src/styles.css`
- `docs/dev/task-tracker.md`
- `docs/dev/2026-05-24-t004-builtin-persona-default-selection.md`

## 工作分支与审批

- 当前工作分支：`dev`
- 是否已获批提交：否
- 是否已获批创建 `dev -> main` 的 PR：否

## 测试与验证

- `cargo test --test local_data_layer`：通过，6 个测试全部通过。
- `cargo check`：通过。
- `cargo fmt --check`：通过。
- `pnpm build`：通过。先在沙箱外执行 `$env:CI='true'; pnpm install --offline` 恢复依赖目录，再在沙箱外完成真实工作区构建验证。

## 未完成事项

- 桌面端手动交互验证尚未执行。
- 尚未提交 commit。
- 尚未创建 PR。

## 后续建议

运行 `pnpm tauri dev` 验证前端默认人格选择面板的实际交互。之后可以继续执行 `T005 实现热词词典`。
