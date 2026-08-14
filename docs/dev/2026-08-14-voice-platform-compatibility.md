# 语音平台架构兼容层

当前 `main` 的前端和设置页继续作为产品主入口，配置存储格式保持扁平
`AppConfig`，这样现有自动保存、密码凭据和旧用户配置不需要迁移。

## 运行时边界

```text
旧设置页 / AppConfig
        │
        ▼
providers::compat
        │  primary + fallbacks
        ▼
Provider Catalog / Adapters
        │
        ▼
ASR、文本处理与历史记录
```

`src-tauri/src/providers/compat.rs` 是唯一的配置翻译入口。它把旧配置转换为
`ProviderRoutingConfig`，因此 Provider 注册表、错误分类和备用路由可以先投入
使用，而不要求前端立刻切换到新架构的嵌套配置。

读取配置时还会识别新架构留下的 `config_version: 2` 路由结构，并一次性转换
为旧的扁平字段后落盘；旧版设置页继续只读写扁平结构。新架构的凭据包也会
映射回旧版 zhipu/openai 凭据，不会因为切回 `main` 而丢失已有 Key。

录音开始时，`CaptureSession` 会尽力保存配置、默认人格和启用热词快照。录音
处理优先使用这份快照；如果本地数据暂时不可读，则保留旧的处理时读取路径，
避免影响录音入口的可用性。

## 后续迁移顺序

1. 在现有设置页中逐步使用 `list_provider_catalog` 渲染 Provider 字段。
2. 为新旧配置增加双向序列化和可回滚迁移，确认旧 `settings.json` 可回滚。
3. 前端消费 `CaptureSnapshot` 后，再将录音 Controller 拆到 `features/capture`。
4. 最后移除兼容层，统一使用嵌套 Provider 路由配置。

在完成第 2 步之前，不应直接把新架构分支的 `AppConfig` 替换到 `main`，否则
会同时破坏设置页字段、凭据读取和旧用户配置。
