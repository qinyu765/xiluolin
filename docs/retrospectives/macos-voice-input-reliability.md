# macOS 语音输入链路可靠性复盘

## 1. 为什么这是一个“系统问题”

表面症状分散在快捷键、录音、云端 API、钥匙串、文本模型和自动粘贴，但它们都属于同一条用户闭环：

```text
用户按快捷键
  → 应用必须准确理解开始/停止
  → 录音会话必须可恢复
  → Provider 必须快速、可观察
  → 结果必须回到原窗口
  → 应用必须继续存活
```

任何一环失败，用户感知都只是“语音输入不能用”。因此排查不能停在某个模块返回成功，而要验证最终文本是否真正进入目标应用。

详细实施时间线见 [`../dev/2026-07-20-t036-macos-voice-pipeline-reliability.md`](../dev/2026-07-20-t036-macos-voice-pipeline-reliability.md)。

## 2. 可靠链路的四层状态

### 2.1 输入事件状态

Pressed 和 Released 是有序事件，但异步处理可能乱序。需要独立事件 gate 保证：

```text
Pressed 完成状态写入 → Released 才开始读取状态
```

### 2.2 CaptureSession 状态

正常路径使用严格状态机：

```text
Recording → Transcribing → Refining → Delivering → Completed
```

异常路径必须允许强制清理：

```text
任意处理中状态 → cancel(session_id) → 可开始下一条输入
```

严格状态机负责发现程序错误，强制取消负责恢复用户能力，两者不能混为一谈。

### 2.3 Provider 状态

Provider 调用至少应区分：

- 本地验证失败；
- 网络传输失败；
- HTTP 非 2xx；
- 响应解析失败；
- 成功但内容为空；
- 成功并使用降级。

只记录“请求失败”会让认证、限流、服务拥堵和客户端 bug 混在一起。

### 2.4 桌面投递状态

处理成功不代表输入成功。投递还包含：

```text
保存新文本到剪贴板
  → 恢复原应用/窗口
  → 模拟 Command+V
  → 等待目标应用消费
  → 恢复原剪贴板
  → 完成 CaptureSession
```

因此日志必须覆盖 `deliver_text`，而不能在文本模型返回后结束。

## 3. macOS 原生 API 的线程边界

本次最隐蔽的问题是 `enigo::key(Key::Unicode('v'))`。

它看起来只是生成一个键盘事件，内部却需要查询当前输入法，把 Unicode 字符映射到物理键。macOS TSM API 对执行队列有要求；从 Tokio blocking worker 调用时，系统触发 `_dispatch_assert_queue_fail`，随后发送 SIGTRAP。

重要结论：

- Rust 的类型安全不能保证外部系统框架的线程契约；
- `spawn_blocking` 只解决“不要阻塞异步 runtime”，不代表其中可以调用所有 GUI API；
- 进程被系统信号杀死时，`Result`、panic hook 和普通错误日志都可能没有机会运行；
- 对固定快捷键，直接使用平台虚拟键码可以避免不必要的输入法查询。

以后在 macOS 上遇到“没有 panic、终端直接退出”时，应立即检查：

```text
~/Library/Logs/DiagnosticReports/*.ips
```

重点阅读：

- `exception.type`；
- `termination.signal`；
- `faultingThread`；
- 故障线程的前 10～20 个 frame；
- 是否出现 AppKit、TSM、Accessibility、CoreGraphics 或 libdispatch 断言。

## 4. 云端 Provider 的可观测性

`Broken pipe` 不是根因，只是客户端观察到的传输结果。远端可能已经返回 401，但客户端仍在写 multipart body，最终只暴露写端断开。

Provider 适配器应做到：

1. 显式超时；
2. 非 2xx 时读取响应体；
3. 错误文案使用实际 Provider 或中性能力名称；
4. 结果为空单独处理；
5. 网络/解析错误时保留可用原文；
6. 不在日志打印凭据和正文。

这使排查路径从“猜音频格式”变成：

```text
401 → 凭据问题
402 → 账户额度/计费问题
429 → 限流或模型拥堵
5xx → Provider 服务问题
timeout → 降级为原文并继续投递
```

## 5. 实时文本整理的模型选择原则

语音润色不是复杂 Agent 任务。默认评估指标应该是：

1. 首 token 延迟；
2. 总延迟；
3. 失败率和高峰期限流；
4. 是否默认产生 reasoning；
5. 短中文改写质量；
6. 单次成本；
7. 是否支持关闭思考。

本次实测说明，同一个 `glm-4.7` 只改变 `thinking.type` 就能从约 21 秒降到约 2.6 秒。对于实时输入，参数选择可能比升级到更大的模型更重要。

模型名保持用户可配置，供应商下拉框不绑定具体型号。新模型上线时，应通过固定语料基准测试，而不是直接替换默认值。

供应商的当前模型和参数能力应以官方文档为准：

- 智谱模型概览：<https://docs.bigmodel.cn/cn/guide/start/model-overview>
- 智谱思考模式：<https://docs.bigmodel.cn/cn/guide/capabilities/thinking-mode>
- 智谱语音转文字：<https://docs.bigmodel.cn/cn/guide/models/sound-and-video/glm-asr-2512>

## 6. Keychain：安全存储不等于良好体验

把三个 Key 分开存储在 Keychain 在安全性上没有明显错误，但每个条目都可能独立授权。再叠加旧服务名、重复读取和 ad-hoc 签名，一次启动可能出现多个弹窗。

更适合当前应用的结构：

```text
Keychain
  └─ com.xiluolin.desktop / app_credentials_v1
       └─ AppCredentials JSON

Process cache
  └─ OnceLock<Mutex<Option<AppCredentials>>>
```

设计原则：

- 凭据仍只存在系统安全存储和当前进程内存；
- 单次读取恢复全部 Provider Key；
- 当前进程不重复访问 Keychain；
- 迁移顺序必须先安全写入，再清理明文；
- 空凭据也是一种明确状态，防止旧值复活；
- 正式签名身份比 ad-hoc 代码哈希更适合长期授权。

## 7. 日志的两种模式

### 默认模式

记录：

- 阶段；
- Provider；
- 模型；
- 时长；
- 字符数；
- fallback；
- 不含用户正文的错误。

### 显式正文模式

```bash
XILUOLIN_LOG_TEXT=1 pnpm tauri dev
```

只用于本机短时排查。正文可能包含姓名、客户信息、代码、账号或尚未公开的业务内容，不应进入截图、Issue 或 CI。

## 8. 测试金字塔需要补上真实桌面层

### 自动化层

适合验证：

- 状态机；
- Provider 请求体；
- HTTP 错误；
- 凭据迁移业务逻辑；
- 音频路径；
- 多屏坐标；
- 固定 macOS V 键码。

### 桌面 smoke test

必须验证：

1. 在外部编辑器中放置光标；
2. 触发全局快捷键；
3. 说一段短文本；
4. 结束录音；
5. 观察状态条阶段；
6. 确认文本进入原窗口；
7. 确认原剪贴板恢复；
8. 再次录音，确认会话已释放；
9. 确认没有新增 `.ips` 崩溃报告。

只验证 ASR/LLM 返回值，会漏掉本次最严重的 native crash。

## 9. 可复用排查清单

### 快捷键后不能再次录音

- 检查 Pressed/Released 日志顺序；
- 检查启动完成是否晚于 Released；
- 检查 CaptureSession 当前状态；
- 检查所有错误路径是否 cancel；
- 检查热更新时是否在录音中重新注册快捷键。

### Provider 返回模糊 I/O 错误

- 用最小请求直接访问同一 URL；
- 记录状态码和脱敏响应体；
- 检查客户端是否在上传中被远端提前拒绝；
- 区分认证、额度、限流和服务拥堵。

### 文本整理太慢

- 比较思考开关；
- 限制最大输出；
- 设置总超时；
- 固定短文本做基准；
- 记录高峰期 429；
- 失败时立即使用原文。

### 应用无错误直接退出

- 查看 DiagnosticReports；
- 找 faulting thread；
- 检查 macOS 系统 API 的线程要求；
- 避免在 worker 上调用布局、输入法或 GUI 查询；
- 真实验证修复后没有新 crash report。

### Keychain 重复弹窗

- 统计服务名和账户数量；
- 统计一次启动读取次数；
- 检查是否存在旧服务名；
- 检查开发二进制签名；
- 合并条目并使用进程缓存。

## 10. 最终结论

可靠语音输入不是“录到音并拿到文本”，而是一个具有严格时序、状态恢复、Provider 降级、系统权限、窗口焦点和原生输入约束的桌面事务。

本次最重要的工程收获是：

> 从用户光标所在位置开始排查，以最终文本进入同一位置且应用仍可继续工作为完成标准。
