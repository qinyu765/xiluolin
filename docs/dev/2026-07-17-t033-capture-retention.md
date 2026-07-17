# T033 增加 Capture 历史和可选录音保留

## 任务目标

在默认删除录音的隐私策略下，允许用户显式保留应用录音，并为历史记录增加输入来源、Provider、模型、降级、投递方式和录音路径快照。

## 实际改动

- `AppConfig` 新增 `retain_recordings`，默认 `false`；关闭自动保存历史时同步关闭录音保留。
- 历史记录新增输入来源、ASR Provider/模型、文本 Provider/模型、文本降级、实际投递方式和可空录音路径。
- SQLite 初始化会检测并迁移旧 `history_records` 表，新增字段带兼容默认值，不删除旧记录。
- 历史写入从后台线程调整为同步返回记录 ID，使 CaptureSession 能关联历史并在投递完成后更新 `paste`、`copy` 或 `manual`。
- 只有“用户开启保留 + 自动历史开启 + 历史写入成功”时才解除录音清理 guard；其他成功或失败路径继续删除 WAV。
- 删除历史记录前先安全删除关联应用录音；删除失败时保留历史，避免产生无法追踪的孤立文件。
- 新增录音存储命令，展示应用录音目录、文件数量和占用空间，支持打开目录和清理全部录音。
- 活跃 CaptureSession 期间禁止执行全部清理；文件操作继续限制在应用 `recordings` 目录和 WAV 格式内。
- 设置页增加保留开关和录音存储卡片；历史列表展示来源、Provider/模型、降级和录音保留状态。
- 上传音频不复制或删除用户外部源文件，历史仅保存文本和 Provider 快照。

## 为什么这么做

此前历史只保存文本、人格和时长，无法解释结果由哪个 Provider 生成，也无法记录真实投递方式。录音则始终删除，无法支持用户主动选择后的试听和重新转写。

本任务保持“默认删除”的隐私策略，只在历史成功建立关联时保留应用录音，并为后续试听、重新转写和重新润色提供可迁移的数据基础。

## 涉及文件

- `src-tauri/src/data.rs`
- `src-tauri/src/pipeline.rs`
- `src-tauri/src/output.rs`
- `src-tauri/src/recording_storage.rs`
- `src-tauri/tests/local_data_layer.rs`
- `src-tauri/tests/recording_file_security.rs`
- `src/components/settings/RecordingStorageCard.tsx`
- `src/pages/SettingsPage.tsx`
- `src/components/home/VoiceInputStatsCard.tsx`
- `src/types/config.ts`
- `src/types/history.ts`
- `README.md`
- `docs/requirements-analysis.md`
- `docs/solution-design.md`

## 测试与验证

执行：

```bash
pnpm check
```

验证场景：

1. 旧历史表自动增加新字段，旧记录保持可读。
2. Provider、模型、降级、投递方式和录音路径完整往返。
3. 默认配置关闭录音保留。
4. 处理成功时可按策略保留录音。
5. 处理失败和 panic 时仍删除录音。
6. 外部路径和非 WAV 不会被存储管理误删。
7. 投递完成后更新历史的真实投递方式。
8. 活跃会话期间拒绝清理全部录音。

实际结果：

- TypeScript 类型检查和 Vite production build 通过。
- Rust 格式检查和 `cargo check` 通过。
- Rust 测试共 45 个通过。
- 新增旧数据库迁移、历史元数据往返、受管路径和录音保留测试。
- `git diff --check` 通过。
- Windows CI 和真实录音目录操作需分支推送后确认。

## 执行复盘

### 遇到的问题

1. 历史原本异步保存并始终返回 `None`，无法为 CaptureSession 提供历史 ID，也无法可靠更新投递方式。
2. 文件保留决策必须发生在处理和历史写入之后，不能仅根据设置提前取消清理。
3. 文件系统删除和 SQLite 更新无法形成同一个事务，需要选择失败时的数据一致性优先级。

### 解决流程

1. 将轻量历史写入改为同步，确保结果返回时已建立记录关联。
2. 让录音处理闭包返回是否保留，只有历史成功后才 disarm 清理 guard。
3. 删除单条历史时先删除受管录音再删除数据库记录；若文件删除失败则保留历史供用户重试。
4. 清理全部操作禁止在活跃会话期间运行，并在删除受管 WAV 后清空历史音频关联。

### 经验总结

- 隐私默认删除和可追溯保留并不冲突，关键是保留必须依赖显式设置和成功的数据关联。
- 需要后续更新的数据不能只依赖 fire-and-forget 异步写入。
- 文件生命周期必须与历史记录生命周期共同设计。

## 未完成事项

- 本任务只建立保留、快照和存储管理；试听、重新转写和重新润色拆分到 T034。
- Windows 真实录音目录打开、清理和录音留存需要桌面端验证。

## 后续建议

执行 T034，为已保留录音增加试听、重新转写和历史文本重新润色。
