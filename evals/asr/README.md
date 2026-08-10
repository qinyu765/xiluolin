# ASR 基准集工作区

该目录只提交结构、80 条录制脚本和指标工具，不提交真人录音、真实转写结果或业务热词。私有数据统一放在 `evals/asr/private/`，该目录已被 Git 忽略。

## 建立数据集

1. 将 `manifest.template.jsonl` 复制为 `private/manifest.jsonl`。
2. 至少邀请两名说话人按模板录制 3–20 秒 WAV，保存在 `private/audio/`。模板的 `duration_ms` 只是录制目标，必须替换成文件的实际时长；工具会读取 WAV 并拒绝相差超过 250ms 的声明值。
3. 普通话、中英混合、专业热词、噪声、远场五类各保留至少一条；模板已经按每类 16 条分配。
4. `reference` 必须由人工听写复核；`hotwords` 只列该条样本期望精确命中的词。
5. 录音中不要包含真实 API Key、客户信息或不必要的个人信息。需要跨成员共享时，使用团队批准的受控存储，不要提交到 Git。

字段契约见 `manifest.schema.json`。基准脚本会拒绝少于 80 条、时长超出范围、类别缺失、ID 重复或音频文件不存在的数据集。

## 收集结果

每轮运行生成 `private/predictions-<model>-<round>.jsonl`，每行对应一条样本：

```json
{"id":"mandarin-01","transcript":"请把今天下午三点的产品评审改到四点半，并通知所有参会同事。","processing_mode":"verbatim","fn_released_at_ms":1000,"pasted_at_ms":3850}
```

- `transcript` 使用最终完整转写；原文模式不得把润色结果当作 ASR 结果。
- 两个时间戳使用同一个单调时钟，统计“松开 Fn → 文本投递完成”；不要混用 Unix 时间和 `performance.now()`。
- 测量云端模型时固定网络、设备、麦克风、热词顺序和应用 commit；每个模型至少连续跑两轮。
- 文件契约见 `predictions.schema.json`。工具只输出聚合统计，不回显单条正文。

## 计算指标

```bash
pnpm eval:asr \
  --manifest evals/asr/private/manifest.jsonl \
  --predictions evals/asr/private/predictions-glm-asr-2512-r1.jsonl \
  --output evals/asr/private/results/glm-asr-2512-r1.json
```

输出包括去标点 CER、热词精确召回率、标点 F1，以及原文/润色模式的 P50/P95。默认门槛为：

- CER ≤ 8%
- 热词精确召回率 ≥ 95%
- 标点 F1 ≥ 85%
- 原文 P95 ≤ 4 秒
- 润色 P95 ≤ 7 秒

评估候选模型时再传入基线报告：

```bash
pnpm eval:asr \
  --manifest evals/asr/private/manifest.jsonl \
  --predictions evals/asr/private/predictions-candidate-r1.jsonl \
  --baseline evals/asr/private/results/glm-asr-2512-r1.json
```

只有候选 CER 相对降低至少 15%、热词召回不低于基线且仍满足 95% 门槛，并继续满足延迟目标时，工具才会给出“满足默认模型替换门槛”。是否切换默认模型仍需两轮结果和人工确认。
