# macOS Apple Silicon 构建与安装

XiLuoLin `v0.1.0-beta.1` 面向 macOS 13 及以上、Apple Silicon（arm64）提供未公证技术预览包。应用使用完整的 ad-hoc 签名以满足 Apple Silicon 运行要求，但没有 Developer ID 身份和 Apple 公证。

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
src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/XiLuoLin_0.1.0-beta.1_aarch64.dmg
```

最低系统版本固定为 macOS 13。构建脚本同时设置 `MACOSX_DEPLOYMENT_TARGET` 和 `CMAKE_OSX_DEPLOYMENT_TARGET`，确保 Tauri Bundle 与 whisper.cpp 原生编译使用相同目标。

## 首次启动

由于应用未经 Apple 公证，从浏览器下载后 Gatekeeper 会要求用户确认：

1. 在 Finder 中打开 DMG，并将 XiLuoLin 拖入“应用程序”。
2. 右键 XiLuoLin 并选择“打开”，在提示中再次确认。
3. 如果仍被阻止，打开“系统设置 → 隐私与安全性”，在安全提示中选择“仍要打开”。
4. 返回应用后按设置页提示授予麦克风和辅助功能权限。

不同预览构建的 ad-hoc 代码身份可能变化，macOS 可能重新请求 Keychain、麦克风或辅助功能权限。

## 卸载与数据

退出 XiLuoLin 后，将“应用程序”中的 XiLuoLin 移到废纸篓。卸载应用不会自动删除 Keychain 凭据、本地历史、模型和设置；需要彻底清理时，应先备份需要保留的数据，再移除 `com.xiluolin.desktop` 对应的应用数据和 Keychain 项目。

## 权限

- **麦克风**：用于录制需要转写的短语音；未授权时录音不会开始。
- **辅助功能**：用于恢复录音开始时的应用窗口并发送 `Command+V`；未授权时识别结果仍会复制到剪贴板。
- 设置页“语音输入就绪检查”可以读取权限状态、请求权限并打开对应的 macOS 设置页。

## 验证产物

```bash
APP="src-tauri/target/aarch64-apple-darwin/release/bundle/macos/XiLuoLin.app"
DMG="src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/XiLuoLin_0.1.0-beta.1_aarch64.dmg"

file "$APP/Contents/MacOS/xiluolin"
plutil -p "$APP/Contents/Info.plist"
codesign --verify --deep --strict --verbose=2 "$APP"
codesign -dv --verbose=4 "$APP"
hdiutil verify "$DMG"
shasum -a 256 "$DMG"
```

预期主程序架构包含 `arm64`，`Info.plist` 包含 `LSMinimumSystemVersion = 13.0`、`CFBundleIdentifier = com.xiluolin.desktop` 和 `NSMicrophoneUsageDescription`。`codesign` 严格验证必须成功并显示 ad-hoc；由于没有 Apple 公证，`spctl` 拒绝属于预期行为。

## 已知限制

- 未使用 Developer ID 签名和 Apple 公证，Gatekeeper 需要用户手动允许。
- 只构建 Apple Silicon，不支持 Intel Mac。
- 多窗口应用会优先恢复录音开始时的精确窗口；无法匹配时退化为恢复原应用。
- 目标应用退出、权限不足或系统无法确认焦点时不会发送按键，文本保留在剪贴板。
- macOS 暂不在录音期间静音其他应用。
