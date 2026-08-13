# macOS Apple Silicon 构建与安装

XiLuoLin `v0.1.0` 面向 macOS 13 及以上、Apple Silicon（arm64）提供未公证安装包。仓库保留通用 ad-hoc 构建，同时提供使用本机 Apple Development 证书的稳定开发签名入口；两种方式都没有 Developer ID 身份和 Apple 公证。

## 环境

- Apple Silicon Mac
- macOS 13 或更高版本
- Xcode Command Line Tools
- Node.js 20+、pnpm 10+、Rust stable、CMake

## 构建

通用构建显式使用 ad-hoc 签名，适用于 CI 或没有开发证书的环境：

```bash
pnpm install --frozen-lockfile
pnpm check
pnpm tauri:build:macos:arm64
```

ad-hoc 签名的代码身份与具体构建绑定。需要在同一台 Mac 上反复安装和体验麦克风、辅助功能时，应使用钥匙串中的个人 Apple Development 身份：

```bash
APPLE_SIGNING_IDENTITY="Apple Development: <name> (<team-id>)" \
  pnpm tauri:build:macos:arm64:signed
```

`APPLE_SIGNING_IDENTITY` 只在当前命令的环境中传入，不应写入仓库、`.env`、日志或共享脚本。可以通过 `security find-identity -v -p codesigning` 查看当前钥匙串可用的身份。稳定签名入口会拒绝空值和 ad-hoc 的 `-`。

证书和对应私钥存在于当前 Mac 的钥匙串即可执行签名；macOS 系统设置当前登录哪个 Apple 账号、常用 Apple 账号位于哪台 Mac，都不参与运行时签名判断。

产物位于：

```text
src-tauri/target/aarch64-apple-darwin/release/bundle/macos/XiLuoLin.app
src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/XiLuoLin_0.1.0_aarch64.dmg
```

最低系统版本固定为 macOS 13。构建脚本同时设置 `MACOSX_DEPLOYMENT_TARGET` 和 `CMAKE_OSX_DEPLOYMENT_TARGET`，确保 Tauri Bundle 与 whisper.cpp 原生编译使用相同目标。

## 首次启动

由于应用未经 Apple 公证，从浏览器下载后 Gatekeeper 会要求用户确认：

1. 在 Finder 中打开 DMG，并将 XiLuoLin 拖入“应用程序”。
2. 右键 XiLuoLin 并选择“打开”，在提示中再次确认。
3. 如果仍被阻止，打开“系统设置 → 隐私与安全性”，在安全提示中选择“仍要打开”。
4. 返回应用后按设置页提示授予麦克风和辅助功能权限。

不同 ad-hoc 构建的代码身份会变化，macOS 可能重新请求 Keychain、麦克风或辅助功能权限。同一张 Apple Development 证书签出的连续开发构建具有稳定的 designated requirement（DR），能在同一台 Mac 上复用授权；证书到期、撤销、重新签发或换 Mac 后仍可能需要重新授权一次。

## 卸载与数据

退出 XiLuoLin 后，将“应用程序”中的 XiLuoLin 移到废纸篓。卸载应用不会自动删除 Keychain 凭据、本地历史、模型和设置；需要彻底清理时，应先备份需要保留的数据，再移除 `com.xiluolin.desktop` 对应的应用数据和 Keychain 项目。

## 权限

- **麦克风**：用于录制需要转写的短语音；未授权时录音不会开始。
- **辅助功能**：用于恢复录音开始时的应用窗口并发送 `Command+V`；未授权时识别结果仍会复制到剪贴板。
- 设置页“语音输入就绪检查”可以读取权限状态、请求权限并打开对应的 macOS 设置页。

如果系统设置显示 XiLuoLin 已开启，但应用仍报告未请求或未授权，通常是旧 ad-hoc 构建的授权与当前 DR 不匹配。确认已经安装稳定签名版本后，可以仅重置 XiLuoLin 的两项记录：

```bash
tccutil reset Microphone com.xiluolin.desktop
tccutil reset Accessibility com.xiluolin.desktop
```

重置不会删除应用配置、历史、模型或录音，但会清除当前麦克风和辅助功能开关。重新启动 `/Applications/XiLuoLin.app`，按设置页提示分别授权即可。

## 验证产物

```bash
APP="src-tauri/target/aarch64-apple-darwin/release/bundle/macos/XiLuoLin.app"
DMG="src-tauri/target/aarch64-apple-darwin/release/bundle/dmg/XiLuoLin_0.1.0_aarch64.dmg"

file "$APP/Contents/MacOS/xiluolin"
plutil -p "$APP/Contents/Info.plist"
codesign --verify --deep --strict --verbose=2 "$APP"
codesign -dv --verbose=4 "$APP"
codesign -d -r - "$APP"
hdiutil verify "$DMG"
shasum -a 256 "$DMG"
```

预期主程序架构包含 `arm64`，`Info.plist` 包含 `LSMinimumSystemVersion = 13.0`、`CFBundleIdentifier = com.xiluolin.desktop` 和 `NSMicrophoneUsageDescription`。`codesign` 严格验证必须成功。

通用构建的 DR 只绑定具体 `cdhash`；稳定开发签名的 DR 应包含 `identifier "com.xiluolin.desktop"`、Apple 签名锚点和所选开发证书。由于没有 Apple 公证，两种构建的 `spctl` 拒绝都属于预期行为。

## 已知限制

- 免费 Apple Developer 账号可以创建本机开发签名，但不能生成 Developer ID 公证发行包；安装到其他 Mac 时仍可能被 Gatekeeper 阻止或要求额外确认。
- 未使用 Developer ID 签名和 Apple 公证，Gatekeeper 需要用户手动允许；这是当前开发版的已知限制。
- 只构建 Apple Silicon，不支持 Intel Mac。
- 多窗口应用会优先恢复录音开始时的精确窗口；无法匹配时退化为恢复原应用。
- 目标应用退出、权限不足或系统无法确认焦点时不会发送按键，文本保留在剪贴板。
- macOS 暂不在录音期间静音其他应用。
