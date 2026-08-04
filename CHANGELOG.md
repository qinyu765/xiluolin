# Changelog

XiLuoLin 遵循 [Semantic Versioning](https://semver.org/)，本文件记录面向用户的版本变化。

## [0.1.0-beta.1] - Unreleased

### 新增

- 提供 macOS 13+ Apple Silicon ad-hoc 签名 DMG 和 Windows 10/11 x64 NSIS 安装包。
- 建立 Git tag 驱动的双平台构建、依赖安全审计、校验和与 Draft Pre-release 流程。
- 提供智谱、OpenAI-compatible 和本地 Whisper ASR，支持人格整理、热词、历史、统计和跨应用文本投递。

### 安全

- API Key 使用操作系统凭据库存储，应用录音默认在处理完成后清理。
- 发布门禁检查 npm/Rust 依赖漏洞和仓库敏感信息。

### 已知限制

- macOS 安装包未经 Apple 公证，首次启动需要用户手动允许。
- Windows 安装包未签名，可能触发 Microsoft Defender SmartScreen。
- 不支持 Intel Mac、Windows ARM64、Linux、应用商店和应用内自动更新。
- 首页录音/上传入口暂未开放，全局快捷键是主要输入入口。

[0.1.0-beta.1]: https://github.com/qinyu765/xiluolin/releases/tag/v0.1.0-beta.1
