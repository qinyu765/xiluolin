# 工程复盘与学习记录

本目录用于记录 XiLuoLin 从黑客松 MVP 向可长期维护的开源桌面语音输入助手演进过程中的关键问题、设计背景、实现方案、验证方法和经验教训。

与 `docs/dev/` 的区别：

- `docs/dev/` 保存按任务产生的历史实施记录，强调“当时做了什么”。
- `docs/retrospectives/` 提炼跨任务的背景、决策、因果关系和可复用经验，强调“为什么这样做、以后如何做得更好”。
- 当前产品事实仍以 [`README.md`](../../README.md)、[`requirements-analysis.md`](../requirements-analysis.md)、[`solution-design.md`](../solution-design.md) 和代码为准。

## 2026-07-17 现代化改造复盘

1. [`2026-07-17-modernization-overview.md`](./2026-07-17-modernization-overview.md)
   - 项目背景、路线调整、实施时间线和整体结果。
2. [`quality-and-security-baseline.md`](./quality-and-security-baseline.md)
   - TypeScript/Rust 质量门禁、CI、凭据存储、敏感日志和录音安全。
3. [`capture-session-and-cross-app-delivery.md`](./capture-session-and-cross-app-delivery.md)
   - CaptureSession、焦点快照、剪贴板恢复、状态悬浮窗和跨应用投递。
4. [`capture-history-and-reprocessing.md`](./capture-history-and-reprocessing.md)
   - 历史数据模型、录音保留、存储清理、试听、重新转写和重新整理。
5. [`local-asr-and-provider-strategy.md`](./local-asr-and-provider-strategy.md)
   - whisper-rs、模型管理、重采样、显式云端降级和真实离线验证。
6. [`collaboration-git-and-ci-lessons.md`](./collaboration-git-and-ci-lessons.md)
   - 并发工作区冲突、worktree 隔离、堆叠 PR、rebase 和 Windows CI。
7. [`future-work-and-checklists.md`](./future-work-and-checklists.md)
   - 遗留问题、发布前验证、后续演进建议和可复用检查清单。
8. [`macos-voice-input-reliability.md`](./macos-voice-input-reliability.md)
   - macOS 快捷键竞态、Provider 可观测性、Keychain 授权、文本延迟和自动粘贴 native crash 的完整复盘。

## 阅读建议

- 想快速了解全貌：先读“现代化改造总览”。
- 排查安全和数据问题：读“质量与安全基线”。
- 开发快捷键、悬浮窗或自动粘贴：读“CaptureSession 与跨应用投递”。
- 修改历史表或录音生命周期：读“Capture 历史与重新处理”。
- 修改本地模型或 Provider：读“本地 ASR 与 Provider 策略”。
- 处理多 Agent、分支和 CI：读“协作、Git 与 CI 经验”。
