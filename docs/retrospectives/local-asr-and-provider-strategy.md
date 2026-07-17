# 本地 ASR 与混合 Provider 策略复盘

## 1. 背景

项目最初依赖云端 ASR：

- 智谱 GLM-ASR；
- OpenAI Whisper-compatible API。

优点：

- 集成快；
- 不需要分发模型；
- 本地资源占用低；
- 识别质量由服务商维护。

缺点：

- 必须联网；
- 音频需要上传；
- 受 API Key、额度、网络和服务稳定性影响；
- 无法满足明确的离线/隐私场景。

因此加入可选本地 ASR，但保持“云端优先、可选本地”，而不是把整个产品转成重型本地 AI 工作站。

## 2. 关键产品决策

### 2.1 本地能力是选项，不是强制依赖

默认仍允许使用云端 Provider。用户可以根据：

- 隐私；
- 网络；
- 延迟；
- 硬件；
- 服务额度；

选择本地或云端。

### 2.2 云端降级必须显式开启

错误方案：

```text
用户选择 local
  → 本地失败
  → 自动上传云端
```

这会违反用户对“本地模式”的合理预期。

最终策略：

```text
allow_cloud_fallback = false
```

只有用户主动开启后：

```text
本地失败
  → 读取 fallback_asr_provider
  → 使用对应云端凭据
  → 历史标记 used_asr_fallback
```

### 2.3 本地文本模型不在本阶段

音频转写本地化不代表文本整理也本地化。

当前边界：

```text
音频 → 本地 Whisper
文本 → 用户配置的文本 Provider
```

因此 README 必须说明：本地 ASR 不等于整个流程完全离线。

## 3. 技术调研更新

旧计划固定 `whisper-rs 0.12` 和约 75 MB 模型，但实施时已经过时。

重新核验后采用：

- `whisper-rs 0.16.0`；
- whisper.cpp 官方 `ggml-base-q5_1.bin`；
- 模型约 57 MB；
- CPU 默认推理；
- 不默认启用 CUDA、Metal、Vulkan 等 feature。

经验：

> 软件依赖和模型格式变化快，实施前必须重新核验官方 crate、源码示例和模型仓库，不能直接照搬数月前的计划。

## 4. 构建环境问题：缺少 CMake

### 4.1 现象

加入 `whisper-rs` 后，`whisper-rs-sys` 构建失败：

```text
failed to execute command
is cmake not installed?
```

### 4.2 原因

whisper-rs 包含 whisper.cpp C/C++ 原生构建，需要：

- CMake；
- C/C++ 编译器；
- 平台 SDK。

仅安装 Rust 不够。

### 4.3 解决

macOS 安装：

```bash
brew install cmake
```

README 环境要求同步增加 CMake。

### 4.4 Windows

Windows CI 负责验证：

- CMake 可用；
- MSVC 可编译 whisper.cpp；
- Rust 绑定可链接；
- 测试可运行。

最终 Windows CI 完整通过，但 whisper.cpp 使 Rust check/test 时间明显增加。

## 5. 模型选择

选择 Base Q5_1 的原因：

- 比 tiny 质量更稳定；
- 比 small/medium 更适合桌面 CPU；
- 约 57 MB，下载和磁盘成本可接受；
- 支持多语言；
- 官方 whisper.cpp 仓库提供直接模型文件。

没有默认选择 `.en` 模型，因为产品面向中文、英文技术词和混合场景。

## 6. 模型管理

模型目录：

```text
{app_data}/models/ggml-base-q5_1.bin
```

设置页能力：

- 查询模型状态；
- 展示大小和路径；
- 下载；
- 下载进度；
- 验证模型可加载；
- 删除模型。

### 6.1 下载原子性

不能直接写最终文件，否则下载中断后会留下“存在但损坏”的模型。

流程：

```text
目标模型.download
  → 流式写入
  → 发送进度事件
  → sync_all
  → 最小大小校验
  → 删除旧目标
  → rename 到最终文件
```

临时文件使用 Drop guard：任何错误自动清理。

### 6.2 大小校验

至少检查：

```text
下载字节数 > 10 MB
```

这不能替代 checksum，但可以防止 HTML 错误页或空响应被当作模型。

后续应增加官方 SHA256 校验。

### 6.3 活跃会话限制

下载/删除模型时检查 CaptureSession：

- 活跃语音输入时拒绝删除；
- 避免推理期间模型文件消失；
- 避免模型管理和输入链路竞争资源。

## 7. 模型缓存

每次转写都重新加载约 57 MB 模型会带来明显延迟。

使用：

```text
OnceLock<Mutex<HashMap<PathBuf, Arc<WhisperContext>>>>
```

行为：

- canonical path 作为 key；
- 首次加载并缓存；
- 每次请求创建新的 WhisperState；
- 删除模型时清空缓存。

为什么缓存 Context 而不是 State：

- Context 主要包含模型权重；
- State 包含一次推理的中间数据；
- 共享 State 会引入并发和污染问题。

## 8. 音频预处理

Whisper 要求：

```text
16 kHz
mono
f32 PCM
```

而真实麦克风通常产生：

- 44.1 kHz；
- 48 kHz；
- 多声道；
- i16 或其他整数格式。

### 8.1 WAV 读取

支持：

- float WAV；
- 16 bit integer WAV；
- 更高位宽 integer WAV。

整数按位宽归一化到 `[-1, 1]`。

### 8.2 多声道合并

每个 frame 对所有声道取平均：

```text
mono = sum(channels) / channel_count
```

### 8.3 重采样

当前使用线性插值：

```text
source position = output_index * source_rate / 16000
value = left * (1 - fraction) + right * fraction
```

优点：

- 无额外依赖；
- 实现简单；
- 对短语音和首版足够。

限制：

- 不是高质量带限重采样；
- 高频可能出现混叠；
- 后续可以替换为 rubato 等专业库。

### 8.4 MP3 边界

本地首版只支持 WAV。

原因：

- hound 不解码 MP3；
- 引入 ffmpeg/symphonia 会扩大依赖和打包复杂度；
- 应用自身录音已经是 WAV；
- 上传 MP3 可以继续选择云端 Provider。

## 9. 推理参数

使用 Greedy sampling：

```text
best_of = 1
```

线程数：

```text
min(available_parallelism, 8)
```

其他设置：

- 自动检测语言；
- 不翻译；
- no_context；
- 关闭 progress/realtime/timestamp 输出。

没有默认指定中文，因为开发者场景可能包含英文、代码和中英混合。

## 10. whisper.cpp 日志泄漏问题

### 10.1 现象

真实 smoke test 中，即使关闭 print 参数，debug 构建仍输出：

- token；
- 概率；
- 完整识别文本；
- 模型加载细节。

这违反此前建立的敏感日志原则。

### 10.2 解决

调用：

```rust
whisper_rs::install_logging_hooks()
```

whisper-rs 会接管 whisper.cpp 和 ggml log callback。

项目没有启用 `log_backend` 或 `tracing_backend`，因此日志 hook 最终不输出内容。

### 10.3 验证

第一次 smoke test显示 token 和文本；安装 hook 后重新执行，只显示测试通过信息。

这说明第三方原生库也需要安全审查，不能只扫描项目自己的 `println!`。

## 11. ASR 结果扩展

原结果：

```text
AsrTranscription { text }
```

扩展后：

```text
AsrTranscription
  ├─ text
  ├─ provider
  ├─ model
  └─ used_fallback
```

原因：历史记录必须保存实际调用结果。

例如 local 降级到 zhipu 后：

```text
provider = zhipu
model = glm-asr-2512
used_fallback = true
```

## 12. 显式云端降级

配置：

```text
asr_provider = local
allow_cloud_fallback = false
fallback_asr_provider = zhipu | openai
```

流程：

```text
本地转写
  ├─ 成功 → 返回 local
  └─ 失败
      ├─ fallback=false → 返回本地错误
      └─ fallback=true
          → 验证云端 Key/Base URL/模型
          → 调用云端
          → used_fallback=true
```

测试使用缺失模型和空云端 Key，证明：

- fallback 关闭时不会进入云端验证；
- fallback 开启时才要求云端 Key；
- 不会静默网络请求。

## 13. Provider 配置统一

实施时发现 OpenAI ASR 字段漂移：

```text
UI 写 openai_api_key/openai_base_url
处理链路读 asr_api_key/asr_base_url
```

最终 `AppConfig` 提供：

- `selected_asr_config()`；
- `cloud_asr_config(provider)`；
- `selected_text_config()`。

所有路径共享：

- 正常录音；
- 上传音频；
- 就绪检查；
- 历史重新转写；
- 本地 fallback。

## 14. 就绪检查

本地 Provider 就绪条件：

```text
模型文件存在
```

不要求 API Key。

云端 fallback 是否配置不影响本地模型本身就绪；fallback 是可选能力。

设置页显示：

- 模型是否存在；
- 模型大小；
- 下载进度；
- fallback 开关；
- fallback Provider。

## 15. 真实 smoke test

下载：

- 官方 `ggml-base-q5_1.bin`；
- whisper.cpp 官方 JFK WAV 样例。

执行：

```bash
XILUOLIN_LOCAL_ASR_MODEL=/tmp/.../ggml-base-q5_1.bin \
XILUOLIN_LOCAL_ASR_AUDIO=/tmp/.../jfk.wav \
cargo test --manifest-path src-tauri/Cargo.toml \
  --test local_asr_smoke -- --ignored
```

验证：

- 模型可加载；
- WAV 可读取；
- 推理成功；
- 自动语言检测正常；
- 无云端请求；
- 日志 hook 生效。

Mock 测试无法替代这一步，因为 mock 不会验证：

- CMake；
- C++ 链接；
- 模型格式；
- 内存需求；
- whisper.cpp runtime；
- 实际音频预处理。

## 16. Windows CI

引入 whisper-rs 后，Windows CI 时间从数分钟增加到接近十分钟，主要成本是：

- whisper.cpp C/C++ 编译；
- cargo check；
- cargo test 再次构建测试产物。

最终 Windows CI 验证：

- CMake/MSVC 可用；
- whisper-rs 可编译；
- 完整测试通过。

后续应增加 Rust build cache，减少重复 native build。

## 17. 已知限制

- 仅 WAV；
- CPU 推理；
- 无 VAD；
- 无流式 partial transcript；
- 无模型 checksum；
- 无 GPU 自动选择；
- 本地文本整理未实现；
- 模型下载没有暂停/续传；
- 模型大小约 57 MB，但运行时内存显著高于模型文件。

## 18. 后续演进

### 短期

- SHA256 校验；
- 下载超时和断点续传；
- 模型下载镜像；
- 中文样例回归测试；
- 高质量重采样；
- 记录实际推理耗时。

### 中期

- tiny/base 模型选择；
- Metal/CUDA/Vulkan feature；
- VAD；
- 本地模型预热；
- 模型内存占用提示。

### 长期

- 本地文本整理；
- 流式音频和 partial transcript；
- Provider 性能对比；
- 端到端本地隐私模式。

## 19. 相关文件

- `src-tauri/src/local_asr.rs`
- `src-tauri/src/local_asr_model.rs`
- `src-tauri/src/asr.rs`
- `src-tauri/src/readiness.rs`
- `src-tauri/tests/local_asr_provider.rs`
- `src-tauri/tests/local_asr_smoke.rs`
- `src/components/settings/LocalAsrSettings.tsx`
- `docs/dev/archive/plans/local-asr-implementation-plan.md`
