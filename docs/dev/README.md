# 历史开发记录

> **归档目录：** 本目录不再定义当前项目范围、Agent 行为或 Git 工作流。需要当前信息时，从 [`../README.md`](../README.md) 进入现行文档。

本目录保存 XiLuoLin 不同阶段的任务拆分、实现记录、验证结论、旧修复过程和候选方案，用于追踪技术演进。

## 内容分类

- `YYYY-MM-DD-*.md`：按时间记录的历史任务和阶段决策。
- [`2026-07-20-t036-macos-voice-pipeline-reliability.md`](./2026-07-20-t036-macos-voice-pipeline-reliability.md)：macOS 真实语音输入链路集中排查和修复记录。
- [`task-tracker.md`](./task-tracker.md)：旧任务状态快照，不再是当前工作的唯一事实源。
- [`task-doc-template.md`](./task-doc-template.md)：旧任务复盘模板，仅在需要补录历史记录时参考。
- [`archive/fixes/`](./archive/fixes/)：已完成或已被当前排查指南替代的修复记录。
- [`archive/plans/`](./archive/plans/)：未实施、部分实施或已过期的候选方案。
- [`archive/guides/`](./archive/guides/)：已经被当前文档吸收的旧指南。

## 当前文档入口

- 文档总览：[`docs/README.md`](../README.md)
- 产品方向：[`roadmap.md`](../roadmap.md)
- 产品需求：[`requirements-analysis.md`](../requirements-analysis.md)
- 技术方案：[`solution-design.md`](../solution-design.md)
- 使用与验证：[`usage-guide.md`](../usage-guide.md)
- 故障排查：[`troubleshooting.md`](../troubleshooting.md)

## 使用原则

历史文档可能包含比赛、demo、MVP、`dev` 分支、直接提交 `main`、旧代码路径或已经变化的配置方式。这些内容只代表当时背景：

- 不要求 Agent 在每次开发前读取本目录。
- 不要求每项改动更新 tracker 或创建任务复盘。
- 不从历史文档推断当前产品能力和发布流程。
- 有长期价值的结论应整理进现行文档，而不是继续扩写旧记录。
