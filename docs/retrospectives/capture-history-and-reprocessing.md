# Capture 历史、录音保留与重新处理复盘

## 1. 背景

早期历史记录主要用于展示：

- 原始识别文本；
- 整理后文本；
- 人格；
- 录音时长；
- 输出字数；
- 创建时间。

这足以做 MVP 列表和统计，但不能回答：

- 这条结果来自智谱、OpenAI 还是本地模型？
- 使用了哪个模型？
- 是否发生过降级？
- 最终是自动粘贴、复制还是手动兜底？
- 原始录音是否还存在？
- 更换人格或模型后能否重新处理？

要支持长期使用，历史记录需要从“结果列表”升级为可解释、可复用的 Capture。

## 2. 设计目标

1. 记录结果的来源和处理上下文。
2. 默认不保留录音。
3. 用户可以明确选择保留应用录音。
4. 录音必须和历史建立成功关联后才能保留。
5. 删除历史时同步删除录音。
6. 可以查看录音占用并清理。
7. 有录音时可以试听和重新转写。
8. 没有录音时仍可用 raw_text 重新整理。
9. 重新处理更新原记录，不创建重复历史。
10. 旧数据库自动迁移且不丢数据。

## 3. 历史数据模型扩展

新增字段：

| 字段 | 作用 |
|---|---|
| `source` | recording、upload、reprocess 等输入来源 |
| `asr_provider` | 实际使用的 ASR Provider |
| `asr_model` | 实际使用的 ASR 模型 |
| `text_provider` | 文本处理 Provider |
| `text_model` | 文本处理模型 |
| `used_asr_fallback` | 本地 ASR 是否降级到云端 |
| `used_fallback` | 文本整理是否降级到原文 |
| `delivery_method` | pending、paste、copy、manual |
| `audio_path` | 可空的受管录音路径 |

保留 `output_mode` 作为兼容字段，并在投递更新时同步写入。

## 4. 为什么需要保存“实际 Provider”

不能只保存配置中的 `asr_provider`。

例如：

```text
配置 Provider = local
本地模型失败
用户允许云端降级
实际调用 = zhipu
```

如果历史只写 `local`，会误导用户，以为音频没有上传。

因此 ASR 结果返回：

```text
text
actual provider
actual model
used_fallback
```

历史以结果为准，而不是以调用前配置为准。

## 5. 从异步历史写入改为同步关联

### 5.1 原实现

为了降低主流程延迟，历史通过后台线程写入：

```text
process result 返回
  → 后台线程写 SQLite
  → VoiceInputResult.history_record = None
```

### 5.2 问题

后续需要：

- 把 history_id 附加到 CaptureSession；
- 投递完成后更新 delivery_method；
- 决定录音是否可以保留；
- 前端立即刷新到新记录。

fire-and-forget 无法保证这些操作发生在同一条记录上。

### 5.3 调整

历史写入改为同步：

```text
ASR + 整理完成
  → 创建历史
  → 返回 HistoryRecord
  → 绑定 session.history_id
  → deliver_text
  → 更新 delivery_method
```

SQLite 单条插入成本相对网络 ASR/LLM 很小，换取的数据一致性更有价值。

### 5.4 经验

异步优化必须围绕业务依赖判断。

如果后续逻辑依赖新记录 ID，就不能把写入当成无关副作用。

## 6. 录音保留策略

新增配置：

```text
retain_recordings = false
```

默认关闭。

录音只有在以下条件全部成立时保留：

```text
retain_recordings == true
AND auto_save_history == true
AND 历史写入成功
AND 处理成功返回
```

否则 cleanup guard 默认删除。

### 为什么依赖自动保存历史

没有历史记录的录音：

- 用户无法在 UI 中发现；
- 不知道来源和时间；
- 无法安全删除；
- 容易成为孤立隐私数据。

因此不允许只保留文件而不建立业务记录。

## 7. cleanup guard 的 disarm 模式

录音处理使用默认删除的 guard：

```text
创建 guard（armed）
  → 读取录音
  → ASR
  → 文本整理
  → 历史写入
  → 计算 retain_recording
```

如果应该保留：

```text
guard.disarm()
```

否则函数结束或异常展开时自动删除。

这个模式把隐私默认值编码进控制流：

> 新增错误路径时，默认结果仍然是删除，而不是意外保留。

## 8. 数据库迁移

### 8.1 兼容目标

旧数据库已经有 `history_records` 表，不能删除重建。

初始化时：

1. `PRAGMA table_info(history_records)`；
2. 收集已有列；
3. 对缺失列执行 `ALTER TABLE ADD COLUMN`；
4. 每个新列提供兼容默认值；
5. 保留已有行。

### 8.2 默认值示例

```text
source = unknown
used_asr_fallback = 0
used_fallback = 0
delivery_method = pending
audio_path = NULL
```

### 8.3 测试

测试会先手工创建旧版表和旧记录，再调用 `initialize()`，验证：

- 旧数据仍存在；
- 新列可读取；
- 默认值正确；
- 重复 initialize 不报错。

## 9. 投递方式更新

历史最初创建时，文本还没有完成输出，因此：

```text
delivery_method = pending
```

`deliver_text` 完成后：

- 自动粘贴成功：`paste`；
- 应用内或复制按钮：`copy`；
- 自动粘贴失败但复制成功：`manual`。

### 刷新顺序问题

最初前端并行执行：

```text
重新加载历史 || 自动投递
```

历史查询可能先完成，UI 会暂时显示 `pending`。

修复为：

```text
先 deliver_text 更新历史
  → 再查询历史和统计
```

这是典型的数据竞争问题：并行并不总是正确，尤其存在写后读依赖时。

## 10. 录音存储管理

设置页新增录音存储卡片：

- 文件数量；
- 总占用；
- 目录路径；
- 打开目录；
- 清理全部。

### 安全约束

- 只统计应用 `recordings` 目录内 WAV；
- canonical path 必须位于受管根目录；
- 活跃 CaptureSession 期间禁止清理；
- 清理后把历史 `audio_path` 置空；
- 用户上传的外部文件不进入该清理逻辑。

### 文件系统与数据库无法同事务

删除单条历史时有两种顺序：

#### 先删数据库

如果文件删除失败，会留下无法追踪的文件。

#### 先删文件

如果数据库删除失败，历史仍存在但文件已经删除。

最终选择单条删除时先删除受管文件，再删除历史。原因是：

- 文件删除失败时保留历史，用户可以重试；
- 相比孤立隐私文件，历史引用一个已删除文件更容易发现和修复。

批量清理则在删除文件后统一清空数据库音频关联，并记录失败。

## 11. 试听

前端调用：

```text
read_retained_recording(history_id)
```

Rust：

1. 从数据库获取 audio_path；
2. 验证属于受管目录；
3. 读取 WAV 字节；
4. 返回前端。

前端：

```text
Uint8Array
  → Blob(audio/wav)
  → URL.createObjectURL
  → HTMLAudioElement.play
  → ended/error 后 revokeObjectURL
```

### 限制

- 没有波形；
- 没有进度控制；
- 没有倍速；
- 大音频通过 IPC 传输成本较高；
- 当前产品仍聚焦短语音。

## 12. 重新转写

### 流程

```text
history_id
  → 获取受管 WAV
  → 读取当前 ASR/Text 配置
  → 当前默认人格和热词
  → ASR
  → 文本整理
  → 更新原历史
```

更新：

- raw_text；
- final_text；
- persona；
- 实际 ASR Provider/模型；
- 文本 Provider/模型；
- 降级状态；
- output_chars。

保留：

- id；
- audio_path；
- source；
- duration；
- delivery_method；
- created_at。

### 为什么不创建新记录

如果每次重新处理都创建新记录：

- 历史数量被人工放大；
- 同一录音分散为多条；
- 统计失真；
- 用户难以理解哪条是原始 Capture。

当前选择把 Capture 视为同一个对象，处理结果可更新。

未来如需保留版本，可增加独立 revision 表，而不是复制整个历史行。

## 13. 重新整理

重新整理不需要音频：

```text
history.raw_text
  → 当前默认人格
  → 当前热词
  → 当前文本 Provider
  → 更新 final_text
```

原始识别文本不变。

适用场景：

- 用户更换人格；
- Prompt 得到改进；
- 文本模型更换；
- 热词更新；
- 之前文本整理失败。

## 14. Provider 字段漂移问题

实施重新转写时发现：

- OpenAI ASR 设置页写入 `openai_api_key/openai_base_url/openai_asr_model`；
- 旧处理链路固定读取 `asr_api_key/asr_base_url`。

结果：OpenAI Whisper 看起来配置成功，但实际处理读取了智谱字段。

解决：在 `AppConfig` 中集中提供：

```text
selected_asr_config()
cloud_asr_config(provider)
selected_text_config()
```

正常处理、就绪检查和重新处理共享这些方法。

经验：

> 可切换 Provider 不能在多个调用点手写 if/else 字段选择，必须收口成一个事实源。

## 15. 测试

- 旧表迁移；
- 新字段往返；
- 实际投递方式更新；
- 默认不保留；
- 成功时可保留；
- 失败/panic 时删除；
- 重新转写保留音频关联；
- 重新整理保留 raw_text；
- 外部音频不被删除；
- 存储路径安全。

## 16. 后续延伸

### 历史版本

增加：

```text
capture_revisions
  ├─ capture_id
  ├─ raw_text
  ├─ final_text
  ├─ provider snapshot
  └─ created_at
```

这样用户可以比较不同模型和人格结果。

### 搜索与导出

- 全文搜索；
- 收藏；
- Provider/人格筛选；
- Markdown/JSON 导出；
- 音频和文本打包导出。

### 存储策略

- 按天数自动清理；
- 最大空间配额；
- 保留收藏记录；
- 加密本地历史；
- 存储目录自定义。

## 17. 相关文件

- `src-tauri/src/data.rs`
- `src-tauri/src/pipeline.rs`
- `src-tauri/src/output.rs`
- `src-tauri/src/recording_storage.rs`
- `src-tauri/src/history_reprocessing.rs`
- `src/components/home/VoiceInputStatsCard.tsx`
- `src/components/settings/RecordingStorageCard.tsx`
- `src-tauri/tests/local_data_layer.rs`
- `src-tauri/tests/recording_file_security.rs`
