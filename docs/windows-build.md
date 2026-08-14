# Windows x64 构建与安装

XiLuoLin `v0.1.0` 面向 Windows 10/11 x64 提供未签名的 NSIS 稳定版安装包。Windows ARM64、Microsoft Store 和已签名安装包不在本次发布范围内。

## 环境

- Windows 10 或 Windows 11 x64
- Microsoft Visual Studio C++ Build Tools
- WebView2 Runtime
- Node.js 20+、pnpm 10+、Rust stable、CMake
- `sherpa-onnx` 在 Windows 使用 shared runtime，构建时会把所需 DLL 放到发布目录并由 NSIS 一并打包，避免与 Whisper 的 CRT 发生冲突。

## 构建

```powershell
pnpm install --frozen-lockfile
pnpm check
pnpm tauri:build:windows:x64
```

产物位于：

```text
src-tauri/target/x86_64-pc-windows-msvc/release/bundle/nsis/XiLuoLin_0.1.0_x64-setup.exe
```

## 安装

本次安装包未进行 Windows 代码签名，Microsoft Defender SmartScreen 可能显示未知发布者警告。只从项目 GitHub Release 下载，并先使用同一 Release 中的 `SHA256SUMS.txt` 校验文件。

确认下载来源和校验和后，可在 SmartScreen 中选择“更多信息 → 仍要运行”。安装完成后，按设置页提示授予麦克风权限并配置 Provider、麦克风和全局快捷键。

## 卸载与数据

通过 Windows“设置 → 应用 → 已安装的应用”卸载 XiLuoLin。卸载应用不等同于删除用户本地历史、模型和设置；需要彻底清理时，应先备份需要保留的数据，再手动移除 XiLuoLin 应用数据目录。

## 验证清单

- 在 Windows 10 和 Windows 11 x64 各验证全新安装、启动、覆盖安装和卸载。
- 确认麦克风、Windows Credential Manager、长按/切换快捷键和状态悬浮窗正常。
- 在普通文本编辑器和至少一个日常应用中验证自动粘贴；跨权限级别失败时应保留剪贴板结果。
- 验证无效 API Key、快捷键冲突、Provider 超时和目标窗口关闭等失败路径。
- 检查安装目录中包含 `sherpa-onnx`/ONNX Runtime DLL；实时预览仍默认关闭，只有用户下载并启用模型后才加载。

## 已知限制

- 安装包未签名，SmartScreen 警告属于当前稳定版的已知行为。
- Windows 不允许低权限进程向高权限窗口发送输入；此时需要手动粘贴。
- 不支持 Windows ARM64，也不提供 MSI 或便携版。
