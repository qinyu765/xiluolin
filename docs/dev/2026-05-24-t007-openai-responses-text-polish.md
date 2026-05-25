# T007 实现 OpenAI Responses API 文本整理

## 任务目标

在 `dev` 分支上实现 OpenAI Responses API 文本整理能力，支持输入原始识别文本、当前人格 prompt 和热词上下文，输出可直接复制使用的整理结果。

## 实际改动

新增 `src-tauri/src/text_polish.rs`，提供 OpenAI 文本整理配置、请求结构、Responses API 调用、响应解析和 Tauri 命令入口。
在本地配置中加入 `openai_api_key` 和 `openai_base_url`，并保留默认模型 `gpt-4.1-mini`。
在前端新增 OpenAI 配置面板，可本地保存 API Key、Base URL 和模型名。
补充 mock HTTP 测试，覆盖缺少 API Key、请求体包含人格与热词上下文、请求失败时返回原始文本兜底。

## 为什么这么做

文本整理是语音输入助手把 ASR 原始文本变成可用文本的关键步骤。当前任务只建立 Provider 能力和配置入口，完整的录音、ASR、整理、复制串联留给 T008。
请求失败时返回原始识别文本，可以让后续主流程保留可复制内容，避免一次文本模型失败导致用户完全丢失输入结果。

## 涉及文件

- `src-tauri/src/text_polish.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/data.rs`
- `src-tauri/tests/openai_text_polish_provider.rs`
- `src-tauri/tests/local_data_layer.rs`
- `src/main.tsx`
- `README.md`
- `docs/dev/task-tracker.md`

## 测试与验证

- `cargo test --test openai_text_polish_provider`
- `cargo test --test local_data_layer`
- `cargo check`
- `pnpm exec tsc --noEmit`
- `pnpm build`

## 执行复盘

- OpenAI Responses API 调用使用 `POST /v1/responses`，`instructions` 承载固定整理规则和当前人格要求，`input` 承载原始识别文本、热词上下文和输出要求。
- 当前实现解析 `output_text` 字段，适合本任务的非流式文本整理闭环。若后续需要更复杂的输出结构，再扩展响应解析。
- 真实 OpenAI smoke test 因未提供本地 API Key 和可公开样例输入，延期到本地配置后执行；仓库内不保存真实 Key。

## 未完成事项

尚未把 ASR 结果、人格化整理和复制输出串成完整主流程。
尚未将整理结果写入历史记录。
尚未使用真实 OpenAI API Key 做端到端 smoke test。

## 后续建议

下一步执行 T008，串联短音频输入、智谱 ASR、OpenAI 文本整理、结果展示和复制。
