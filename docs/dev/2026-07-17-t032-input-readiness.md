# T032 增加语音输入就绪检查

## 任务目标

统一检查麦克风、ASR、文本处理、快捷键和自动粘贴能力，并在设置页展示缺失项和可操作提示。自动粘贴不可用时不得阻断录音、识别、历史保存和复制兜底。

## 实际改动

- 新增 `read_input_readiness` Tauri 命令，返回麦克风、ASR、文本处理、快捷键和自动粘贴五项检查。
- 每项检查包含 `ready`、`blocking` 和用户可读说明，前端可以区分阻断错误与软限制。
- 增加三个聚合状态：
  - `models_ready`：ASR 和文本 Provider 配置完整，供上传音频入口使用。
  - `can_process`：模型配置完整且麦克风可用，供应用内录音入口使用。
  - `can_dictate`：在 `can_process` 基础上至少一个全局快捷键真实注册成功。
- ASR 和文本配置按当前选中的 Provider 检查，不再固定要求 OpenAI Key。
- 设置页新增就绪卡片，以成功、阻断、警告三种状态展示检查结果和原因。
- 就绪检查在页面首次进入、用户手动刷新或任意配置保存成功后执行，不周期轮询系统凭据库。
- Windows 自动粘贴标记为可用并提示 UIPI 限制；macOS 标记为非阻断警告，提示辅助功能权限和目标恢复限制。
- 修复上传和应用内录音入口硬编码检查 OpenAI Key 的问题，统一改用后端就绪结果。

## 为什么这么做

此前用户只有在触发录音或调用 Provider 后才能发现配置、麦克风或快捷键问题，而且应用内入口固定要求 OpenAI Key，即使当前文本 Provider 是智谱也会被错误阻断。

就绪检查将配置完整性和运行能力集中到一个后端事实源，并明确自动粘贴属于可降级能力：即使不可用，用户仍可以完成识别、历史保存和复制。

## 涉及文件

- `src-tauri/src/readiness.rs`
- `src-tauri/src/lib.rs`
- `src/components/settings/InputReadinessCard.tsx`
- `src/pages/SettingsPage.tsx`
- `src/main.tsx`
- `src/types/config.ts`
- `README.md`
- `docs/requirements-analysis.md`
- `docs/solution-design.md`

## 测试与验证

执行：

```bash
pnpm check
```

覆盖场景：

1. 默认配置在没有凭据时不就绪。
2. 智谱 ASR 和智谱文本 Provider 使用各自字段计算就绪状态。
3. OpenAI ASR 和 OpenAI 文本 Provider 使用当前选中字段计算就绪状态。
4. 未知 Provider 即使存在 Key 也不会被判定为就绪。
5. 自动粘贴不可用不影响 `models_ready`、`can_process` 的计算。
6. 上传音频不再要求麦克风，也不再硬编码要求 OpenAI Key。
7. 应用内录音要求模型和麦克风就绪。

实际结果：

- TypeScript 类型检查和 Vite production build 通过。
- Rust 格式检查和 `cargo check` 通过。
- Rust 测试共 41 个通过，其中新增 4 个 Provider 就绪计算测试。
- `git diff --check` 通过。
- Windows CI 和真实设置页环境验证需要分支推送后确认。

## 执行复盘

### 遇到的问题

1. 最初设计使用周期轮询，但每次检查都会读取系统凭据库，可能产生不必要的 Keychain/Credential Manager 访问。
2. 前端旧逻辑直接检查 `asr_api_key` 和 `openai_api_key`，与可切换文本 Provider 的配置模型不一致。

### 解决流程

1. 改为首次进入、手动刷新和配置保存事件触发，不周期轮询。
2. 将 Provider 选择和必填字段判断集中到 Rust，就绪结构作为前端操作入口的统一依据。
3. 将模型、麦克风、快捷键和自动粘贴拆分，避免软能力阻断核心语音流程。

### 经验总结

- 就绪检查应复用真实运行配置，不能在前端复制一套容易漂移的 Provider 判断。
- 系统凭据库不适合高频轮询，应由配置变更事件驱动刷新。
- 自动粘贴必须作为可降级能力，而不是语音输入成立的前置条件。

## 未完成事项

- Windows 真实麦克风、快捷键冲突和自动粘贴能力需要桌面环境手动验证。
- macOS 尚未调用系统 API 精确读取辅助功能授权状态，当前展示为非阻断警告。

## 后续建议

进入 M2 Capture 与可选录音保留，并在历史记录中保存输入来源、实际 Provider 和投递方式快照。
