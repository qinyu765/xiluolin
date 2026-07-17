# 本地轻量 ASR 方案实施计划

> **归档状态：未实施方案。** 本地离线 ASR 仍是路线图候选项，不代表当前已经提供该能力。

## 背景

当前项目使用远端 ASR API（智谱 GLM-ASR-2512 和 OpenAI Whisper），存在网络依赖和稳定性问题。需要添加本地 Whisper 模型支持，实现远端 + 本地双模式，提供离线能力和更好的隐私保护。

**旧项目经验**（D:\For coding\project\Agents\example\time）：
- 使用 faster-whisper（Python），base 模型 142MB，INT8 量化
- 单例加载 + VAD 过滤，CPU 推理
- 适合演示场景，性能表现良好

**当前项目现状**：
- Rust Tauri 架构，远端 API 调用
- 音频格式：单声道 WAV (16-bit PCM)
- 处理流程：录音 → ASR → 文本整理 → 输出
- 无本地 ML 依赖，无重试机制

## 技术选型

### 本地 ASR 引擎：whisper-rs

**选择理由**：
- 基于 whisper.cpp，性能优化成熟（SIMD、量化支持）
- Rust 原生绑定，与 Tauri 技术栈一致
- 支持 GGML 量化模型，体积小（base 模型量化后 ~75MB）
- CPU 推理友好，无需 GPU 依赖

### 模型选择：Whisper Base (INT8 量化)

- 原始大小：~140MB，量化后：~75MB
- 准确率：适合短语音（<30s）场景
- 推理速度：CPU 上 1-3 秒（取决于音频长度）

## 架构设计

### 核心原则

1. **最小侵入性**：复用现有 `AsrConfig` 和 `transcribe_audio_file` 接口
2. **渐进式加载**：模型按需下载和加载，避免影响启动速度
3. **优雅降级**：本地失败自动回退到远端 API
4. **跨平台兼容**：考虑 Windows/macOS/Linux 编译和二进制体积

### 降级流程

```
用户触发语音输入
    ↓
检查 ASR Provider
    ↓
Provider = Local?
    ↓ 是
检查模型文件存在
    ↓
加载模型（首次 2-3s，后续复用）
    ↓
执行推理
    ↓
成功？
    ↓ 否
启用降级？
    ↓ 是
调用远端 API（智谱/OpenAI）
    ↓
返回结果
```

## 关键文件修改清单

### 1. 依赖变更

**文件：`src-tauri/Cargo.toml`**

添加依赖：
```toml
whisper-rs = "0.12"      # Whisper 推理引擎
once_cell = "1.19"       # 全局单例缓存
futures-util = "0.3"     # 异步流处理（用于下载进度）
```

更新 reqwest 特性：
```toml
reqwest = { version = "0.12", features = ["blocking", "multipart", "json", "stream"] }
```

### 2. ASR 模块重构

**文件：`src-tauri/src/asr.rs`**

修改内容：
- 将 `provider` 字段从 `String` 改为枚举 `AsrProvider::Local | Zhipu | OpenAI`
- 扩展 `AsrConfig` 添加 `local_model_path`、`enable_fallback`、`fallback_provider` 字段
- 修改 `transcribe_audio_file` 添加本地分支和降级逻辑
- 扩展 `AsrError` 添加本地模型相关错误类型

### 3. 新增本地 Whisper 模块

**新建文件：`src-tauri/src/asr/local_whisper.rs`**

实现功能：
- 全局单例模型缓存（使用 `once_cell::Lazy`）
- `transcribe_with_local_whisper` 函数：加载模型 → 读取音频 → 推理 → 提取文本
- `load_audio_as_f32` 函数：将 WAV 文件转换为 whisper-rs 需要的 f32 数组
- 音频格式验证：确保 16kHz 单声道

### 4. 模型管理模块

**新建文件：`src-tauri/src/asr/model_manager.rs`**

实现功能：
- `download_whisper_model` 命令：从 Hugging Face 下载模型，支持进度回调
- `check_model_exists` 命令：检查模型文件是否存在
- `get_model_info` 命令：获取模型信息（大小、路径）
- 模型存储路径：`{app_data_dir}/models/ggml-base.bin`

### 5. 配置层扩展

**文件：`src-tauri/src/data.rs`**

扩展 `AppConfig` 结构：
```rust
pub local_asr_enabled: bool,
pub local_model_name: String,           // 默认 "ggml-base.bin"
pub enable_asr_fallback: bool,
pub fallback_asr_provider: String,      // 默认 "zhipu"
```

**文件：`src/types/config.ts`**

同步前端类型定义，添加对应字段。

### 6. UI 实现

**文件：`src/pages/SettingsPage.tsx`**

在"模型配置"标签页添加"本地 ASR"卡片：
- 启用本地 ASR 开关
- 模型状态显示（已下载/未下载，文件大小）
- 下载模型按钮（带进度条）
- 降级配置开关和目标选择
- 性能说明提示

**文件：`src/pages/SettingsPage.tsx`**

在现有设置页的模型配置区域添加本地 ASR 卡片，并实现前端逻辑：
- 检查模型状态（`get_model_info`）
- 监听下载进度事件（`model-download-progress`）
- 下载模型处理（`download_whisper_model`）

### 7. 命令注册

**文件：`src-tauri/src/main.rs`**

注册新的 Tauri 命令：
```rust
.invoke_handler(tauri::generate_handler![
    // 现有命令...
    download_whisper_model,
    check_model_exists,
    get_model_info,
])
```

## 实施步骤

### 阶段 1：基础架构（预计 2-3 小时）

1. 添加 `whisper-rs`、`once_cell`、`futures-util` 依赖
2. 创建 `src-tauri/src/asr/local_whisper.rs` 模块
3. 扩展 `AsrProvider` 枚举和 `AsrConfig` 结构
4. 修改 `transcribe_audio_file` 添加本地分支
5. 扩展 `AsrError` 错误类型
6. 编写单元测试验证编译通过

**验证方法**：
- `cargo build` 编译通过
- 单元测试通过（模拟本地推理失败场景）

### 阶段 2：模型管理（预计 2-3 小时）

1. 创建 `src-tauri/src/asr/model_manager.rs`
2. 实现模型下载命令（支持进度回调）
3. 实现模型检查和信息查询命令
4. 在 `main.rs` 注册新命令
5. 测试模型下载流程

**验证方法**：
- 手动调用 `download_whisper_model` 命令
- 验证模型文件下载到正确路径
- 验证进度事件正常触发

### 阶段 3：配置层集成（预计 1-2 小时）

1. 扩展 `AppConfig` 添加本地 ASR 字段
2. 更新前端类型定义 `src/types/config.ts`
3. 修改 `pipeline.rs` 读取本地配置并传递给 ASR 模块
4. 测试配置读写

**验证方法**：
- 修改配置后重启应用，验证配置持久化
- 检查 `config.json` 文件包含新字段

### 阶段 4：UI 实现（预计 2-3 小时）

1. 在设置页面添加"本地 ASR"卡片
2. 实现模型状态检查逻辑
3. 实现下载按钮和进度条
4. 实现降级配置 UI
5. 添加性能说明提示

**验证方法**：
- 打开设置页面，验证 UI 正常显示
- 点击下载按钮，验证进度条更新
- 下载完成后，验证状态更新为"已下载"

### 阶段 5：端到端测试（预计 1-2 小时）

1. 启用本地 ASR，录制音频并验证识别结果
2. 测试本地推理失败时的降级逻辑
3. 测试音频格式错误时的错误提示
4. 测试首次推理和后续推理的性能差异
5. 测试跨平台编译（Windows/macOS/Linux）

**验证方法**：
- 录制 5 秒中文语音，验证识别准确率
- 删除模型文件，验证降级到远端 API
- 上传非 16kHz 音频，验证错误提示
- 记录首次推理和后续推理的耗时

## 性能预期

- **模型加载时间**：首次 2-3 秒，后续复用缓存
- **推理时间**：10 秒音频约 1-2 秒
- **内存占用**：模型加载后 +200MB
- **二进制体积增量**：+5MB（whisper-rs 依赖）
- **模型文件大小**：~75MB（GGML 量化）

## 风险和注意事项

1. **编译依赖**：whisper-rs 需要 C++ 编译器（Windows 需要 MSVC，macOS/Linux 需要 clang）
2. **音频格式限制**：whisper-rs 要求 16kHz 单声道，当前录音模块使用设备默认采样率，可能需要重采样
3. **首次体验**：首次推理需要加载模型，用户可能感觉卡顿，需要在 UI 中明确提示
4. **模型下载失败**：网络问题可能导致下载失败，需要提供重试机制
5. **跨平台测试**：需要在 Windows/macOS/Linux 上分别测试编译和运行

## 后续优化方向

1. **音频重采样**：如果录音采样率不是 16kHz，自动重采样而非报错
2. **模型选择**：支持 tiny/base/small 多个模型，让用户根据性能需求选择
3. **流式推理**：支持长音频分段推理，避免内存占用过高
4. **GPU 加速**：可选启用 CUDA/Metal 加速（需要额外依赖）
5. **模型缓存预热**：应用启动时预加载模型，减少首次推理延迟
