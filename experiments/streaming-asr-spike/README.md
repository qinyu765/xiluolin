# 实时 ASR 独立验证工具

这个工程只验证 `sherpa-onnx` 的 Rust API、PCM 增量输入、热词流和延迟指标，**不属于 Tauri 生产 crate**，也不接入麦克风、IPC、前端或悬浮窗。

## 运行

```bash
cargo run --release -- \
  --encoder /path/to/encoder.int8.onnx \
  --decoder /path/to/decoder.int8.onnx \
  --joiner /path/to/joiner.int8.onnx \
  --tokens /path/to/tokens.txt \
  --wav /path/to/sample.wav \
  --hotword Next.js \
  --hotword XiLuoLin \
  --realtime \
  --repeat 10
```

参数必须显式提供 encoder、decoder、joiner 和 tokens；工具不会根据文件名猜测模型类型。热词会 trim、过滤空值并按首次出现顺序去重。

输入 WAV 被切成 100 ms PCM 块。识别器固定使用 transducer、`modified_beam_search`、`max_active_paths=4`、per-stream hotwords 和 endpoint detection。末尾追加 300 ms 静音后调用 `input_finished()`，避免遗漏最后一段结果。

输出字段包括：

- `model_load_ms`：创建 recognizer 的耗时；
- `revision`：非空且发生变化的局部结果序号；
- `audio_ms` / `wall_ms`：已送入的原始音频时长和墙钟时间；
- `first_partial_ms`：从本轮开始到首个非空局部结果的墙钟延迟；
- `max_update_interval_ms`：相邻文本更新的最大墙钟间隔；
- `endpoint` / `segment_final`：sherpa endpoint 与结果 final 标记。

`--realtime` 会按音频时间节奏送入；不带该参数则用于离线吞吐对比。`--repeat` 复用同一个 recognizer 并为每轮创建独立 stream。

## 本地资产与边界

`models/`、`fixtures/`、WAV、发布包和构建产物均被本目录 `.gitignore` 排除。模型、测试音频和转写正文不得提交。公开或合成音频只用于工程和延迟比较，不能据此宣称真实场景识别质量达标。

## 验证

```bash
cargo fmt --all -- --check
cargo test
cargo check
```
