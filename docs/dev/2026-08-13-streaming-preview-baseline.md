# 实时语音预览研发基线

## 范围

本文记录 XiLuoLin 二阶段实时语音预览 Task 0 的研发基线。当前阶段只调查和验证本地流式 ASR，不修改已经验收的录音、最终 ASR、文本整理、投递、UI 或悬浮窗链路。

基线 commit：`5789d00f94e8857e11e86b3bee2755fbd1987822`

工作分支：`feat/streaming-preview-research`

## 环境

- 设备：Apple M4，24 GiB 内存。
- 系统：Darwin 25.5.0，`arm64`。
- Rust：`rustc 1.97.1 (8bab26f4f 2026-07-14)`。
- Cargo：`cargo 1.97.1 (c980f4866 2026-06-30)`。
- Node.js：`v22.22.1`。
- pnpm：`10.26.2`。
- 已安装 Rust target：`aarch64-apple-darwin`、`x86_64-pc-windows-msvc`。

## 当前链路

- `recording_worker.rs` 使用 CPAL 打开一条输入流，按声道平均下混并写入原生采样率 WAV。
- 松开 Fn 或达到 28 秒限制后，现有完整 WAV 交给 `asr.rs`；默认最终识别仍是智谱 `glm-asr-2512`。
- 本地 Whisper 由 `local_asr.rs` 在完整 WAV 上运行，并在 worker 之外重采样到 16 kHz。
- `AsrCapabilities.live_audio` 对现有三个 Provider 均为 `false`。
- 录音悬浮窗、最终结果投递和历史记录仍只消费现有权威链路；本阶段不改变这些行为。

## 自动化基线

在全新 linked worktree 中运行：

```bash
pnpm install --frozen-lockfile
pnpm check
```

结果：

- Prettier、ESLint、TypeScript 和前端构建通过。
- Vitest：8 个测试文件、18 项测试通过。
- ASR 评测脚本：6 项测试通过。
- Tauri TypeScript bindings 与 Rust 契约一致。
- Rust 库单元测试：51 项通过。
- Rust 集成测试均通过；真实 Whisper smoke test 1 项按设计忽略，真实智谱 smoke test 1 项按设计忽略。
- `cargo check`、`cargo test` 和 doc tests 通过。

## 本轮无法复测的门禁

仓库当前不存在 `evals/asr/private/`、私人 WAV、真实预测文件、Provider 凭据或可复用的录音性能报告，因此本轮不能用自动化结果替代以下实机结论：

- 至少 80 条真人中文录音的 CER、热词召回率、标点 F1 和端到端 P50/P95。
- 真正小于 300 ms 的物理 Fn 短按取消。
- 25 秒提醒和 28 秒自动停止的物理设备边界竞态。
- 麦克风切换、拔出、默认设备回退和睡眠唤醒。
- 连续录音 100 次后的麦克风占用、线程、RSS 和临时文件状态。
- `Next.js` 等中英混合热词的真实语音大小写和标点保真。

这些项目保持“未在本轮执行”，不得记录为通过。实时预览仍必须默认关闭，且不得替换当前最终识别链路。

## Task 1 进入条件

Task 1 的 spike 必须与 Tauri 生产 crate、CPAL 录音、UI 和 IPC 隔离。候选模型只能用于本地实验；在模型权重、训练数据和再分发许可得到书面证据前，不得进入安装包或生产模型下载流程。
