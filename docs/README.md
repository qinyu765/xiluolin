# XiLuoLin 文档导航

本目录只维护长期有效的产品、架构、使用和设计说明。单次任务的实施过程、旧修复记录和未落地方案统一保存在 [`dev/`](./dev/) 中，不作为当前行为或实现的事实来源。

## 推荐阅读顺序

1. [`../README.md`](../README.md)：项目定位、当前状态和本地开发入口。
2. [`roadmap.md`](./roadmap.md)：长期方向、当前优先级和非核心范围。
3. [`requirements-analysis.md`](./requirements-analysis.md)：产品场景、能力边界和验收目标。
4. [`solution-design.md`](./solution-design.md)：当前架构、模块、数据模型和关键流程。
5. [`usage-guide.md`](./usage-guide.md)：安装、Provider 配置、使用和验证步骤。
6. [`troubleshooting.md`](./troubleshooting.md)：录音、快捷键、Provider 和跨应用输出排查。

## 当前文档

| 文档 | 唯一职责 | 主要读者 |
|---|---|---|
| [`roadmap.md`](./roadmap.md) | 项目方向和优先级 | 用户、维护者、贡献者 |
| [`requirements-analysis.md`](./requirements-analysis.md) | 产品需求、范围和验收标准 | 产品与开发人员 |
| [`solution-design.md`](./solution-design.md) | 当前技术架构和设计约束 | 开发人员 |
| [`usage-guide.md`](./usage-guide.md) | 配置、使用和端到端验证 | 用户与测试人员 |
| [`troubleshooting.md`](./troubleshooting.md) | 当前故障定位和恢复方法 | 用户与开发人员 |
| [`design/ui-style.md`](./design/ui-style.md) | 当前 UI 视觉规范 | 设计与前端开发人员 |
| [`hotword-recommendations.md`](./hotword-recommendations.md) | 可选热词参考列表 | 用户与测试人员 |
| [`retrospectives/README.md`](./retrospectives/README.md) | 工程复盘、决策背景和可复用经验 | 维护者与贡献者 |

## 工程复盘

[`retrospectives/`](./retrospectives/) 保存跨任务的背景、决策、问题根因、解决过程和经验总结。它们用于解释当前架构为何形成，不替代当前需求和方案文档。

- [`2026-07-17-modernization-overview.md`](./retrospectives/2026-07-17-modernization-overview.md)：现代化改造总览、时间线和阶段结果。
- [`quality-and-security-baseline.md`](./retrospectives/quality-and-security-baseline.md)：质量门禁、凭据、日志和录音安全。
- [`capture-session-and-cross-app-delivery.md`](./retrospectives/capture-session-and-cross-app-delivery.md)：CaptureSession、焦点和跨应用投递。
- [`capture-history-and-reprocessing.md`](./retrospectives/capture-history-and-reprocessing.md)：历史模型、录音留存和重新处理。
- [`local-asr-and-provider-strategy.md`](./retrospectives/local-asr-and-provider-strategy.md)：本地 ASR、模型管理和降级策略。
- [`collaboration-git-and-ci-lessons.md`](./retrospectives/collaboration-git-and-ci-lessons.md)：多 Agent、worktree、堆叠 PR 和 CI。
- [`future-work-and-checklists.md`](./retrospectives/future-work-and-checklists.md)：遗留事项和通用检查清单。

## 历史归档

[`dev/`](./dev/) 保存不同阶段的任务记录、旧流程、修复过程和候选方案：

- `dev/YYYY-MM-DD-*.md`：按时间记录的历史开发任务。
- `dev/archive/fixes/`：已完成或已被当前排查指南替代的修复记录。
- `dev/archive/plans/`：未实施、部分实施或已过期的候选方案。
- `dev/archive/guides/`：已被当前文档吸收的旧指南。

历史文档可能包含旧分支、比赛、demo、MVP、旧代码路径或已经变化的配置方式。需要确认当前行为时，以代码和本页列出的当前文档为准。

## 维护原则

- 优先更新已有的当前文档，不为单次修复重复创建长期指南。
- 只有独立、长期维护的主题才新增当前文档，并在本页登记职责。
- 用户行为和配置写入 `usage-guide.md`，技术结构写入 `solution-design.md`，问题恢复方法写入 `troubleshooting.md`。
- 阶段性实施记录放入 `dev/`；旧内容应归档，而不是继续占用当前入口。
- 文档移动后必须更新相对链接并运行 Markdown 链接检查。

## Agent 软链接说明

仓库使用真正的 Git 软链接，让 Claude 与其他 Agent 共用同一套规则：

```text
CLAUDE.md -> AGENTS.md
.claude -> .agents
```

Windows 检出前需要启用 Git symlink 支持，并确保系统允许创建符号链接；否则 Git 可能把链接检出为普通文本文件。
