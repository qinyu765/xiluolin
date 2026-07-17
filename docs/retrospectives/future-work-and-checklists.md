# 后续工作、验收清单与演进建议

## 1. 文档目的

现代化改造完成了工程和产品骨架，但仍有一部分能力只能通过真实桌面环境、真实 Provider 和长期使用验证。

本文把遗留事项分为：

- 必须尽快验证；
- 可靠性改进；
- 性能和发布；
- 可选产品演进；
- 每次改动可复用的检查清单。

## 2. 必须尽快完成的真实环境验证

### 2.1 Windows 跨应用输入

测试应用：

- Notepad；
- VS Code；
- Chrome/Edge 输入框；
- 微信/企业微信或其他聊天工具；
- PowerShell/Windows Terminal；
- 提升权限运行的应用。

验证：

- [ ] 快捷键开始时目标窗口正确；
- [ ] 状态窗不抢焦点；
- [ ] 处理期间切换窗口后仍回到原目标；
- [ ] 目标关闭后结果进入剪贴板；
- [ ] 提升权限窗口被 UIPI 阻止时有明确兜底；
- [ ] 粘贴成功后原文本剪贴板恢复；
- [ ] 原图片剪贴板恢复；
- [ ] 连续快速录音不会被旧 hide timer 影响；
- [ ] 处理中重复按快捷键不会取消旧 session。

### 2.2 macOS

- [ ] 麦克风权限；
- [ ] 快捷键注册；
- [ ] Accessibility 权限；
- [ ] 状态窗不抢焦点；
- [ ] Cmd+V 自动输入；
- [ ] 失败时复制兜底；
- [ ] Keychain 读写；
- [ ] 应用签名/打包后行为。

当前 macOS 尚未恢复录音开始时的原目标窗口，应在 UI 中明确平台差异。

### 2.3 真实云端 Provider

智谱 ASR：

- [ ] 正常 WAV；
- [ ] 无效 Key；
- [ ] 网络断开；
- [ ] 超时；
- [ ] 限额；
- [ ] 错误响应不泄露敏感数据。

OpenAI Whisper：

- [ ] 设置页字段与实际调用一致；
- [ ] 模型名切换；
- [ ] Base URL 兼容；
- [ ] 历史记录实际 Provider 正确。

文本 Provider：

- [ ] OpenAI-compatible；
- [ ] 智谱；
- [ ] 请求失败回退原文；
- [ ] used_fallback 标记正确；
- [ ] 不同人格输出可区分。

### 2.4 本地 ASR

- [ ] 设置页模型下载；
- [ ] 下载中断后无损坏最终文件；
- [ ] 下载进度；
- [ ] 模型验证；
- [ ] 模型删除；
- [ ] 无网络转写；
- [ ] 48 kHz 麦克风重采样；
- [ ] 中文短语音；
- [ ] 中英混合和代码词；
- [ ] 本地失败且 fallback 关闭时无网络；
- [ ] fallback 开启时实际 Provider 记录正确；
- [ ] Windows CPU 延迟和内存占用。

## 3. PR 与分支状态清理

当前代码已经进入 `main`，但部分堆叠 PR 仍可能显示 Open。

处理步骤：

1. 比较每个 PR head 和 main：

```bash
git fetch origin
git diff origin/main...origin/feat/input-readiness
git diff origin/main...origin/feat/capture-retention
git diff origin/main...origin/feat/capture-reprocessing
git diff origin/main...origin/feat/local-asr
```

2. 确认 main 已包含全部功能；
3. 在 PR 留言对应 main commit；
4. 关闭已落地 PR；
5. 删除不再使用的远端 feature 分支；
6. 删除本地 backup 分支和临时 worktree；
7. 更新路线图和发布说明。

## 4. CI 改进

### 4.1 Action runtime

当前有 Node 20 runtime 弃用警告，应升级：

- actions/checkout；
- actions/setup-node；
- 其他依赖旧 Node runtime 的 action。

### 4.2 Rust 缓存

whisper.cpp native build 使 Windows CI 接近 10 分钟。

建议缓存：

- Cargo registry；
- Cargo git；
- `src-tauri/target`；
- 按 Cargo.lock、target triple、profile 生成 key。

### 4.3 Job 拆分

```text
frontend-quality
rust-windows
rust-macos（可选）
docs-links
```

优点：

- 快速看到前端错误；
- Rust job 可独立缓存；
- 平台问题更清晰；
- 可并行执行。

### 4.4 Workflow dispatch

增加：

```yaml
workflow_dispatch:
  inputs:
    ref:
```

支持对堆叠分支手动运行完整 CI，避免修改 PR base 或创建空提交。

### 4.5 Concurrency

同一分支新提交时取消旧 run：

```yaml
concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true
```

## 5. 可靠性改进

### 5.1 CaptureSession 超时

处理异常或前端崩溃可能留下 session。

建议：

- session created_at；
- 最大录音时长；
- 最大 processing 时长；
- 应用启动时清理 stale session；
- 用户“取消当前输入”命令。

### 5.2 录音流生命周期

当前录音流通过泄漏对象保持活动，后续应改为显式保存在 RecordingState 并在停止时 drop。

好处：

- 不泄漏资源；
- 可以取消；
- 可以处理设备断开；
- 状态更清晰。

### 5.3 历史写入失败

当前历史改为同步，以获得 history ID。可以进一步：

- SQLite transaction；
- 写入失败时仍允许输出；
- UI 明确“文本成功、历史保存失败”；
- 后台重试但不阻塞投递。

### 5.4 自动粘贴确认

enigo 成功只表示事件发送成功，不表示目标应用消费。

可研究：

- Windows UI Automation 检测文本控件；
- 粘贴前后控件值变化；
- 目标应用适配；
- 不支持应用名单。

## 6. 数据与隐私改进

### 6.1 模型 checksum

模型下载应增加：

- SHA256；
- 可信来源列表；
- 下载完成校验；
- 校验失败自动删除。

### 6.2 录音保留策略

新增：

- 最大保留天数；
- 最大空间；
- 收藏不清理；
- 自动清理预览；
- 手动选择存储目录；
- 可选本地加密。

### 6.3 历史版本

重新处理目前覆盖同一记录。后续可以增加 revision：

- 保留多个模型结果；
- 对比人格；
- 回滚；
- 标记当前版本。

### 6.4 完全本地模式

当前本地 ASR 后仍可能调用云端文本 Provider。

真正的完全本地模式需要：

- 本地文本模型；
- 清晰的网络禁用状态；
- Provider 网络审计；
- UI 隐私模式；
- 离线端到端测试。

## 7. 本地 ASR 性能改进

### 7.1 重采样

当前线性插值可以替换为专业带限重采样库。

验证指标：

- 中文 CER；
- 代码词识别；
- CPU 时间；
- 不同采样率。

### 7.2 VAD

加入 Voice Activity Detection：

- 去除前后静音；
- 拒绝纯静音；
- 减少推理时长；
- 改善短语音体验。

### 7.3 模型选择

设置页可支持：

- tiny：快速、低内存；
- base Q5_1：默认；
- small：更高质量；
- 各模型大小和预估内存。

### 7.4 硬件加速

评估：

- macOS Metal；
- Windows CUDA；
- Windows Vulkan；
- AMD/Intel 兼容性。

feature 必须按目标平台配置，避免一个平台的 feature 破坏另一个平台编译。

## 8. UI 和产品体验

### 8.1 首页输入入口

QuickStartCard 当前隐藏。需要决定：

- 恢复大型录音按钮；
- 更轻量的键盘提示和测试按钮；
- 首次启动引导；
- 快捷键冲突时的备用入口。

### 8.2 就绪检查动作化

当前展示状态和说明，后续可以：

- 跳转系统设置；
- 测试麦克风；
- 测试 Provider；
- 测试快捷键；
- 测试粘贴；
- 一键修复默认配置。

### 8.3 Capture 播放器

从基础播放升级：

- 波形；
- 进度；
- 暂停；
- 倍速；
- 音频下载；
- 重新处理版本对比。

## 9. 发布准备

### Windows

- [ ] MSI/NSIS 安装包；
- [ ] WebView2 检查；
- [ ] Credential Manager；
- [ ] 麦克风权限；
- [ ] 普通/管理员窗口；
- [ ] SmartScreen 签名；
- [ ] 自动更新策略。

### macOS

- [ ] `.app`/DMG；
- [ ] 签名；
- [ ] notarization；
- [ ] 麦克风描述；
- [ ] Accessibility 引导；
- [ ] Keychain；
- [ ] Apple Silicon/Intel 策略。

### GitHub Release

- [ ] 版本号；
- [ ] CHANGELOG；
- [ ] 安装包；
- [ ] checksum；
- [ ] 已知限制；
- [ ] 隐私说明；
- [ ] 升级/迁移说明。

## 10. 每次任务通用检查清单

### 开始前

- [ ] 工作区干净或明确文件归属；
- [ ] main 与 origin/main 同步；
- [ ] 新建短生命周期分支/worktree；
- [ ] 读取当前 AGENTS.md；
- [ ] 明确验收标准。

### 实现中

- [ ] 不记录用户敏感数据；
- [ ] 配置前后端同步；
- [ ] 失败路径有清理；
- [ ] 默认隐私策略不被绕过；
- [ ] 平台差异有条件编译；
- [ ] 文档和代码同步。

### 验证

- [ ] `pnpm check`；
- [ ] 专项测试；
- [ ] `git diff --check`；
- [ ] 敏感日志扫描；
- [ ] 构建产物检查；
- [ ] Windows CI；
- [ ] 必要的真实服务/模型 smoke test。

### 发布

- [ ] commit 单一主题；
- [ ] PR 描述包含问题、方案、验证和影响；
- [ ] CI 通过；
- [ ] PR/main 状态对账；
- [ ] 合并后清理分支/worktree；
- [ ] 更新 CHANGELOG/roadmap（如需要）。

## 11. 建议优先级

### P0

1. 清理已进入 main 但仍 Open 的堆叠 PR。
2. Windows 真实跨应用粘贴测试。
3. 本地模型设置页下载/删除真实验证。
4. 修复 GitHub Actions Node runtime 警告。
5. 增加 CI cache。

### P1

1. macOS 焦点快照和权限检测。
2. 首页备用录音入口。
3. Provider 连接测试。
4. 录音流显式生命周期。
5. Capture 搜索和版本历史。

### P2

1. 模型选择和 GPU 加速。
2. 本地文本模型。
3. 流式转写。
4. 高级播放器和导出。
