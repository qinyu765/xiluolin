# 多 Provider 架构与千问接入设计

## 背景

XiLuoLin 当前已经支持智谱、OpenAI-compatible 与本地 Whisper，但 Provider 选择、字段配置、就绪校验、凭据保存和调用分支分散在 ASR、文本整理、pipeline 与设置页中。继续以厂商条件分支扩展会让每次新增模型都同时修改业务流程和 UI，也难以正确记录 fallback 后实际使用的 Provider。

本次改造同时覆盖 ASR 与文本整理，目标是让新增 Provider 主要由能力 adapter 与 descriptor 完成，而 pipeline 和通用设置页只依赖稳定接口。

## 目标与边界

### 目标

- 使用能力接口、静态注册表和协议级 transport 隔离业务流程与厂商协议。
- 支持 ASR 与文本各自独立的 primary/fallback 调用链，每条链最多三个 Provider。
- 保留 `zhipu`、`openai`、`local`，新增 `qwen-audio`、`qwen3-asr` 与 `qwen`。
- 配置和凭据从固定厂商字段迁移到 capability/provider 动态映射。
- 设置页从注册表元数据渲染常规字段，本地 Whisper 模型管理继续使用专用组件。
- 历史记录保存实际成功的 Provider、模型和 fallback 状态。

### 不在范围内

- 运行时插件、动态代码加载或第三方 adapter 注入。
- FileTrans、WebSocket 实时 ASR、TTS 或 Embedding。
- Provider 内部自动重试、指数退避或后台任务队列。
- 改变历史数据库表结构。

## 架构

```text
设置页 ── list_provider_catalog ── ProviderCatalog
  │                                      │
  └──── AppConfig / Credentials ─────────┘
                     │
                     ▼
              Capability Router
              ├── ASR route
              └── Text route
                     │
          primary → fallback 1 → fallback 2
                     │
                     ▼
              Provider Adapter
              ├── multipart transcription
              ├── OpenAI-compatible chat
              ├── DashScope multimodal
              └── local Whisper
                     │
                     ▼
                 Transport
```

### 能力接口

`AsrProvider` 接收音频输入、模型设置、热词与软提示，返回文本和实际模型。`TextProvider` 接收 ASR 原文、人格提示与热词，返回整理后的文本和实际模型。两者不暴露具体 HTTP payload。

Provider 通过静态 registry 注册。registry 同时保存 adapter 与供前端消费的 descriptor；ID 在同一 capability 内必须唯一。pipeline 只调用 router，不按厂商名称分支。

### Catalog 与字段描述

`ProviderCatalog` 按 ASR 和 Text 输出 `ProviderDescriptor`。descriptor 包含：

- 稳定 ID、展示名、capability、默认 Base URL 与默认模型；
- Key、Base URL、模型与扩展选项字段描述；
- 热词、软提示、本地模型管理、语言提示等 capability；
- 字段必填、类型、选项、数量上限和帮助信息。

扩展选项只使用文本、布尔和字符串列表三种持久化值，避免前后端通过任意 JSON 隐式约定。

## Provider 与协议

### ASR

| ID | 协议 | 默认模型 | 热词语义 |
|---|---|---|---|
| `zhipu` | multipart transcription | `glm-asr-2512` | 原生 `hotwords[]`，稳定去重后最多 100 个 |
| `openai` | multipart transcription | `whisper-1` | 与上下文合并为软提示 |
| `local` | whisper.cpp | 本地模型路径 | 与上下文合并为 `initial_prompt` |
| `qwen-audio` | DashScope 原生同步多模态 | `qwen-audio-3.0-asr-flash` | 原生 vocabulary，稳定去重后最多 100 个、权重 5 |
| `qwen3-asr` | OpenAI-compatible chat completions | `qwen3-asr-flash` | system glossary |

`qwen-audio` 将短音频编码为 Base64 Data URI，adapter 在用户配置的地域 Base URL 后追加原生多模态 endpoint，解析 `output.text`。`language_hints` 最多四项。

`qwen3-asr` 使用 `input_audio` 内容块，可选单语言与 `enable_itn`，默认关闭。adapter 在 Base URL 后追加 `chat/completions`。

### 文本整理

| ID | 协议 | 默认模型 | 额外约束 |
|---|---|---|---|
| `zhipu` | OpenAI-compatible chat completions | 保持现有默认值 | 关闭 thinking |
| `openai` | OpenAI-compatible chat completions | 保持现有默认值 | 保持现有请求语义 |
| `qwen` | OpenAI-compatible chat completions | `qwen3.7-flash` | 固定 `enable_thinking: false` |

OpenAI-compatible Provider 共用请求与响应 transport，但各 adapter 仍负责厂商扩展字段和默认 endpoint。

## 路由与错误语义

`ProviderRoutingConfig` 包含 `primary`、最多两个 `fallbacks` 与按 Provider ID 保存的 `settings`。route 必须满足：

- primary 非空；
- 总长度不超过三项；
- 同一路由不得重复 Provider ID；
- 每个 Provider 每次请求只尝试一次。

router 按固定顺序调用。Provider 特有的配置、格式或大小不兼容可以继续后项；文件不存在、无法读取等全局输入错误立即停止。

错误统一分类为配置、输入不兼容、鉴权、限流、超时、网络、远端失败、本地运行时和无效响应。对外错误只包含 Provider ID、模型、分类、可选 HTTP 状态及截断后的安全消息，不包含 API Key、用户全文、完整音频路径或原始响应体。

ASR 全部失败时返回聚合后的安全错误，并停止后续文本处理。文本 Provider 全部失败时返回 ASR 原文，`used_text_fallback=true`，成功 Provider 与模型留空。任意 secondary 成功时保存实际 Provider/模型并标记 fallback。

## 配置与凭据

### v2 配置

```text
AppConfig
├── config_version: 2
├── asr: ProviderRoutingConfig
└── text: ProviderRoutingConfig

ProviderRoutingConfig
├── primary
├── fallbacks[]
└── settings{provider_id: ProviderSettings}

ProviderSettings
├── api_key            # 只在运行时合并，不写普通设置文件
├── base_url
├── model
└── options{}
```

ASR 与 Text 即使使用同一厂商也分别保存凭据，避免能力间隐式耦合。

### 迁移事务

首次读取旧配置时：

1. 将旧智谱、OpenAI 与本地 ASR 字段映射到 `asr.settings` 和 route。
2. 将旧智谱与 OpenAI 文本字段映射到 `text.settings` 和 route。
3. 仅当旧 `allow_cloud_fallback=true` 时，把相应云 Provider 转换为 local 后的单一 fallback。
4. 将旧 Keychain bundle 转为 capability/provider 动态映射；旧 OpenAI Key 分别复制到 ASR 和 Text。
5. 先保存脱敏 v2 配置与新凭据，全部成功后才清理旧结构。

任一步失败都保留旧数据并返回错误。普通设置文件永远只保存脱敏配置。

## 设置页与隐私

`ModelSettings` 从 catalog 渲染 Key、Base URL、模型、select、multi-select 与 switch，并提供 primary 选择、fallback 排序、删除和长度校验。descriptor 声明 local capability 时挂载现有本地模型管理组件。

用户把云 Provider 主动加入本地 ASR 的 fallback 链时，设置页在保存前显示隐私提示并要求确认。确认后的 route 即为上传授权，不增加每次调用弹窗或额外总开关。readiness 根据 descriptor 校验整条 route，并指出配置不完整的具体 Provider。

## 测试策略

- registry：ID 唯一、默认值、能力描述和未知 Provider。
- routing：去重、长度、顺序、单次尝试、secondary 成功和全部失败。
- adapters：使用本地 HTTP 服务校验 URL、鉴权、multipart/Base64、热词、语言和响应解析。
- errors：401、429、5xx、超时、无效 JSON、空文本与脱敏。
- migration：旧配置/Keychain 迁移、回滚和普通配置脱敏。
- pipeline：ASR 终止语义、文本原文降级、实际 Provider/模型与重新处理。
- frontend：catalog 字段、route 排序和上限、隐私确认、本地模型操作与配置隔离。
- smoke：真实千问测试标记为 `#[ignore]`，只读取环境变量和本地样本路径。

完整验证执行 `pnpm bindings:generate`、`pnpm check` 与 `git diff --check`。

## 提交与集成

实现位于 `feat/multi-provider-architecture` 的独立 worktree，按设计、核心注册表、千问 adapter、配置迁移、UI/pipeline 和文档拆分小提交。不 push、不创建 PR、不合并，等待维护者明确指令。
