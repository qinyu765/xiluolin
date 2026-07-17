# T008 实现主流程

> **归档说明：** 本文记录特定开发阶段的背景与决策，其中的 MVP、demo、比赛或旧分支流程表述仅用于保留历史，不代表 XiLuoLin 当前的开源项目定位与协作方式。当前信息请以根目录 `README.md`、`CONTRIBUTING.md` 和 `docs/roadmap.md` 为准。

## 任务目标

在 `dev` 分支上实现短音频输入主流程，支持用户上传 wav 或 mp3 短音频后，串联智谱 ASR、OpenAI Responses API 文本整理、结果展示、复制和历史保存。

## 实际改动

新增 `src-tauri/src/pipeline.rs`，提供上传音频字节写入临时文件、主流程编排和 `process_uploaded_audio` Tauri 命令。
前端新增“短音频输入”面板，支持选择 wav 或 mp3 文件、展示处理状态、显示原始识别文本和整理结果，并提供复制结果按钮。
新增 `src-tauri/tests/voice_input_pipeline.rs`，覆盖空音频拒绝和上传音频临时文件准备逻辑。
更新任务跟踪表，将 T008 标记为 Review。

## 为什么这么做

当前已有智谱 ASR Provider、OpenAI 文本整理 Provider、本地人格、热词和历史记录能力。T008 选择先实现短音频上传路径，可以用最少改动串起现有能力，形成可演示的最小闭环。

本任务没有先实现实时录音和全局快捷键，原因是录音权限、设备采集和快捷键监听会引入新的系统交互面。短音频上传已经能验证核心产品链路，录音能力可以在后续任务中独立补齐。

## 涉及文件

- `src-tauri/src/pipeline.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/tests/voice_input_pipeline.rs`
- `src/main.tsx`
- `docs/dev/task-tracker.md`

## 测试与验证

- `cargo test --test voice_input_pipeline`
- `cargo check`
- `pnpm exec tsc --noEmit`
- `pnpm build`

沙盒内 `pnpm exec tsc --noEmit` 报 `tsc` 找不到，但已确认 `node_modules/.bin/tsc.cmd` 存在；按项目技能规则在沙盒外重跑后通过。
沙盒内 `pnpm build` 报 `node_modules/picomatch/index.js` 读取失败；按项目技能规则在沙盒外重跑后通过。

真实智谱 ASR + OpenAI 端到端 smoke test 尚未执行，因为仓库不保存真实 API Key，也没有可公开提交的样例音频。该验证应在本地配置 API Key 和样例短音频后执行。

## 执行复盘

主流程复用已有 Provider 和数据层，没有新增第三方依赖。前端通过文件输入读取音频字节，再调用 Tauri 命令，Rust 侧写入临时文件后交给现有 ASR 文件接口。

OpenAI 整理失败时沿用 T007 的兜底策略，返回原始识别文本作为最终结果，保证用户仍有可复制内容。ASR 失败时不写入历史记录。

## 未完成事项

尚未实现麦克风录音和全局快捷键触发。
尚未实现自动粘贴到当前输入位置。
尚未执行真实服务端到端 smoke test。

## 后续建议

下一步可以执行 T009，展示历史记录和统计卡片；也可以在单独任务中补齐录音控制，让主流程从“上传短音频”扩展为“录音或上传”。
