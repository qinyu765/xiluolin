# 模型服务配置说明

> **归档状态：** 旧 Provider 配置指南。当前用户配置见 [`../../../usage-guide.md`](../../../usage-guide.md)，当前技术设计见 [`../../../solution-design.md`](../../../solution-design.md)。

## 概述

项目支持两类模型服务的 provider 配置：

### 1. 语音识别服务 (ASR)
- **智谱 AI (GLM-ASR-2512)** - 默认选项，提供免费额度
- **OpenAI (Whisper)** - 需要付费 API Key

### 2. 文本处理服务
- **智谱 AI (GLM-4.7-Flash)** - 默认选项，提供免费额度
- **OpenAI** - 需要付费 API Key

两种服务均使用标准 API 格式，智谱 AI 完全兼容 OpenAI API。

## 配置方式

### 方式一：UI 设置界面（推荐）

1. 打开应用，进入「设置」页面
2. 切换到「模型配置」标签
3. 分别配置「语音识别服务」和「文本整理服务」：
   - 选择服务商（智谱 AI 或 OpenAI）
   - 填写对应的 API Key
   - 确认 Base URL 和模型名称
4. 点击对应的保存按钮

### 方式二：配置文件

配置存储在 `settings.json` 中（位于应用数据目录）：

```json
{
  "asr_provider": "zhipu",
  "asr_api_key": "your-zhipu-api-key",
  "asr_base_url": "https://open.bigmodel.cn/api/paas/v4",
  "asr_model": "glm-asr-2512",
  "openai_asr_model": "whisper-1",
  "text_provider": "zhipu",
  "zhipu_api_key": "your-zhipu-api-key",
  "zhipu_base_url": "https://open.bigmodel.cn/api/paas/v4",
  "zhipu_model": "glm-4.7-flash",
  "openai_api_key": "your-openai-api-key",
  "openai_base_url": "https://api.openai.com/v1",
  "openai_model": "gpt-4o-mini"
}
```

## 语音识别服务 (ASR)

### 智谱 GLM-ASR-2512

#### 获取 API Key
1. 访问 [智谱 AI 开放平台](https://open.bigmodel.cn/)
2. 注册并登录账号
3. 在控制台创建 API Key

#### 默认配置
- **Base URL**: `https://open.bigmodel.cn/api/paas/v4`
- **模型**: `glm-asr-2512`

#### 特点
- 提供免费额度，适合低成本开发和功能验证
- 支持 wav、mp3 格式
- 最大文件大小 25MB

### OpenAI Whisper

#### 获取 API Key
1. 访问 [OpenAI Platform](https://platform.openai.com/)
2. 注册并登录账号
3. 在 API Keys 页面创建密钥

#### 默认配置
- **Base URL**: `https://api.openai.com/v1`
- **模型**: `whisper-1`

#### 特点
- 官方 OpenAI Whisper API
- 需要付费使用
- 识别准确度高

## 文本处理服务

### 智谱 GLM-4.7-Flash

#### 获取 API Key

1. 访问 [智谱 AI 开放平台](https://open.bigmodel.cn/)
2. 注册并登录账号
3. 在控制台创建 API Key

#### 默认配置

- **Base URL**: `https://open.bigmodel.cn/api/paas/v4`
- **模型**: `glm-4.7-flash`
- **Temperature**: `0.3`（代码中固定）

#### 特点

- 完全兼容 OpenAI API 格式
- 提供免费额度，适合低成本开发和功能验证
- 30B 参数，200K 上下文窗口
- 支持工具调用、结构化输出

### OpenAI

#### 获取 API Key

1. 访问 [OpenAI Platform](https://platform.openai.com/)
2. 注册并登录账号
3. 在 API Keys 页面创建密钥

#### 默认配置

- **Base URL**: `https://api.openai.com/v1`
- **模型**: `gpt-4o-mini`
- **Temperature**: `0.3`（代码中固定）

#### 特点

- 官方 OpenAI API
- 需要付费使用
- 稳定性和响应速度较好

## 实现细节

### 代码结构

- **后端配置**: `src-tauri/src/data.rs` - `AppConfig` 结构
- **ASR 处理**: `src-tauri/src/asr.rs` - `AsrConfig` 和多 provider 支持
- **文本处理**: `src-tauri/src/text_polish.rs` - `TextPolishConfig` 和 API 调用
- **前端类型**: `src/types/config.ts` - TypeScript 类型定义
- **UI 界面**: `src/pages/SettingsPage.tsx` - 配置表单

### ASR API 调用流程

1. 用户选择 `asr_provider`（`zhipu` 或 `openai`）
2. 根据 provider 选择对应的配置：
   - 智谱：使用 `asr_api_key`、`asr_base_url`、`asr_model`
   - OpenAI：使用 `openai_api_key`、`openai_base_url`、`openai_asr_model`
3. 构造 `AsrConfig` 传递给 `transcribe_audio_file`
4. 根据 provider 调用对应的实现函数
5. 返回统一的 `AsrTranscription` 结构

### 文本处理 API 调用流程

1. 用户选择 `text_provider`（`zhipu` 或 `openai`）
2. 根据 provider 选择对应的 `api_key`、`base_url`、`model`
3. 构造 `TextPolishConfig` 传递给 `polish_text_with_openai`
4. 发送 POST 请求到 `{base_url}/chat/completions`
5. 使用 Bearer Token 认证
6. 解析 OpenAI 格式的响应

### 降级策略

如果文本处理 API 请求失败：
- 返回原始 ASR 识别文本（trim 后）
- 设置 `used_fallback: true`
- 记录错误信息到 `error_message`

## 测试

运行文本处理测试：

```bash
cd src-tauri
cargo test --test openai_text_polish_provider
```

测试覆盖：
- 默认配置验证
- API Key 缺失检查
- 请求格式和认证
- 降级策略

## 推荐配置组合

### 方案一：全智谱（推荐用于快速配置）
- ASR: 智谱 GLM-ASR-2512
- 文本处理: 智谱 GLM-4.7-Flash
- 优势：免费额度充足，单一 API Key 管理

### 方案二：全 OpenAI
- ASR: OpenAI Whisper
- 文本处理: OpenAI GPT-4o-mini
- 优势：稳定性好，识别准确度高

### 方案三：混合配置
- ASR: 智谱 GLM-ASR-2512（节省成本）
- 文本处理: OpenAI GPT-4o-mini（保证质量）
- 优势：平衡成本和质量

## 注意事项

1. **API Key 安全**：配置文件不会提交到 Git 仓库
2. **免费额度**：智谱 AI 的免费额度有限，生产环境需评估用量
3. **兼容性**：智谱服务均使用 OpenAI 兼容格式，切换无需修改业务逻辑
4. **Temperature**：文本处理当前固定为 0.3，保证稳定性
5. **独立配置**：ASR 和文本处理可以独立选择不同的 provider
