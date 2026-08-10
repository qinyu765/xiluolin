import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import {
  evaluateBenchmark,
  levenshteinDistance,
  modelReplacementEligible,
  normalizeForCer,
  percentile,
  readWavDurationMs,
  validateDataset,
} from "./asr-eval.mjs";

function pcmWav(durationMs) {
  const sampleRate = 16_000;
  const dataBytes = Math.round((sampleRate * durationMs) / 1_000) * 2;
  const buffer = Buffer.alloc(44 + dataBytes);
  buffer.write("RIFF", 0, "ascii");
  buffer.writeUInt32LE(36 + dataBytes, 4);
  buffer.write("WAVEfmt ", 8, "ascii");
  buffer.writeUInt32LE(16, 16);
  buffer.writeUInt16LE(1, 20);
  buffer.writeUInt16LE(1, 22);
  buffer.writeUInt32LE(sampleRate, 24);
  buffer.writeUInt32LE(sampleRate * 2, 28);
  buffer.writeUInt16LE(2, 32);
  buffer.writeUInt16LE(16, 34);
  buffer.write("data", 36, "ascii");
  buffer.writeUInt32LE(dataBytes, 40);
  return buffer;
}

test("CER 规范化会移除标点空白并统一宽度和大小写", () => {
  assert.deepEqual(normalizeForCer("ＸiLuoLin，你好！"), [
    "x",
    "i",
    "l",
    "u",
    "o",
    "l",
    "i",
    "n",
    "你",
    "好",
  ]);
  assert.equal(levenshteinDistance(["你", "好"], ["你", "号"]), 1);
});

test("百分位采用 nearest-rank", () => {
  assert.equal(percentile([4, 1, 3, 2], 0.5), 2);
  assert.equal(percentile([4, 1, 3, 2], 0.95), 4);
  assert.equal(percentile([], 0.95), null);
});

test("从 WAV 数据块校验真实录音时长", () => {
  const directory = fs.mkdtempSync(
    path.join(os.tmpdir(), "xiluolin-asr-eval-"),
  );
  const filePath = path.join(directory, "sample.wav");
  try {
    fs.writeFileSync(filePath, pcmWav(3_250));
    assert.equal(readWavDurationMs(filePath), 3_250);
    const categories = [
      "mandarin",
      "mixed_language",
      "hotword",
      "noise",
      "far_field",
    ];
    const manifest = categories.map((category, index) => ({
      id: `wav-${index}`,
      audio_path: "sample.wav",
      duration_ms: 3_250,
      category,
      reference: "真实时长。",
      hotwords: [],
      processing_mode: index % 2 === 0 ? "verbatim" : "polish",
    }));
    validateDataset(manifest, { minSamples: 5, manifestDir: directory });
    manifest[0].duration_ms = 4_000;
    assert.throws(
      () =>
        validateDataset(manifest, { minSamples: 5, manifestDir: directory }),
      /相差超过 250ms/,
    );
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test("完整预测会通过质量和延迟门槛", () => {
  const categories = [
    "mandarin",
    "mixed_language",
    "hotword",
    "noise",
    "far_field",
  ];
  const manifest = categories.map((category, index) => ({
    id: `sample-${index}`,
    audio_path: `audio/${index}.wav`,
    duration_ms: 5_000,
    category,
    reference: "今天测试 XiLuoLin，效果很好！",
    hotwords: ["XiLuoLin", "XiLuoLin", ""],
    processing_mode: index % 2 === 0 ? "verbatim" : "polish",
  }));
  validateDataset(manifest, { minSamples: 5, requireAudioFiles: false });
  const predictions = manifest.map((sample, index) => ({
    id: sample.id,
    transcript: sample.reference,
    processing_mode: sample.processing_mode,
    fn_released_at_ms: 1_000,
    pasted_at_ms: index % 2 === 0 ? 4_000 : 7_000,
  }));

  const report = evaluateBenchmark(manifest, predictions);

  assert.equal(report.metrics.cer, 0);
  assert.equal(report.metrics.hotword_recall, 1);
  assert.equal(report.metrics.punctuation_f1, 1);
  assert.equal(report.accepted, true);
});

test("缺类别、短数据集和模型替换不足都会被拒绝", () => {
  assert.throws(
    () =>
      validateDataset([], {
        minSamples: 80,
        requireAudioFiles: false,
      }),
    /至少需要 80 条/,
  );
  const candidate = {
    cer: 0.07,
    hotword_recall: 0.96,
    latency: {
      verbatim: { p95_ms: 3_000 },
      polish: { p95_ms: 6_000 },
    },
  };
  const baseline = { cer: 0.08, hotword_recall: 0.96 };
  assert.equal(modelReplacementEligible(candidate, baseline), false);
  assert.equal(
    modelReplacementEligible({ ...candidate, cer: 0.068 }, baseline),
    true,
  );
});

test("评测拒绝基准集之外的预测结果", () => {
  const manifest = [
    {
      id: "known",
      audio_path: "audio/known.wav",
      duration_ms: 5_000,
      category: "mandarin",
      reference: "已知样本。",
      hotwords: [],
      processing_mode: "verbatim",
    },
  ];
  assert.throws(
    () =>
      evaluateBenchmark(manifest, [
        {
          id: "known",
          transcript: "已知样本。",
          processing_mode: "verbatim",
          fn_released_at_ms: 0,
          pasted_at_ms: 1,
        },
        { id: "extra", transcript: "额外样本" },
      ]),
    /基准集之外/,
  );
});
