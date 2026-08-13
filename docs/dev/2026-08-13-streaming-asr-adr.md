# ADR：实时语音预览的 ASR 依赖与模型候选

- 日期：2026-08-13
- 状态：Task 0–1 完成；生产模型 **No-Go**
- 基线：`5789d00f94e8857e11e86b3bee2755fbd1987822`
- 范围：独立 CLI 技术验证、模型/许可审计、延迟与跨平台构建验证

## 决策

1. Rust 技术路径采用官方 `sherpa-onnx = "=1.13.5"` crate，spike 使用 `static` feature。`OnlineRecognizer` 能在 Apple Silicon 上完成 100 ms PCM 增量输入、per-stream hotwords、endpoint detection 和结果 revision 输出，API 路径可行。
2. 识别固定为 transducer + `modified_beam_search` + `max_active_paths=4`。官方热词文档明确只有 transducer 的 `modified_beam_search` 支持 hotwords。
3. 两个候选模型当前均不允许进入 Task 2：双语模型的训练数据是未公开的内部数据，许可链不可审计；14M 中文模型虽可追溯到 WenetSpeech，但不满足中英混合和英文技术词需求。两者都缺少私有真实录音质量验证。
4. GLM-Realtime 不作为实时听写主方案。官方把输入音频转录描述为独立模型的参考结果，可能为空且可能与推理结果不同；文档也没有给出本任务所需的原生热词与最终听写语义保证。

本 ADR 原始研究只确认依赖/API 可行性。集成草稿分支在保留生产模型 No-Go 结论的前提下，将固定依赖、模型清单和 UI/IPC 接成默认关闭、显式下载的实验性旁路；模型不打入安装包，也不替代现有最终 ASR。

## 依赖与供应链

| 项目 | 结论 |
| --- | --- |
| crate | [`sherpa-onnx 1.13.5`](https://crates.io/crates/sherpa-onnx/1.13.5)，锁定精确版本 |
| 上游版本 | [`k2-fsa/sherpa-onnx v1.13.5`](https://github.com/k2-fsa/sherpa-onnx/releases/tag/v1.13.5) |
| 代码许可 | Apache-2.0 |
| 本机链接 | macOS arm64 静态库解析、链接和运行通过 |
| macOS x64 | release 构建通过，产物为 `Mach-O 64-bit executable x86_64` |
| Windows x64 | 集成 crate 使用同版本 shared 包，避免官方 static `/MT` 与 whisper.cpp `/MD` 的 CRT 冲突；GitHub Actions 原生链接与测试通过后仍需验证 Tauri 安装包携带 DLL 和目标设备运行 |

`sherpa-onnx-sys` 的构建脚本会按目标平台获取预编译包，但 crate 内部没有对下载资产执行固定摘要校验。若后续进入生产，应在 CI 预取并验证官方发布摘要，通过 `SHERPA_ONNX_ARCHIVE_DIR` 或等价的受控制品流程提供依赖，禁止安装/打包阶段静默联网获取。

## 模型资产审计

实测使用官方说明的量化组合：int8 encoder + fp32 decoder + int8 joiner。文件从模型作者 `csukuangfj` 的 Hugging Face 仓库下载，下载时记录仓库 revision，并以本文 SHA-256 固定本次实际资产；文件只保留在被忽略的 `experiments/streaming-asr-spike/models/`。

### 中英双语 Zipformer

- 模型：[`sherpa-onnx-streaming-zipformer-bilingual-zh-en-2023-02-20`](https://huggingface.co/csukuangfj/sherpa-onnx-streaming-zipformer-bilingual-zh-en-2023-02-20)
- revision：`98590b7ed6443e77b714204da2757d75e1a642f4`
- 权重标注许可：Apache-2.0
- 模型单元：char + BPE；上游报告 AIShell-1 3.04、WenetSpeech TEST_NET 8.97、TEST_MEETING 8.83（modified beam search）
- 训练来源：模型卡指向社区 checkpoint；官方 sherpa 文档说明使用数万小时内部数据。数据清单、来源授权和再分发许可未公开。
- 再分发结论：**No-Go**。权重的 Apache-2.0 标签不能补足内部训练数据的来源与授权证据。

| 文件 | 字节 | SHA-256 |
| --- | ---: | --- |
| encoder int8 | 181,895,032 | `8fa764187a261844f859d7143ebaa563af5d10adfece4c18a8f414c88cba2a9b` |
| decoder fp32 | 13,876,452 | `2e3b5ec371f8899ee6acd829fd753ba45772df57a91bdf37cde3136354e7db7d` |
| joiner int8 | 3,228,404 | `1ed689c5ed19dbaa725d9d191bb4822b5f4855a39e1ffd28cbc1f340d25b2ee0` |
| tokens | 56,317 | `a8e0e4ec53810e433789b54a5c0134a7eaa2ffca595a6334d54c00da858841d3` |
| BPE model | 244,836 | `bcae393dbc5611be5ffa4c7ae0841558978a5a4f484008cb9dff3a2cc97ebe01` |
| BPE vocab | 12,564 | `d0b642f3a2eacd5fadefdeff9e0e1358cab729647cbb7fe58cf738e1f7407029` |

### 轻量中文 Zipformer 14M

- 模型：[`sherpa-onnx-streaming-zipformer-zh-14M-2023-02-23`](https://huggingface.co/csukuangfj/sherpa-onnx-streaming-zipformer-zh-14M-2023-02-23)
- revision：`204ad334e2e683fd295359930cc16fc0432a23ac`
- 权重标注许可：Apache-2.0
- 模型单元：中文字符；官方说明仅支持中文，训练于 WenetSpeech
- 训练来源：WenetSpeech 10,000+ 小时。数据集标注为 CC BY 4.0，但其说明保留原始音视频权利并要求署名，生产分发仍需法务确认权重的归因和通知义务。
- 再分发结论：**No-Go（本任务）**。许可证据比双语候选完整，但能力不覆盖中英混合与英文技术词，且尚未完成法务归因清单。

| 文件 | 字节 | SHA-256 |
| --- | ---: | --- |
| encoder int8 | 21,621,684 | `1c556ea57cec304e55ec4b72e52c1cc098bb01476ed7d90f3de939fe126487b1` |
| decoder fp32 | 7,509,745 | `5ee0f03a2768ff1d5c83ef3a493243c7935d316cd41280037b14783a3467cc78` |
| joiner int8 | 1,795,562 | `a7cf9d82757bdcf786059454495a9ca95e4bd7347f72473fc08d794475c36169` |
| tokens | 48,697 | `8b294db9045d6e5f94647f4c1eec1af4da143a75053c399611444b378ff966ac` |

## Apple Silicon 性能结果

环境为 Apple M4 / 24 GiB / macOS 26.5.1 arm64，2 个 CPU inference threads。每个候选使用仓库提供的公开 WAV，以 `--realtime --repeat 10` 运行；recognizer 在进程内复用、每轮新建 stream。公开样本仅用于工程和延迟比较。

| 指标 | 双语模型 | 中文 14M |
| --- | ---: | ---: |
| 官方 WAV 时长 | 10,053 ms | 5,612 ms |
| recognizer 创建 | 710 ms | 592 ms |
| 首个局部结果，10 次均值 | 1,029 ms | 711 ms |
| 首个局部结果，范围 | 1,017–1,043 ms | 707–719 ms |
| 每轮 revision | 14 | 11 |
| 10 轮进程墙钟 | 101.57 s | 56.87 s |
| 进程 user + sys CPU / 墙钟 | 24.1% | 15.7% |
| 最大 RSS | 419 MiB | 124 MiB |

“recognizer 创建”是新进程中的创建耗时，但操作系统文件缓存未清空，不等同于重启后的物理冷盘测试。CPU 是 `/usr/bin/time -l` 的进程 CPU 时间与墙钟之比；不是整机采样。常驻 RSS 以 10 次重复的最大 RSS 近似记录。

## 热词与技术词结果

- 中文 `西罗林` 能在两个 transducer 候选上创建 per-stream hotword，证明 `modified_beam_search` 热词路径可工作。
- 双语模型直接传入 `Next.js`、`TypeScript`、`ChatGPT`、`XiLuoLin` 时，sherpa 报告 token 无法映射并跳过整组热词。该 char+BPE 模型还需要正确的 `modeling_unit=cjkchar+bpe` 和 `bpe.vocab` 预处理支持；当前固定 CLI 接口没有这些参数，因此这四个词的热词门禁为失败。
- 本机 `say` 合成的技术词 WAV（未跟踪）只作烟雾测试。双语模型无热词输出能辨出 `JS TYPESCRIPT CHAT GPT`，但把 `XiLuoLin` 识别为近似英文音节；中文 14M 对该样本输出中文近音。合成样本不构成准确率结论。

因此，普通话工程路径可行；中英混合和四个指定术语的准确率/热词偏差均未达生产门禁。

## GLM-Realtime 复核

截至 2026-08-13，[官方 GLM-Realtime 文档](https://docs.bigmodel.cn/cn/guide/models/sound-and-video/glm-realtime)将产品定位为实时音视频通话，输出模态为音频。`conversation.item.input_audio_transcription.completed` 的说明明确：转文本来自独立模型，可能与推理结果有出入、可能为空，并且仅作参考。文档未给出原生 hotwords 参数或面向听写的稳定局部 revision / 最终文本契约。因此维持“不作为主方案”。

## 未完成门禁

- `evals/asr/private`、真实录音和 Provider 凭据不存在：无法复测一期真实 CER、延迟与中英混合准确率。
- 未执行 100 次真实录音、麦克风/物理设备稳定性验证。
- 未验证四个指定英文技术词的可用热词预处理方案。
- Windows 仅通过本机 cross `cargo check`；Windows 原生链接、运行和 Tauri 打包未验证。
- x86_64 macOS 只确认 release 构建与产物架构，未在 Intel Mac 上运行。
- 模型许可/NOTICE/归因清单尚未获法务确认；双语候选缺少训练数据授权证据。

在上述门禁关闭前，不推荐生产模型；集成草稿中的 Task 2 代码只能作为默认关闭的实验性旁路供本地体验，不得据此宣称生产就绪或发布模型资产。

## 复现命令摘要

```bash
cargo fmt --all -- --check
cargo test
cargo check
cargo build --release --target x86_64-apple-darwin
file target/x86_64-apple-darwin/release/xiluolin-streaming-asr-spike
cargo check --target x86_64-pc-windows-msvc
```

实跑命令见 `experiments/streaming-asr-spike/README.md`。模型、WAV、转写正文与性能原始输出均不提交仓库。
