# macOS Apple Silicon 构建与安装

XiLuoLin 当前提供 macOS 13 及以上、Apple Silicon（arm64）的实验构建能力。Intel、Universal Binary、Developer ID 签名和 Apple 公证尚未接入。

## 环境

- Apple Silicon Mac
- macOS 13 或更高版本
- Xcode Command Line Tools
- Node.js 20+、pnpm 10+、Rust stable、CMake

## 构建

```bash
pnpm install --frozen-lockfile
pnpm check
pnpm tauri:build:macos:arm64
```

产物位于：

```text
src-tauri/target/aarch64-apple-darwin/release/bundle/macos/XiLuoLin.app
src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/XiLuoLin_0.1.0_aarch64.dmg
```

最低系统版本固定为 macOS 13。构建脚本同时设置 `MACOSX_DEPLOYMENT_TARGET` 和 `CMAKE_OSX_DEPLOYMENT_TARGET`，确保 Tauri Bundle 与 whisper.cpp 原生编译使用相同目标。

## 首次启动

当前安装包未使用 Apple Developer ID 签名和公证，仅包含构建工具生成的临时 ad-hoc 签名。若 Finder 阻止启动：

1. 尝试打开一次 XiLuoLin。
2. 打开“系统设置 → 隐私与安全性”。
3. 在安全性提示中选择“仍要打开”。
4. 返回应用后按设置页提示授予麦克风和辅助功能权限。

也可以在 Finder 中右键应用并选择“打开”，再确认启动。

## 权限

- **麦克风**：用于录制需要转写的短语音；未授权时录音不会开始。
- **辅助功能**：用于恢复录音开始时的应用窗口并发送 `Command+V`；未授权时识别结果仍会复制到剪贴板。
- 设置页“语音输入就绪检查”可以读取权限状态、请求权限并打开对应的 macOS 设置页。

## 验证产物

```bash
file src-tauri/target/aarch64-apple-darwin/release/bundle/macos/XiLuoLin.app/Contents/MacOS/xiluolin
plutil -p src-tauri/target/aarch64-apple-darwin/release/bundle/macos/XiLuoLin.app/Contents/Info.plist
hdiutil verify src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/XiLuoLin_0.1.0_aarch64.dmg
```

预期主程序架构包含 `arm64`，`Info.plist` 包含 `LSMinimumSystemVersion = 13.0`、`CFBundleIdentifier = com.xiluolin.desktop` 和 `NSMicrophoneUsageDescription`。

## 已知限制

- 未签名和公证，Gatekeeper 可能要求手动允许。
- 只构建 Apple Silicon，不支持 Intel Mac。
- 多窗口应用会优先恢复录音开始时的精确窗口；无法匹配时退化为恢复原应用。
- 目标应用退出、权限不足或系统无法确认焦点时不会发送按键，文本保留在剪贴板。
- macOS 暂不在录音期间静音其他应用。
