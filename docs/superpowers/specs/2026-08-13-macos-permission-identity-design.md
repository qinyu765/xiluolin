# macOS 权限身份与本机清理设计

## 背景

XiLuoLin 当前通过 Tauri 的 `signingIdentity: "-"` 生成 ad-hoc 签名。macOS 使用应用的 designated requirement（DR）识别麦克风、辅助功能等隐私权限；ad-hoc 签名的 DR 与具体构建绑定，因此新构建无法稳定继承旧构建的授权。

本机复现结果为：系统设置中 XiLuoLin 的麦克风和辅助功能开关均为开启，但当前集成构建分别读取为 `not_determined` 和未授权；请求麦克风后没有出现系统提示，并立即变为 `denied`。LaunchServices 还记录了 15 个同名应用位置，其中多数是已卸载的临时 DMG 路径。

## 目标

- 本机开发构建使用同一个个人 Apple Development 身份，使权限在连续构建间保持稳定。
- 不使用公司开发者身份，不提交个人证书信息、私钥或本机证书哈希。
- 清理失效的 LaunchServices 注册和当前测试 DMG，仅保留 `/Applications/XiLuoLin.app` 作为体验入口。
- 重置 XiLuoLin 的麦克风和辅助功能授权，重新完成一次授权并验证录音与自动粘贴。
- 不修改暗色模式，不删除源码、worktree、应用数据、历史、模型或录音。

## 设计

### 签名策略

仓库配置不绑定任何个人或公司证书。macOS 本机构建通过 `APPLE_SIGNING_IDENTITY` 注入当前钥匙串中的个人 Apple Development 身份；构建脚本和文档只描述环境变量用法，不记录身份值。

当前 Mac 的钥匙串已经包含可用的个人 Apple Development 证书及私钥，因此系统设置登录了哪个 Apple 账号、常用 Apple 账号位于哪台 Mac，都不影响当前机器执行本地签名。免费个人开发者证书只用于本机开发体验，不承诺 Developer ID 公证或面向其他用户分发。

证书到期、撤销、重新签发或换 Mac 后，DR 可能变化，届时允许再次重置并授权权限。

### 构建与安装

- 将仓库中的强制 ad-hoc 配置改为默认不指定签名身份。
- CI 或未配置证书的开发环境继续使用其现有构建策略；本机体验构建显式提供 `APPLE_SIGNING_IDENTITY`。
- 构建完成后，将同一稳定签名的 `.app` 安装到 `/Applications/XiLuoLin.app`。
- 验证 `codesign -d -r -` 的 DR 不再是仅包含具体 `cdhash` 的 ad-hoc 身份。

### 权限与清理

- 退出正在运行的 XiLuoLin。
- 推出当前挂载的 XiLuoLin DMG。
- 注销 LaunchServices 中不存在的 XiLuoLin 路径；不删除任何构建产物或 worktree。
- 使用 `tccutil reset Microphone com.xiluolin.desktop` 与 `tccutil reset Accessibility com.xiluolin.desktop` 清除冲突授权。
- 启动 `/Applications/XiLuoLin.app`，分别请求麦克风和辅助功能权限。
- 重新读取就绪状态，并进行一次短录音和一次跨应用自动粘贴验证。

## 测试

- 配置测试：macOS bundle 配置不再强制使用 ad-hoc 身份。
- 文档检查：本机构建说明包含环境变量、免费账号限制和重新授权条件，且不含证书值。
- 签名检查：安装包具有 Apple Development DR，连续两次构建满足同一签名要求。
- UI 检查：麦克风和自动粘贴就绪卡在授权后显示已就绪。
- 功能检查：短录音能生成历史记录；自动粘贴能向测试文本框发送 Command+V，失败时仍保留剪贴板降级。
- 回归检查：`pnpm check`、相关 Rust 测试、`git diff --check` 和 macOS arm64 构建通过。

## 非目标

- 不处理暗色模式。
- 不配置 Developer ID、公证、App Store 或 GitHub CI 证书。
- 不保证免费证书构建可无警告分发给其他 Mac。
- 不清空应用业务数据。
