# T010 补齐演示和评审文档

## 任务目标

同步项目文档到当前 MVP 和 UI 重构状态，补齐面向评审的 README 说明、演示复现路径和演示脚本。

## 实际改动

- 更新 `README.md` 的项目状态，说明核心能力、当前界面结构和待验证项。
- 更新 `README.md` 的演示复现流程，明确本地运行、模型配置、人格、热词、首页历史和统计的验证路径。
- 补全 `README.md` 中新增前端和 Rust 依赖的用途说明。
- 新增 `docs/demo-script.md`，用于评审讲解和录屏准备。
- 更新 `docs/solution-design.md` 的 UI 信息架构，反映当前左侧导航、首页、人格页、热词页和设置页结构。
- 继续同步当前实际限制：`QuickStartCard` 在首页被注释隐藏，前端尚未监听快捷键录音完成事件。
- 记录新增录音指示器窗口：`src-tauri/src/indicator.rs`、根目录 `indicator.html` 和 `src-tauri/indicator.html` 已加入；两份 HTML 内容一致，生产打包资源配置仍需验证。
- 记录配置弹窗风险：`AppSettingsDialog` 仍存在且引用 `recording_mode`，但当前 `AppConfig` 类型未包含该字段，主入口也未使用该弹窗。
- 更新 `docs/demo-script.md` 和 T028 验证清单，避免把未接线的语音输入入口描述成当前可演示能力。
- 更新 `docs/dev/task-tracker.md` 中 T010 的状态和文档链接。

## 为什么这么做

旧 README 仍停留在较早阶段，把录音模块、快捷键、主界面和设置页写成“进行中”。当前仓库已经完成大部分 MVP 模块和 UI 重构任务，但 T028 端到端验证仍未完成。文档需要同时体现已完成能力和待验证边界，避免评审或后续开发基于过期状态做判断。

## 涉及文件

- `README.md`
- `docs/demo-script.md`
- `docs/solution-design.md`
- `docs/dev/task-tracker.md`
- `docs/dev/2026-05-25-t010-demo-docs-sync.md`

## 测试与验证

- 检查 README 中不再保留旧的“进行中：录音模块、全局快捷键、主界面...”状态描述。
- 检查新增依赖说明覆盖 `package.json` 和 `src-tauri/Cargo.toml` 中当前使用的关键依赖。
- 检查演示脚本明确了 T028、真实 API smoke test 和快捷键录音联调仍待验证。

## 执行复盘

文档同步选择保守表述：已实现的模块写入“代码层已完成”，当前可见界面单独说明，未接线和未做端到端真实验证的能力写入“待完成验证”。这样可以保持项目对评委可解释，也避免把当前 UI 重构后的未验证路径写成稳定交付。

## 未完成事项

- T028 UI 重构端到端验证尚未执行。
- 真实智谱 GLM-ASR-2512 和 OpenAI Responses API smoke test 尚未记录。
- 快捷键录音触发后的桌面端完整联调尚未完成（前端事件监听已完成，待配置 API Key 后手动测试）。
- 首页可见语音输入入口尚未恢复或替代。
- 录音指示器生产打包路径尚未验证，根目录 `indicator.html` 已删除，仅保留 `src-tauri/indicator.html`。
- `AppSettingsDialog` 与当前配置类型不一致，后续需要清理或修正。

## 后续建议

- 执行 T028 验证后，把验证结果补入 `docs/dev/2026-05-25-t028-ui-e2e-verification.md`。
- 若通过真实服务 smoke test，更新 README 的当前验证状态和演示脚本。
- 稳定后准备 `dev -> main` 的阶段性 PR 描述。
