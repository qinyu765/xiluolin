# T005 实现热词词典

## 任务目标

在 `dev` 分支上实现 MVP 阶段的本地热词词典，支持新增、编辑、删除、启用或停用热词，并提供可注入文本整理流程的启用热词上下文。

## 实际改动

- 在本地数据层补齐热词更新、删除和启用热词上下文组装能力。
- 注册 `update_hotword`、`delete_hotword`、`enabled_hotword_context` Tauri 命令，供前端调用。
- 在前端主界面新增热词修正面板，支持热词列表、新增、编辑、删除和启用状态切换。
- 在热词面板展示当前启用热词数量和上下文文本，方便后续接入 OpenAI 文本整理流程。
- 为热词 CRUD 和上下文组装补充集成测试。

## 为什么这么做

热词词典是提高专有名词、项目名和技术词修正准确度的基础能力。当前阶段选择本地 SQLite 存储和 Tauri 命令调用，延续已有数据层结构，避免引入新依赖和额外服务。

上下文格式采用列表形式，例如：

```text
- 七牛 -> 七牛云存储（云服务）
```

这个格式便于后续直接放入文本整理模型的输入中，也方便用户在界面上理解当前启用的修正规则。

## 涉及文件

- `src-tauri/src/data.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/tests/local_data_layer.rs`
- `src/main.tsx`
- `docs/dev/task-tracker.md`
- `docs/dev/2026-05-24-t005-hotword-dictionary.md`

## 测试与验证

- `cargo test --test local_data_layer`：通过，8 个测试全部通过。
- `cargo check`：通过。
- `pnpm exec tsc --noEmit`：通过。该命令在沙盒外运行，因为沙盒内 `node_modules` 可执行链接指向临时路径。
- `pnpm build`：通过。该命令在沙盒外运行，因为沙盒内 Vite 解析 `picomatch` 失败，根因是本地 `node_modules` 链接状态与沙盒视图不一致。

## 未完成事项

- 尚未接入 T007 的 OpenAI Responses API 文本整理流程。
- 尚未做批量导入、导出、分类筛选或搜索。
- 尚未运行 `pnpm tauri dev` 做桌面窗口手动检查。

## 后续建议

- 在 T007 文本整理任务中直接复用 `enabled_hotword_context` 的输出。
- 在热词数量增加后，再考虑搜索、分类筛选和批量管理。
