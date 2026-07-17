# T034 增加 Capture 试听和重新处理

## 任务目标

为已保留录音提供试听和重新转写，并允许所有历史记录使用当前人格和文本 Provider 重新整理。

## 实际改动

- 新增受管录音读取命令，只允许通过历史记录关联读取应用 `recordings` 目录内的 WAV。
- 历史列表为已保留录音增加试听和重新转写按钮；所有记录增加重新整理按钮。
- 试听在前端构造临时 WAV Blob，播放结束或启动失败后释放对象 URL。
- 重新转写使用当前 ASR、文本 Provider、模型、默认人格和热词，更新原历史的原始文本、最终文本及 Provider 快照。
- 重新整理只使用历史 `raw_text`，不要求保留录音，并更新当前人格、文本 Provider、模型和降级状态。
- 两类重新处理都保留原录音关联、输入来源、录音时长和既有投递方式，不创建重复历史。
- 修复 OpenAI ASR 配置漂移：设置页写入的 `openai_api_key/openai_base_url/openai_asr_model` 现在由正常处理、就绪检查和重新转写共同使用。
- `AppConfig` 提供统一的当前 ASR 和文本 Provider 字段选择，减少不同流程再次漂移。

## 为什么这么做

T033 已建立录音和历史关联，但用户尚不能在界面中利用保留录音。重新处理能力使历史从静态结果升级为可复用 Capture，同时允许用户更换人格或模型后重新生成结果。

实现过程中发现 OpenAI ASR 设置页和处理链路使用不同字段；如果不统一，OpenAI Whisper 会表现为“配置成功但无法调用”。因此将 Provider 字段选择收口到 `AppConfig`。

## 涉及文件

- `src-tauri/src/history_reprocessing.rs`
- `src-tauri/src/recording_storage.rs`
- `src-tauri/src/data.rs`
- `src-tauri/src/readiness.rs`
- `src-tauri/src/pipeline.rs`
- `src/components/home/VoiceInputStatsCard.tsx`
- `src/pages/HomePage.tsx`
- `src/main.tsx`
- `src-tauri/tests/local_data_layer.rs`
- `README.md`
- `docs/requirements-analysis.md`
- `docs/solution-design.md`

## 测试与验证

执行：

```bash
pnpm check
```

结果：

- TypeScript 类型检查和 Vite production build 通过。
- Rust 格式检查和 `cargo check` 通过。
- Rust 测试共 46 个通过。
- 数据层测试验证重新转写和重新整理后保留录音关联、来源和投递方式。
- Provider 就绪测试验证 OpenAI ASR 使用 OpenAI 配置字段。
- `git diff --check` 通过。

手动/真实服务待验证：

1. 历史录音试听。
2. 真实 ASR 重新转写。
3. 当前人格重新整理。
4. OpenAI Whisper 配置和调用。

## 执行复盘

### 遇到的问题

1. OpenAI ASR 设置页使用 OpenAI 字段，但旧处理链路固定读取智谱 ASR 字段。
2. 重新处理不能创建新历史，否则会丢失 Capture 的连续性并产生重复记录。
3. 浏览器播放二进制录音需要控制 Blob URL 生命周期。

### 解决流程

1. 在 `AppConfig` 中统一选择当前 ASR 和文本 Provider 的 Key、Base URL 和模型。
2. 为数据层增加原记录更新方法，分别覆盖重新转写和仅重新整理场景。
3. 受管读取返回 WAV 字节，前端播放后释放临时 URL。
4. 保留录音路径、输入来源、时长和投递方式，只更新处理结果和实际 Provider 快照。

### 经验总结

- Provider 可切换系统必须只有一个配置选择事实源。
- Capture 重新处理应更新同一记录，而不是把每次尝试都伪装成新的语音输入。
- 音频读取必须继续通过历史关联和受管目录校验，不能恢复任意路径读取接口。

## 未完成事项

- 自动化测试不调用真实第三方 Provider，真实重新转写和人格整理需要配置 API Key 手动验证。
- 播放器目前使用系统 WebView 的基础音频能力，没有波形、暂停进度和倍速控制。

## 后续建议

进入 M3 本地 ASR 与混合 Provider；本地模式必须显式控制是否允许云端降级。
