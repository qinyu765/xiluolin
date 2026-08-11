# Changelog

XiLuoLin 遵循 [Semantic Versioning](https://semver.org/)，本文件记录面向用户的版本变化。

## [0.1.0] - 2026-08-11

### 修复

- 升级 `cpal` 至 `0.17.3`，在停止、取消、超时、通道关闭和错误退出路径统一暂停并释放录音流，避免 macOS 继续占用麦克风。
- 默认麦克风配置直接使用系统默认设备对象；明确选择的设备不可用时回退到真正的系统默认设备。
- 自动输入失败时保留完整结果并打开可恢复结果窗口，支持再次复制、失败原因提示和 `Esc` 关闭；正常自动粘贴路径保持不变。

### 识别与体验

- 明确热词是全局 ASR 偏置，原文听写也会受到热词竞争影响；“原文听写”保留 ASR 原始措辞，仅规范空白，不调用文本模型纠错。
- 补充录音释放、失败投递、热词和原文听写的回归测试与生成绑定一致性检查。

### 发布

- 发布 macOS 13+ Apple Silicon ad-hoc、未公证 DMG，以及 Windows 10/11 x64 未签名 NSIS 安装包。
- 稳定版 Release 自动执行前端、Rust、依赖安全、密钥扫描、双平台构建和 SHA256 校验文件生成。

### 已知限制

- macOS 安装包未经 Apple 公证，首次启动需要用户手动允许；Windows 安装包未签名，可能触发 Microsoft Defender SmartScreen。
- 不支持 Intel Mac、Windows ARM64、Linux、应用商店和应用内自动更新。
- 首页录音/上传入口暂未开放，全局快捷键是主要输入入口；第一阶段不支持实时麦克风流或边说边出字。

## [0.1.0-beta.1] - 2026-08-04

### 新增

- 提供 macOS 13+ Apple Silicon ad-hoc 签名 DMG 和 Windows 10/11 x64 NSIS 安装包。
- 建立 Git tag 驱动的双平台构建、依赖安全审计、校验和与 Draft Pre-release 流程。
- 提供智谱、OpenAI-compatible 和本地 Whisper ASR，支持人格整理、热词、历史、统计和跨应用文本投递。
- 智谱原生接收前 100 个启用热词；OpenAI 与本地 Whisper 使用软提示，并新增跳过文本模型的“原文听写”。
- macOS 可显式开启按住 Fn 录音；录音在 25 秒提示、28 秒自动停止，短按 Fn 会取消暂存录音。
- 提供 80 条 ASR 基准录制模板和 CER、热词召回、标点 F1、端到端延迟评测工具。

### 安全

- API Key 使用操作系统凭据库存储，应用录音默认在处理完成后清理。
- 发布门禁检查 npm/Rust 依赖漏洞和仓库敏感信息。

### 已知限制

- macOS 安装包未经 Apple 公证，首次启动需要用户手动允许。
- Windows 安装包未签名，可能触发 Microsoft Defender SmartScreen。
- 不支持 Intel Mac、Windows ARM64、Linux、应用商店和应用内自动更新。
- 首页录音/上传入口暂未开放，全局快捷键是主要输入入口。
- 第一阶段仅支持停止录音后的完整结果，不支持实时麦克风流或边说边出字。

[0.1.0]: https://github.com/qinyu765/xiluolin/releases/tag/v0.1.0
[0.1.0-beta.1]: https://github.com/qinyu765/xiluolin/releases/tag/v0.1.0-beta.1
