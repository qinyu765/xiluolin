# Task 2：人格处理模式、数据迁移与 Pipeline 顺序——实施报告

## 范围与实现

- `Persona` 与 `PersonaDraft` 增加 `processing_mode`；SQLite 新旧库均默认 `polish`。
- 新增内置人格 `verbatim`（名称“原文听写”），模式为 `verbatim`。迁移只插入缺失记录，绝不覆盖已有内置或用户人格；已有默认人格保持不变，重复初始化不会重复插入。
- `HistoryRecord` 与 `HistoryRecordDraft`、建表 SQL、迁移、读写及重新处理更新路径增加 `text_processing_mode`；旧历史记录迁移为 `polish`。
- `LocalDatabase::enabled_hotword_texts()` 返回结构化的启用热词，排序与现有词典列表一致。Pipeline 在 ASR 前读取默认人格和这批热词；人格描述没有发送给 ASR。
- Pipeline 将完整列表传给 `AsrRequest`（智谱 Provider 仍负责 trim、稳定去重和前 100 个限制）。润色请求仍使用全部启用热词上下文。
- `verbatim` 跳过文本 Provider 和 `Refining` 进度，仅用 `split_whitespace` 做 Unicode 空白折叠及首尾清理，保留词内容、标点和大小写；历史明确写入 `verbatim`，文本 Provider/模型为空。
- 人格编辑框可选择“文本润色”或“原文听写”；热词页在启用、trim 后稳定去重的词数超过 100 时显示“ASR 仅用前 100 个、其余仍用于文本整理”的中文提示，不阻止保存。
- 已重新生成 Tauri TypeScript bindings。

## TDD 记录

### RED

1. 初始 Rust 定向测试（在 `src-tauri`）按预期因以下功能尚不存在而编译失败：
   - `normalize_verbatim_text`、`Persona.processing_mode`、`HistoryRecord.text_processing_mode`；
   - `PersonaDraft.processing_mode`、`HistoryRecordDraft.text_processing_mode`；
   - `LocalDatabase::enabled_hotword_texts`。
2. 前端纯函数测试按预期失败：`src/lib/hotword-limit.ts` 不存在，Vite 无法解析 `./hotword-limit`。
3. 原文 Pipeline 回归测试的变异验证：临时将 verbatim 分支改为润色分支，测试按预期失败，错误为 `文本处理 API Key 不能为空`。恢复后测试通过。
4. 润色 Pipeline 回归测试的变异验证：临时传给 `AsrRequest` 空热词列表，测试按预期失败于 `asr_request.contains("词100")`。恢复完整列表后测试通过。

### GREEN

- 数据迁移、热词结构化列表、原文规范化以及前端提示的定向测试通过。
- Pipeline 集成测试覆盖：
  - 原文模式：ASR 收到热词而不含人格描述；未触发 `Refining`；无文本 API Key 也可完成；历史模式为 `verbatim`。
  - 润色模式：第 101 个热词同时出现在 OpenAI ASR 软提示及文本润色上下文；ASR 不含人格描述，文本 Provider 收到人格描述；进度依次为 `Transcribing`、`Refining`。

## 验证结果

- `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`：通过。
- `cargo check --manifest-path src-tauri/Cargo.toml`：通过。
- `cargo test --manifest-path src-tauri/Cargo.toml`：通过，84 个非忽略 Rust 测试通过；本地模型 smoke 与智谱真实 smoke 各 1 个按预期忽略。
- `pnpm bindings:generate && pnpm bindings:check`：通过。
- `pnpm check`：通过；包含 Prettier、ESLint、Vitest（3 个文件、7 个测试）、TypeScript、前端生产构建、bindings 与完整 Rust 门禁。
- 开发中曾两次从仓库根目录直接运行未带 `--manifest-path` 的 Cargo 命令，均因根目录没有 `Cargo.toml` 立即失败；随后已使用 `src-tauri` 或 `--manifest-path` 重新运行并通过，不是代码失败。

## 已知风险

- 真实智谱和本地模型 smoke 未运行，以避免未显式提供凭据或模型时产生外部依赖；常规 Provider 的 mock 覆盖保持通过。
- 智谱的 100 词裁剪仍位于 Task 1 Provider 层；本任务故意向 `AsrRequest` 保留完整结构化列表，便于 OpenAI/本地软提示和文本润色使用全部热词。
