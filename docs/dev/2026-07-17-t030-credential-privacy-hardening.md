# T030 加固本地凭据和隐私数据处理

## 任务目标

将 API Key 从 Tauri Store 明文配置迁移到操作系统凭据库，移除敏感调试日志，并限制和清理应用生成的录音临时文件。

## 实际改动

- 引入 `keyring` 4.1.5，通过 Windows Credential Manager、macOS Keychain 和系统原生凭据存储管理密钥。
- 新增凭据抽象，统一处理 ASR、OpenAI 和智谱文本 Provider 三类 API Key。
- `read_app_config` 读取系统凭据并恢复到现有 `AppConfig`，前端接口保持不变。
- `update_app_config` 先写入或删除系统凭据，再把清空密钥字段的配置写入 Tauri Store。
- 兼容旧版明文配置：系统凭据不存在时迁移旧值，写入失败则返回错误且不清空旧配置。
- 删除 API Key 长度和片段、原始文本、最终文本、剪贴板文本和录音完整路径日志。
- 限制 `process_recording_file` 只能处理应用 `recordings` 目录内的 WAV，并通过 canonical path 拒绝外部路径、目录穿越和 symlink 越界。
- 应用录音通过路径和格式校验后立即挂载清理 guard；无论读取、配置、ASR、文本处理失败或处理过程 panic 都尝试删除，清理失败不覆盖原处理结果，也不打印完整路径。
- 用户上传音频流程保持独立，不删除用户自行选择的外部文件。

## 为什么这么做

此前完整 `AppConfig` 会被序列化到 `settings.json`，API Key 因此以明文落盘；调试日志还可能输出密钥片段、用户文本和录音路径。应用录音文件处理后也会保留在应用数据目录。

本任务在保持设置页和前端配置接口兼容的前提下，把持久化职责拆分为“非敏感配置进入 Tauri Store，密钥进入系统凭据库”，并收紧录音文件的读取和生命周期边界。

## 涉及文件

- `src-tauri/src/credentials.rs`
- `src-tauri/src/data.rs`
- `src-tauri/src/pipeline.rs`
- `src-tauri/src/indicator.rs`
- `src-tauri/tests/recording_file_security.rs`
- `src-tauri/Cargo.toml`
- `README.md`
- `docs/requirements-analysis.md`
- `docs/solution-design.md`

## 测试与验证

执行：

```bash
pnpm check
```

结果：

- TypeScript 检查和 Vite 构建通过。
- Rust 格式检查和 `cargo check` 通过。
- Rust 测试共 32 个通过：
  - 原有集成测试 21 个；
  - 凭据拆分、恢复和迁移单元测试 5 个；
  - 录音路径及清理集成测试 6 个。
- 敏感日志扫描未发现 API Key 长度或片段、用户文本和完整录音路径输出。
- Tauri Store 写入点只序列化 `sanitized_config`。
- Windows CI 需要本任务推送到 `main` 后再次确认。

验证场景：

1. 旧明文凭据在系统凭据不存在时自动迁移。
2. 系统凭据优先于遗留明文值。
3. 凭据写入失败时旧配置保持不变。
4. 空密钥会删除系统凭据。
5. 录音处理成功后删除 WAV。
6. 配置、读取或处理失败后仍删除 WAV。
7. 处理过程 panic 时仍通过清理守卫删除 WAV。
8. 外部文件、目录穿越和非 WAV 被拒绝且不会删除源文件。

## 执行复盘

### 遇到的问题

1. `keyring` 4.x 已拆分为统一 v1 兼容 API 和平台存储 crate，需要按当前 crate 源码确认错误类型和平台初始化行为。
2. 凭据迁移不能先清空 Tauri Store，否则系统凭据写入失败会造成密钥丢失。
3. `process_recording_file` 原本接受任意字符串路径，既可能读取外部文件，也无法保证只清理应用生成文件。

### 解决流程

1. 使用 `keyring::Entry` 和 `keyring::Error::NoEntry` 实现系统存储适配器，并使用内存实现测试业务逻辑。
2. 迁移时先读取系统值；不存在时写入旧明文；全部成功后才保存清空密钥的配置。
3. 将录音读取封装为可测试函数，先 canonicalize 和目录归属校验，再通过 Drop guard 覆盖读取失败、正常错误和 panic 的清理路径。
4. 扫描 Rust 和前端日志，删除所有包含敏感内容的输出。

### 经验总结

- 敏感配置迁移必须遵循“先安全写入、后删除明文”的顺序。
- 路径前缀比较前必须 canonicalize，单纯检查字符串前缀无法阻止 `..` 和 symlink 越界。
- 隐私检查既需要功能测试，也需要对日志和持久化写入点做静态扫描。

## 未完成事项

- 未使用真实 API Key 执行迁移 smoke test，避免自动化测试污染用户凭据库。
- Windows Credential Manager 的真实写入将在桌面应用手动测试时确认；当前 Windows CI 只验证编译和自动化测试。

## 后续建议

- 在 Windows 桌面环境使用测试凭据执行一次设置保存、应用重启和密钥读取 smoke test。
- 后续可增加设置页中的“凭据已安全保存”状态提示，但不应把完整密钥返回给非必要的前端组件。
