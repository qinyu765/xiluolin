# T036 macOS 语音输入链路可靠性、凭据和 Provider 体验修复

> **实施记录：** 本文记录 2026-07-20 对 macOS 真实语音输入链路的集中排查。内容包括当时的日志、错误尝试、根因、最终实现和验证结论。当前使用方式以 [`../usage-guide.md`](../usage-guide.md) 和代码为准。

## 开源协作说明

关联 Issue：无，问题由本地真实运行和用户反馈直接触发。

实施分支：`fix/macos-voice-pipeline-reliability`。

## 任务背景

XiLuoLin 已经具备以下模块：

```text
全局快捷键
  → 录音
  → ASR
  → 文本整理
  → 恢复录音开始时的窗口
  → Command+V 自动粘贴
  → 历史记录
```

单元测试和短链路测试能够通过，但在 macOS 真实使用中连续出现多种看似独立、实际互相放大的问题：

1. 长按快捷键松开后偶尔仍继续录音；
2. 后续快捷键一直提示“上一条语音输入仍在处理中”；
3. 智谱 ASR 只返回 `Broken pipe`，看不到真实 HTTP 错误；
4. 文本整理错误被固定描述为 OpenAI 错误，即使用户选择的是智谱；
5. GLM 文本整理有时需要 20～40 秒；
6. 应用启动时连续弹出多个 macOS 钥匙串授权窗口；
7. ASR 和整理完成后，应用及 `pnpm tauri dev` 被系统直接终止；
8. 自动粘贴没有发生；
9. 设置页把服务商与固定模型名绑定，和“模型名可配置”冲突；
10. 录音状态窗口和基础控件缺少清晰的过程反馈。

这些问题不能只靠修改某一处错误提示解决，需要沿着快捷键、会话状态、网络请求、凭据、模型参数和 macOS 输入注入完整排查。

## 任务目标

- 消除快捷键 Pressed/Released 乱序造成的录音悬挂；
- 确保任何处理失败都会释放 CaptureSession；
- 让 ASR 返回真实 HTTP 状态和响应体；
- 减少 macOS Keychain 的授权次数；
- 降低智谱文本整理延迟；
- 消除自动粘贴导致的 macOS 原生崩溃；
- 保留剪贴板兜底和目标窗口恢复；
- 将文本处理提示改为 Provider 中性；
- 增加兼顾可观测性和隐私的结果日志；
- 改善录音状态条和桌面交互反馈；
- 补齐自动化测试和用户文档。

## 问题一：长按快捷键事件竞态

### 现象

典型日志顺序：

```text
快捷键事件触发: state=Pressed
长按模式: 按键按下，准备开始录音
快捷键事件触发: state=Released
长按模式: 录音启动成功
```

`Released` 已经到达，但“录音启动成功”发生在它之后。松开处理读取到的状态仍是“未录音”，因此直接返回；稍后启动完成后，录音已经没有对应的停止事件。

下一次触发会看到：

```text
启动录音失败: 上一条语音输入仍在处理中
```

### 根因

全局快捷键回调把每个事件分别交给 `tauri::async_runtime::spawn`。录音启动包含设备和文件初始化，Pressed 任务不一定在 Released 任务之前完成。

事件产生顺序正确，不代表异步任务完成顺序正确。

### 解决

在 `HotkeyState` 中增加独立的 `event_gate`：

```text
Pressed 进入 gate
  → 完成录音启动和状态写入
  → Released 才能进入 gate
  → 正常停止录音
```

这个锁只负责快捷键事件顺序，不与状态锁混用，避免长时间持有 `HotkeyState` 锁。

### 实施中遇到的编译错误

初版写法在 block 末尾直接返回 `state.lock().await.event_gate.clone()`，Rust 报告 `E0597`：临时 `MutexGuard` 的析构时机可能晚于局部变量。

最终改为显式局部变量：

```rust
let gate = state.lock().await.event_gate.clone();
gate
```

确保 guard 在离开 block 前完成析构。

## 问题二：失败会话未必释放

### 现象

ASR 或文本处理失败后，后续录音持续提示：

```text
上一条语音输入仍在处理中
```

### 根因

错误路径调用：

```rust
sessions.finish(session_id, CaptureStatus::Failed)
```

`finish` 会验证状态机转换，而调用方忽略了返回值。只要此前某次状态更新失败或状态与预期不一致，`finish` 也会失败，`current` 会话便一直保留。

### 解决

错误路径不再尝试完成正常状态机，而是按 session ID 无条件取消：

```rust
sessions.cancel(&session_id)
```

正常成功路径仍使用严格状态机；异常恢复路径优先保证系统可以继续接收下一条语音。

## 问题三：智谱 ASR 的 `Broken pipe`

### 现象

智谱 ASR 日志只有：

```text
ASR 请求失败：io: Broken pipe (os error 32)
```

看不到状态码，也无法区分 Key 无效、余额、限流或请求格式问题。

### 根因

原实现使用 `ureq` 发送 multipart。远端网关可能在客户端仍上传请求体时提前拒绝请求；此时客户端只观察到写端断开，真实 HTTP 响应丢失。

### 解决

智谱 ASR multipart 改用 `reqwest::blocking`：

- 显式读取音频字节；
- WAV 使用 `audio/wav`，MP3 使用 `audio/mpeg`；
- 60 秒请求超时；
- 手动读取非 2xx 状态和响应体；
- 成功后再解析 JSON。

修复后，同一个无效 Key 可以得到真实错误：

```text
http status: 401
令牌已过期或验证不正确
```

这证明网络链路和请求格式已经可观测，认证问题应由用户在 Provider 控制台处理，而不是继续修改上传代码。

## 问题四：macOS Keychain 多次授权

### 现象

应用启动后连续出现多个系统弹窗，要求访问：

```text
服务：com.xiluolin.desktop
账户：asr_api_key / openai_api_key / zhipu_api_key
```

本机还保留旧服务名 `com.xiluolin.app`，最多存在两套、共六个旧条目。

### 放大因素

- 三个 API Key 分成三个 Keychain 条目；
- 后端启动注册快捷键时读取配置；
- 前端初始化再次读取配置；
- 就绪检查和业务命令还会重复读取；
- `tauri dev` 二进制是 ad-hoc 签名，重新编译后代码哈希变化，“始终允许”不一定继续匹配。

### 解决

新增单一凭据条目：

```text
service = com.xiluolin.desktop
account = app_credentials_v1
value = JSON 编码的 AppCredentials
```

并增加进程级缓存：

```text
首次读取 Keychain
  → 解析全部凭据
  → 写入 OnceLock<Mutex<Option<AppCredentials>>>
  → 当前进程后续读取只访问内存
```

迁移策略：

1. 优先读取新的 bundled 条目；
2. 不存在时读取旧的三个分散条目；
3. 必要时从旧明文配置迁移；
4. 写入 bundled 条目；
5. 后续启动只读取 bundled 条目。

即使用户清空全部 Key，也保存空的 bundled 凭据作为明确状态，避免旧分散条目被再次迁移回来。

真实旧条目由用户授权后一次性删除，之后重新在设置页保存 Key。

## 问题五：文本整理延迟和 Provider 文案错误

### 现象

- 文本整理曾等待约 40 秒；
- 智谱请求失败时提示“OpenAI 文本整理请求失败”；
- 设置页即使选择智谱，保存逻辑仍校验 OpenAI 字段；
- 服务商下拉框写死 `GLM-4.7-Flash`、`Whisper` 等模型名。

### 实际请求对比

使用脱敏临时凭据对短文本做最小请求时观察到：

| 模型与参数 | 结果 |
|---|---|
| `glm-4.7` 默认思考 | 约 21 秒，产生大量 reasoning token |
| `glm-4.7` + `thinking.type=disabled` | 约 2.6 秒 |
| `glm-4-flash-250414` | 约 0.82 秒 |
| `glm-4.7-flash` 高峰期 | 可能直接返回 429 |

这说明“模型新”不等于“适合实时语音润色”。短文本链路更重视首 token 延迟、稳定性、输出长度和是否默认推理。

### 解决

智谱文本处理请求默认加入：

```json
{
  "thinking": { "type": "disabled" },
  "max_tokens": 512
}
```

并增加 12 秒全局超时。网络错误、HTTP 错误和响应解析错误都降级为原始 ASR 文本，不阻断投递。

Provider 相关体验同时调整：

- `polish_text_with_openai` 重命名为 `polish_text_with_provider`；
- 错误统一为“文本整理请求失败”等中性文案；
- 设置保存根据当前文本 Provider 校验对应 API Key、Base URL 和模型；
- 服务商显示为“智谱 AI”“OpenAI 兼容”“本地（离线）”；
- 模型名只在独立模型输入框中配置，切换服务商不覆盖用户输入。

## 问题六：ASR 和润色后应用直接退出

### 现象

流程日志停在文本整理成功，终端立即回到 shell：

```text
文本润色 ✅ 成功
process_voice_input 总耗时: ...
timekettle@... %
```

没有 Rust panic，也没有 `deliver_text` 返回错误，文本没有粘贴。

### 关键证据

检查 `~/Library/Logs/DiagnosticReports/xiluolin-*.ips` 后，四份最新报告均指向同一调用链：

```text
EXC_BREAKPOINT / SIGTRAP
_dispatch_assert_queue_fail
TSMCurrentKeyboardInputSourceRefCreate
enigo::key(Key::Unicode('v'))
send_paste_shortcut
clipboard_paste
Tokio blocking worker
```

### 根因

`clipboard_paste` 在 `spawn_blocking` 线程中执行。`enigo` 处理 `Key::Unicode('v')` 时会查询当前 macOS 输入源，将字符映射为键码。该 TSM API 要求特定主队列，后台线程调用触发 libdispatch 断言，系统以 SIGTRAP 直接终止进程。

这类原生崩溃不会转换成 Rust `Result` 或 panic，因此普通业务日志看不到错误。

### 解决

macOS 不再使用布局相关的 Unicode 字符映射，而是直接发送 ANSI V 的虚拟键码 `9`：

```rust
enigo.raw(9, Direction::Click)
```

Command 修饰键仍使用 Enigo 的修饰键 API。

Windows 和其他平台继续使用 `Key::Unicode('v')`，避免改变现有行为。

### 真实链路验证

修复后完整日志：

```text
ASR 成功
文本润色成功
[文本投递] 开始
[文本投递] 快捷键录音：准备恢复目标窗口并自动粘贴
[文本投递] 自动粘贴成功：restore_level=Window, clipboard_restored=true
```

应用和开发服务器继续运行，目标窗口收到文本，原剪贴板内容恢复。

## 问题七：结果日志的可观测性与隐私

### 决策

输出 ASR 和整理结果对调试有帮助，性能影响可忽略；但默认输出全文会把口述内容写入终端、截图、崩溃附件或 CI 日志。

因此默认只记录摘要：

```text
[结果] stage=ASR, provider=zhipu, model=glm-asr-2512, chars=24, fallback=false
[结果] stage=文本润色, provider=zhipu, model=glm-4.7, chars=31, fallback=false
```

需要本机排查正文时显式运行：

```bash
XILUOLIN_LOG_TEXT=1 pnpm tauri dev
```

全文日志只用于本机临时诊断，分享前必须脱敏。

## 问题八：录音状态反馈和基础交互

当前改动同步改善了状态悬浮窗：

- 透明、无边框、鼠标穿透；
- 置顶并显示在全部工作区；
- 屏幕顶部居中，支持带负坐标的副屏；
- 展示录音时长；
- 覆盖录音、识别、整理、输入、完成、失败六个阶段；
- 结束状态短暂停留后隐藏；
- 胶囊外围透明，避免大块窗口背景；
- 基础按钮、输入框、选择器、开关和 Tab 增加明确 hover/active 光标反馈。

macOS 透明窗口启用了 Tauri `macos-private-api` 和 `macOSPrivateApi`。这适合当前独立分发模式，不适用于 Mac App Store 沙盒分发。

## 具体改动

### 快捷键和会话

- `src-tauri/src/hotkey.rs`
  - 增加快捷键事件串行 gate。
- `src-tauri/src/pipeline.rs`
  - 错误路径取消 CaptureSession；
  - 空 ASR 结果改为明确提示；
  - 增加安全结果摘要和可选全文日志。

### Provider 请求

- `src-tauri/src/asr.rs`
  - 智谱 multipart 改用 reqwest；
  - 增加 MIME、超时和真实错误体。
- `src-tauri/src/text_polish.rs`
  - Provider 中性命名；
  - 智谱关闭思考；
  - 限制输出；
  - 增加超时和统一降级。
- `src-tauri/src/history_reprocessing.rs`
  - 使用统一文本 Provider 接口。

### 凭据

- `src-tauri/src/credentials.rs`
  - 三个分散条目合并为一个 bundled 条目；
  - 增加进程内缓存；
  - 保留旧凭据迁移。
- `src-tauri/src/data.rs`
  - 配置读写改用 bundled 凭据接口。

### 自动粘贴

- `src-tauri/src/output.rs`
  - macOS 使用物理 V 键码；
  - 增加投递阶段日志；
  - 保留焦点恢复、剪贴板恢复和复制兜底。

### 状态条和界面

- `src-tauri/src/indicator.rs`
- `public/indicator.html`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`
- `src/components/ui/*.tsx`
- `src/pages/SettingsPage.tsx`
- `src/components/settings/LocalAsrSettings.tsx`
- `src/main.tsx`

### 测试和文档

- `src-tauri/tests/openai_text_polish_provider.rs`
- `docs/usage-guide.md`
- `docs/troubleshooting.md`
- `docs/solution-design.md`
- `docs/retrospectives/`

## 补充修复：并行测试临时文件碰撞

首次执行 `pnpm check` 时，`local_asr_provider` 的业务断言已通过，但在清理 WAV 时出现 `No such file or directory`。测试原先只使用系统纳秒时间生成文件名；并行测试可能获得相同时间值，一个测试会删除另一个测试的文件。

测试临时文件名改为 `Uuid::new_v4()`，随后使用 `--test-threads=8` 单独回归通过。这个问题说明测试基础设施也需要并发安全，不能把高精度时间戳当作唯一 ID。

## 测试与验证

执行过：

```bash
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
pnpm build
```

关键结果：

- Rust 单元测试 28 项通过；
- 文本 Provider 集成测试 6 项通过；
- Voice Pipeline 测试 2 项通过；
- 智谱 ASR Provider 测试 4 项通过；
- 录音路径安全测试 7 项通过；
- TypeScript 类型检查通过；
- Vite 生产构建通过；
- macOS 真实快捷键链路完成录音、ASR、文本整理、窗口恢复、粘贴和剪贴板恢复；
- 修复后没有新增 `xiluolin` SIGTRAP 崩溃。

最终执行仓库总检查：

```bash
pnpm check
```

结果：TypeScript、Vite、rustfmt、`cargo check`、全部 Rust 单元/集成测试和 Doc-tests 通过；真实本地模型 smoke test 因依赖外部模型与 WAV fixture 按设计保持 ignored。

## 执行复盘

### 有效的排查方法

1. 用时间顺序日志确认快捷键是否存在事件乱序；
2. 用真实最小 API 请求把认证、模型和代码问题拆开；
3. 对云端请求保留非 2xx 响应体，不把传输错误当成服务端根因；
4. 进程无 panic 直接退出时检查 macOS `.ips` 崩溃报告；
5. 对模型延迟比较“相同输入、不同参数”，而不是凭模型名称猜测；
6. 对钥匙串问题同时检查条目数量、读取次数和代码签名身份；
7. 用一次真实跨应用链路验证代替只看单元测试。

### 无效或不完整的思路

- 只重启应用：会暂时清空会话，但不能解决竞态；
- 把 `Broken pipe` 归因于 48 kHz WAV：实际首先暴露的是认证响应丢失；
- 只按“始终允许”：ad-hoc 重新编译后授权身份可能变化，而且多个条目仍会弹多次；
- 只捕获 Rust panic：SIGTRAP 发生在系统框架断言，不经过 Rust 错误机制；
- 直接把全文结果写日志：虽然方便，但会破坏既有隐私基线；
- 在服务商标签中固定模型名：会制造“可配置但 UI 暗示固定”的冲突。

## 经验总结

- 桌面异步事件必须显式保证顺序，不能假设回调顺序等于任务完成顺序。
- 状态机的正常完成和异常清理应采用不同策略。
- HTTP 客户端错误可能掩盖服务端响应；可观测性是 Provider 集成的一部分。
- macOS AppKit、TSM、Accessibility 等 API 有严格线程约束，后台线程调用可能直接导致 native trap。
- 模型参数通常比更换模型更能影响实时交互延迟。
- 系统凭据安全不仅是“存在哪里”，还包括条目粒度、调用频率、迁移和签名稳定性。
- 调试日志应默认最小化正文，全文必须显式启用。
- 真正的端到端测试需要从用户正在输入的目标应用开始，并以文本实际出现为结束。

## 补充发现：智谱短音频时长边界

在停止一段约 82 秒的意外长录音时，智谱返回：

```text
http status: 400
code: 1214
transcriptions 文件时长限制为 0-30 秒
```

这不是客户端崩溃，但说明当前云端接口只适合短语音输入。现阶段通过明确错误和使用文档提示用户控制在 30 秒以内；未来如果产品需要长语音，应设计自动分段或切换到长音频/流式能力，而不是简单提高本地录音时长。

## 未完成事项

- 尚未在 Windows 真实桌面环境回归本次所有改动；
- `tauri dev` 仍是 ad-hoc 签名，重新编译后的系统权限稳定性不等同于正式签名包；
- 云端模型可用性、限流和价格会变化，不应把某个具体模型硬编码为长期推荐；
- 还可以增加自动化的 CaptureSession 并发回归测试和文本投递抽象测试；
- Mac App Store 分发需要重新评估透明窗口私有 API。

## 后续建议

1. 发布测试优先使用稳定 Apple Development 签名的 `.app`；
2. 保留 `glm-asr-2512` 等模型名为用户配置，不在供应商标签中绑定；
3. 新模型评估至少记录首 token、总延迟、失败率、输出质量和成本；
4. macOS 原生崩溃排查固定检查 `~/Library/Logs/DiagnosticReports`；
5. 新增 Provider 时复用中性错误、统一超时、结果摘要和原文降级约定；
6. 分享日志前检查 API Key、正文、录音路径和剪贴板内容。
