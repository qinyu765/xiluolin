# T006 实现智谱 GLM-ASR-2512 Provider

## 任务目标

在 `dev` 分支上实现智谱 GLM-ASR-2512 的基础语音识别能力，支持本地保存 ASR 配置，并能把短音频文件发送到 `audio/transcriptions` 接口得到原始识别文本。

## 实际改动

新增了 `src-tauri/src/asr.rs`，提供 ASR 配置、短音频本地校验、multipart 转写请求和 Tauri 命令入口。
在本地配置中加入 `asr_api_key`，并在前端新增智谱 ASR 配置面板，可保存 API Key、Base URL 和模型名。
补充了集成测试，覆盖缺少 API Key、音频格式不支持、以及 mock 服务转写成功的行为。

## 为什么这么做

ASR 是语音输入主流程的第一环，MVP 需要先把短音频转成原始文本，后续才能进入人格化整理。
把 API Key 和模型配置放在本地配置里，符合隐私预期，也便于用户在桌面端完成一次性配置。
测试先覆盖失败和成功路径，避免后续把请求参数、接口路径或配置字段改错。

## 涉及文件

- `src-tauri/src/asr.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/data.rs`
- `src-tauri/tests/zhipu_asr_provider.rs`
- `src-tauri/tests/local_data_layer.rs`
- `src/main.tsx`
- `README.md`
- `docs/dev/task-tracker.md`

## 测试与验证

- `cargo test --test zhipu_asr_provider`
- `cargo test --test local_data_layer`
- `cargo check`
- `pnpm exec tsc --noEmit`
- `pnpm build`

## 未完成事项

尚未接入录音采集与完整主流程。
尚未把 ASR 结果写入历史或串联 OpenAI 文本整理。

## 后续建议

下一步实现 OpenAI Responses API 文本整理，然后把 ASR、人格和复制输出串成完整主流程。
