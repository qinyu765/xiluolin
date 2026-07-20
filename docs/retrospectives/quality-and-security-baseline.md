# 质量基线、安全与隐私改造复盘

## 1. 背景

本阶段对应 T029 和 T030。目标不是增加用户可见功能，而是建立后续功能可以信赖的工程基础。

开始时存在一个典型风险：项目“看起来能构建”，但没有统一命令证明前端类型、Rust 编译、测试和 Windows 平台同时可用。与此同时，API Key、录音文件和调试日志的边界不清晰。

如果直接继续开发本地 ASR、Capture 或更多 UI，会把已有问题继续扩散：

- 配置字段继续漂移；
- 安全问题进入更多代码路径；
- 每次 PR 都无法证明没有破坏 Windows；
- 文档继续宣称“已完成”，但缺少可复现验证。

因此先建立质量与安全基线。

## 2. 问题一：Vite 构建成功掩盖 TypeScript 错误

### 2.1 现象

执行：

```bash
pnpm build
```

结果成功。

但执行：

```bash
pnpm exec tsc --noEmit
```

出现 6 个错误：

- 旧 `AppSettingsDialog` 仍引用已经删除的 `output_mode`；
- 前端 `AppConfig` 与 Rust `AppConfig` 不一致；
- Lucide 动态图标通过 namespace 索引，类型中混入非组件导出；
- 快捷键输入组件调用不存在的 `setHasNonModifierKey`。

### 2.2 根因

`package.json` 中的构建脚本只调用自定义 Vite build：

```text
build → scripts/build.mjs → vite.build()
```

Vite 负责转译和打包，不负责完整 TypeScript 类型检查。因此“构建成功”只能说明 JavaScript bundle 生成成功，不能证明类型正确。

### 2.3 解决方案

统一脚本语义：

```text
typecheck     → tsc --noEmit
build         → typecheck + frontend build
check:rust    → cargo fmt --check + cargo check + cargo test
check         → build + check:rust
```

并在 Windows GitHub Actions 中执行同等检查。

### 2.4 具体改动

- 删除已经没有运行时入口的 `AppSettingsDialog`；
- 删除漂移的 `recording_mode` 前端字段；
- 建立类型安全的人格图标映射；
- 删除不存在的 setter；
- 新增 `pnpm typecheck`、`pnpm check:rust`、`pnpm check`；
- 让正式 build 强制先通过类型检查；
- 新增 Windows CI。

### 2.5 经验

- Vite、Babel、esbuild 的“能打包”不等于 TypeScript “类型正确”。
- `cargo check` 不等于 `cargo test`，集成测试可能引用已经删除的字段。
- 一个仓库应该有一个明确的总检查命令，例如 `pnpm check`。
- README 的验证命令必须与 CI 使用同一套语义。

## 3. 问题二：Rust 工具链和跨平台构建缺失

### 3.1 现象

最初环境中：

```text
rustc: command not found
cargo: command not found
```

因此无法验证 Tauri/Rust 代码。

安装 Rust 后，又遇到：

- 缺失 `icon.png` 导致 Tauri 生成上下文失败；
- macOS 透明窗口 API 需要私有 feature；
- 历史测试仍引用旧字段；
- 原有 Rust 文件没有统一 rustfmt 基线。

### 3.2 解决方案

- 安装 rustup stable；
- 补充 Tauri 所需 PNG 图标；
- 通过条件编译避免 macOS 默认启用私有透明窗口 API；
- 更新测试到当前数据结构；
- 全仓建立 rustfmt 基线；
- Windows runner 作为首要平台验证 Rust 编译。

### 3.3 后续注意

GitHub Actions 当前有 Node 20 action runtime 弃用警告：

```text
actions/checkout@v4
actions/setup-node@v4
```

未来应升级到支持新 runtime 的 action 版本，避免警告最终变成失败。

## 4. 问题三：API Key 明文落盘

### 4.1 现象

虽然 API Key 没有提交到 Git，但完整 `AppConfig` 会被序列化到 Tauri Store 的 `settings.json`。

这意味着：

- Git 安全不等于本地安全；
- 任何能读取应用配置目录的进程都能获得 API Key；
- README 中“本地保存”没有说明是明文还是安全存储。

### 4.2 设计目标

在不大规模重写设置页的前提下：

- 前端仍使用现有 `AppConfig`；
- API Key 不再明文写入 `settings.json`；
- 旧用户配置自动迁移；
- 迁移失败不能丢失旧 Key；
- 清空输入框应删除系统凭据。

### 4.3 持久化拆分

```text
SQLite
  ├─ 人格
  ├─ 热词
  ├─ 历史
  └─ 统计来源

Tauri Store
  ├─ Provider
  ├─ Base URL
  ├─ 模型名
  ├─ 快捷键
  └─ 非敏感偏好

系统凭据库
  └─ com.xiluolin.desktop / app_credentials_v1
      └─ AppCredentials JSON（ASR、OpenAI-compatible、智谱文本 Key）

进程内缓存
  └─ OnceLock<Mutex<Option<AppCredentials>>>
```

Windows 使用 Credential Manager，macOS 使用 Keychain。

### 4.4 迁移顺序

旧明文配置迁移遵循：

```text
读取 settings.json
  → 读取系统凭据
  → 系统凭据不存在时写入旧值
  → 所有写入成功
  → 保存清空 Key 字段的 settings.json
  → 返回内存中已恢复 Key 的 AppConfig
```

不能采用：

```text
先清空 settings.json → 再写系统凭据
```

否则系统凭据写入失败会造成不可恢复的数据丢失。

### 4.5 凭据抽象

引入 `CredentialStore` trait：

- 生产实现：系统凭据库；
- 测试实现：内存 HashMap；
- 业务逻辑不直接依赖操作系统 API；
- 可以测试迁移失败、删除和优先级。

### 4.6 macOS 授权体验修正

早期实现把三个 Key 保存成三个独立条目，并兼容旧服务名 `com.xiluolin.app`。后端启动、前端初始化和就绪检查又会重复读取配置；在 ad-hoc 签名的 `tauri dev` 下，这会放大为多次 Keychain 授权弹窗。

当前实现改为一个 bundled 条目并在首次读取后缓存到进程内。旧分散条目只用于一次性迁移。这样不降低安全存储级别，同时减少授权次数和重复系统调用。

### 4.7 测试场景

- 旧明文值成功迁移；
- 已存在的安全凭据优先于旧值；
- 中途写入失败时配置不被清空；
- 空字符串删除安全凭据；
- 序列化配置不含 Key。

## 5. 问题四：敏感日志泄漏

### 5.1 已发现内容

日志曾输出：

- API Key 长度；
- API Key 前 8 或 10 个字符；
- 原始识别文本；
- 最终整理文本；
- 剪贴板文本；
- 完整录音文件路径；
- whisper.cpp token 和识别文本。

### 5.2 风险

日志可能进入：

- IDE 控制台；
- CI 日志；
- 用户提供的故障截图；
- 崩溃报告；
- 第三方日志收集工具。

部分 API Key 前缀足以识别账号或与其他泄漏片段组合。

### 5.3 处理原则

可以记录：

- 阶段名称；
- Provider 名称；
- 模型名；
- 时长；
- 成功/失败；
- 不含用户数据的错误分类。

不记录：

- API Key 内容或片段；
- 用户文本；
- 完整录音路径；
- 模型 token 输出。

### 5.4 whisper.cpp 特殊处理

即使 `FullParams` 关闭 progress/realtime 输出，debug 构建仍可能输出 token 和文本。

最终使用：

```rust
whisper_rs::install_logging_hooks()
```

项目没有启用对应 log/tracing backend，因此 whisper.cpp/ggml 日志被安全丢弃。

## 6. 问题五：录音路径可由前端传入

### 6.1 原始风险

`process_recording_file(file_path)` 接受字符串路径，Rust 直接读取。

如果 WebView 被错误调用或未来页面出现注入问题，理论上可以：

- 读取应用目录外文件；
- 把任意文件作为音频发送到远端；
- 删除非应用生成文件。

### 6.2 解决方案

处理前：

1. canonicalize 应用 `recordings` 目录；
2. canonicalize 目标文件；
3. 检查目标必须位于受管目录；
4. 检查是普通文件；
5. 检查扩展名为 WAV。

这比字符串 `starts_with` 安全，因为 canonical path 会消解：

- `..`；
- 符号链接；
- macOS `/var` 与 `/private/var` 的路径别名。

### 6.3 清理 guard

最初实现只在处理闭包返回后删除文件，但以下情况仍会残留：

- 文件读取失败；
- 中间函数提前返回；
- 处理闭包 panic。

最终使用 RAII/Drop guard：

```text
路径和格式校验成功
  → 创建 cleanup guard
  → 读取和处理
  → 默认 Drop 删除
  → 只有满足保留条件时 disarm
```

### 6.4 测试场景

- 正常处理后删除；
- ASR/配置失败后删除；
- panic 后删除；
- 外部路径拒绝且不删除；
- `..` 穿越拒绝；
- symlink 越界拒绝；
- 非 WAV 拒绝且不删除；
- 显式保留成功时文件存在。

## 7. CI 设计

Windows CI 执行：

```text
checkout
  → Node/pnpm
  → pnpm install --frozen-lockfile
  → TypeScript
  → Vite build
  → Rust stable
  → rustfmt
  → cargo check
  → cargo test
```

选择 Windows 作为首要 CI 平台的原因：

- 产品主要依赖 Windows 全局快捷键和窗口恢复；
- 音频静音使用 Win32；
- 自动粘贴涉及 Windows UIPI；
- whisper.cpp 在 Windows 上需要独立验证；
- 仅在 macOS 本地成功不能证明 Windows 可交付。

## 8. 结果

本阶段完成后：

- `pnpm build` 不再绕过类型检查；
- `pnpm check` 成为统一质量入口；
- Windows CI 可以验证完整 Rust/Tauri 代码；
- API Key 不再明文保存；
- 旧配置可迁移；
- 敏感日志被移除；
- 应用录音读取和删除受到目录约束；
- 成功、失败和 panic 清理均有测试。

## 9. 可复用检查清单

### 新配置字段

- [ ] 前后端类型同时更新；
- [ ] Rust 字段有 `serde(default)` 或迁移策略；
- [ ] 默认值测试更新；
- [ ] README/方案文档同步；
- [ ] 敏感字段不进入普通 Store。

### 新文件处理命令

- [ ] canonicalize 根目录和目标；
- [ ] 限制文件类型；
- [ ] 处理失败清理；
- [ ] 不打印完整路径；
- [ ] 外部文件不被误删；
- [ ] 有 traversal/symlink 测试。

### 新日志

- [ ] 不包含 API Key；
- [ ] 不包含用户完整文本；
- [ ] 不包含完整本地路径；
- [ ] 错误信息可用于定位但不泄露数据。

## 10. 相关文件

- `src-tauri/src/credentials.rs`
- `src-tauri/src/data.rs`
- `src-tauri/src/pipeline.rs`
- `src-tauri/src/recording_storage.rs`
- `src-tauri/tests/recording_file_security.rs`
- `.github/workflows/ci.yml`
- `package.json`
