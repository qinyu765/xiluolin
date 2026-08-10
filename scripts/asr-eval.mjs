import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const REQUIRED_CATEGORIES = [
  "mandarin",
  "mixed_language",
  "hotword",
  "noise",
  "far_field",
];

export const ACCEPTANCE_THRESHOLDS = Object.freeze({
  cer: 0.08,
  hotwordRecall: 0.95,
  punctuationF1: 0.85,
  verbatimP95Ms: 4_000,
  polishP95Ms: 7_000,
});

export function normalizeForCer(text) {
  return Array.from(
    String(text)
      .normalize("NFKC")
      .toLocaleLowerCase()
      .replace(/[\p{P}\s]/gu, ""),
  );
}

export function levenshteinDistance(left, right) {
  const previous = Array.from(
    { length: right.length + 1 },
    (_, index) => index,
  );
  for (let leftIndex = 1; leftIndex <= left.length; leftIndex += 1) {
    const current = [leftIndex];
    for (let rightIndex = 1; rightIndex <= right.length; rightIndex += 1) {
      const substitutionCost =
        left[leftIndex - 1] === right[rightIndex - 1] ? 0 : 1;
      current[rightIndex] = Math.min(
        previous[rightIndex] + 1,
        current[rightIndex - 1] + 1,
        previous[rightIndex - 1] + substitutionCost,
      );
    }
    previous.splice(0, previous.length, ...current);
  }
  return previous[right.length];
}

function punctuationTokens(text) {
  let anchor = 0;
  const occurrences = new Map();
  const tokens = [];
  for (const character of Array.from(String(text).normalize("NFKC"))) {
    if (/\s/u.test(character)) continue;
    if (/\p{P}/u.test(character)) {
      const base = `${anchor}:${character}`;
      const occurrence = (occurrences.get(base) ?? 0) + 1;
      occurrences.set(base, occurrence);
      tokens.push(`${base}:${occurrence}`);
    } else {
      anchor += 1;
    }
  }
  return tokens;
}

function countPunctuationMatches(reference, prediction) {
  const referenceTokens = punctuationTokens(reference);
  const predictionTokens = punctuationTokens(prediction);
  const referenceSet = new Set(referenceTokens);
  const matches = predictionTokens.filter((token) =>
    referenceSet.has(token),
  ).length;
  return {
    matches,
    referenceCount: referenceTokens.length,
    predictionCount: predictionTokens.length,
  };
}

export function percentile(values, percentileValue) {
  if (values.length === 0) return null;
  const sorted = [...values].sort((left, right) => left - right);
  const index = Math.max(
    0,
    Math.min(sorted.length - 1, Math.ceil(percentileValue * sorted.length) - 1),
  );
  return sorted[index];
}

export function readWavDurationMs(filePath) {
  const buffer = fs.readFileSync(filePath);
  if (
    buffer.length < 12 ||
    buffer.toString("ascii", 0, 4) !== "RIFF" ||
    buffer.toString("ascii", 8, 12) !== "WAVE"
  ) {
    throw new Error(`不是有效的 RIFF/WAVE 文件：${filePath}`);
  }
  let byteRate = null;
  let dataBytes = null;
  let offset = 12;
  while (offset + 8 <= buffer.length) {
    const chunkId = buffer.toString("ascii", offset, offset + 4);
    const chunkSize = buffer.readUInt32LE(offset + 4);
    const chunkStart = offset + 8;
    if (chunkStart + chunkSize > buffer.length) {
      throw new Error(`WAV chunk 超出文件范围：${filePath}`);
    }
    if (chunkId === "fmt " && chunkSize >= 12) {
      byteRate = buffer.readUInt32LE(chunkStart + 8);
    } else if (chunkId === "data") {
      dataBytes = chunkSize;
    }
    offset = chunkStart + chunkSize + (chunkSize % 2);
  }
  if (!byteRate || dataBytes === null) {
    throw new Error(`WAV 缺少 fmt 或 data chunk：${filePath}`);
  }
  return Math.round((dataBytes / byteRate) * 1_000);
}

function stableUniqueHotwords(hotwords) {
  const seen = new Set();
  const result = [];
  for (const value of hotwords ?? []) {
    const hotword = String(value).trim().normalize("NFKC");
    if (!hotword || seen.has(hotword)) continue;
    seen.add(hotword);
    result.push(hotword);
  }
  return result;
}

function latencySummary(values) {
  return {
    samples: values.length,
    p50_ms: percentile(values, 0.5),
    p95_ms: percentile(values, 0.95),
  };
}

export function validateDataset(
  manifest,
  {
    minSamples = 80,
    requireAudioFiles = true,
    manifestDir = process.cwd(),
  } = {},
) {
  if (manifest.length < minSamples) {
    throw new Error(
      `基准集至少需要 ${minSamples} 条，当前只有 ${manifest.length} 条`,
    );
  }
  const ids = new Set();
  const categories = new Set();
  for (const sample of manifest) {
    if (!sample.id || ids.has(sample.id)) {
      throw new Error(`基准样本 id 缺失或重复：${sample.id ?? "<empty>"}`);
    }
    ids.add(sample.id);
    if (!sample.reference?.trim()) {
      throw new Error(`样本 ${sample.id} 缺少 reference`);
    }
    if (normalizeForCer(sample.reference).length === 0) {
      throw new Error(`样本 ${sample.id} 的 reference 不能只有标点或空白`);
    }
    if (
      !Number.isInteger(sample.duration_ms) ||
      sample.duration_ms < 3_000 ||
      sample.duration_ms > 20_000
    ) {
      throw new Error(`样本 ${sample.id} 时长必须在 3000–20000ms 内`);
    }
    if (!REQUIRED_CATEGORIES.includes(sample.category)) {
      throw new Error(`样本 ${sample.id} 使用未知类别：${sample.category}`);
    }
    categories.add(sample.category);
    if (!Array.isArray(sample.hotwords)) {
      throw new Error(`样本 ${sample.id} 的 hotwords 必须是数组`);
    }
    if (
      sample.processing_mode !== "verbatim" &&
      sample.processing_mode !== "polish"
    ) {
      throw new Error(`样本 ${sample.id} 的 processing_mode 无效`);
    }
    if (!sample.audio_path) {
      throw new Error(`样本 ${sample.id} 缺少 audio_path`);
    }
    if (requireAudioFiles) {
      const audioPath = path.resolve(manifestDir, sample.audio_path);
      if (!fs.existsSync(audioPath)) {
        throw new Error(`样本 ${sample.id} 的音频不存在：${audioPath}`);
      }
      if (path.extname(audioPath).toLocaleLowerCase() !== ".wav") {
        throw new Error(`样本 ${sample.id} 必须使用 WAV：${audioPath}`);
      }
      const actualDurationMs = readWavDurationMs(audioPath);
      if (actualDurationMs < 3_000 || actualDurationMs > 20_000) {
        throw new Error(
          `样本 ${sample.id} 的实际 WAV 时长必须在 3000–20000ms 内`,
        );
      }
      if (Math.abs(actualDurationMs - sample.duration_ms) > 250) {
        throw new Error(
          `样本 ${sample.id} 的 duration_ms 与 WAV 相差超过 250ms：声明 ${sample.duration_ms}，实际 ${actualDurationMs}`,
        );
      }
    }
  }
  const missingCategories = REQUIRED_CATEGORIES.filter(
    (category) => !categories.has(category),
  );
  if (missingCategories.length > 0) {
    throw new Error(`基准集缺少类别：${missingCategories.join(", ")}`);
  }
}

export function evaluateBenchmark(manifest, predictions) {
  const predictionById = new Map(
    predictions.map((prediction) => [prediction.id, prediction]),
  );
  if (predictionById.size !== predictions.length) {
    throw new Error("预测结果包含重复 id");
  }

  let referenceCharacters = 0;
  let editDistance = 0;
  let expectedHotwords = 0;
  let matchedHotwords = 0;
  let punctuationMatches = 0;
  let referencePunctuation = 0;
  let predictionPunctuation = 0;
  const latencyByMode = { verbatim: [], polish: [] };

  for (const sample of manifest) {
    const prediction = predictionById.get(sample.id);
    if (!prediction || typeof prediction.transcript !== "string") {
      throw new Error(`样本 ${sample.id} 缺少 transcript 预测结果`);
    }
    const reference = normalizeForCer(sample.reference);
    const transcript = normalizeForCer(prediction.transcript);
    referenceCharacters += reference.length;
    editDistance += levenshteinDistance(reference, transcript);

    const normalizedTranscript = prediction.transcript.normalize("NFKC");
    for (const hotword of stableUniqueHotwords(sample.hotwords)) {
      expectedHotwords += 1;
      if (normalizedTranscript.includes(hotword)) matchedHotwords += 1;
    }

    const punctuation = countPunctuationMatches(
      sample.reference,
      prediction.transcript,
    );
    punctuationMatches += punctuation.matches;
    referencePunctuation += punctuation.referenceCount;
    predictionPunctuation += punctuation.predictionCount;

    const mode = prediction.processing_mode ?? sample.processing_mode;
    const releasedAt = prediction.fn_released_at_ms;
    const pastedAt = prediction.pasted_at_ms;
    if (
      (mode === "verbatim" || mode === "polish") &&
      Number.isFinite(releasedAt) &&
      Number.isFinite(pastedAt) &&
      pastedAt >= releasedAt
    ) {
      latencyByMode[mode].push(pastedAt - releasedAt);
    }
  }

  const manifestIds = new Set(manifest.map((sample) => sample.id));
  const extraPrediction = predictions.find(
    (prediction) => !manifestIds.has(prediction.id),
  );
  if (extraPrediction) {
    throw new Error(`预测结果包含基准集之外的 id：${extraPrediction.id}`);
  }

  const punctuationPrecision =
    predictionPunctuation === 0
      ? 0
      : punctuationMatches / predictionPunctuation;
  const punctuationRecall =
    referencePunctuation === 0 ? 0 : punctuationMatches / referencePunctuation;
  const punctuationF1 =
    punctuationPrecision + punctuationRecall === 0
      ? 0
      : (2 * punctuationPrecision * punctuationRecall) /
        (punctuationPrecision + punctuationRecall);
  const metrics = {
    samples: manifest.length,
    cer: referenceCharacters === 0 ? 0 : editDistance / referenceCharacters,
    hotword_recall:
      expectedHotwords === 0 ? null : matchedHotwords / expectedHotwords,
    punctuation_f1: punctuationF1,
    latency: {
      verbatim: latencySummary(latencyByMode.verbatim),
      polish: latencySummary(latencyByMode.polish),
    },
    counts: {
      reference_characters: referenceCharacters,
      edit_distance: editDistance,
      expected_hotwords: expectedHotwords,
      matched_hotwords: matchedHotwords,
      reference_punctuation: referencePunctuation,
      prediction_punctuation: predictionPunctuation,
      matched_punctuation: punctuationMatches,
    },
  };

  const checks = {
    cer: metrics.cer <= ACCEPTANCE_THRESHOLDS.cer,
    hotword_recall:
      metrics.hotword_recall !== null &&
      metrics.hotword_recall >= ACCEPTANCE_THRESHOLDS.hotwordRecall,
    punctuation_f1:
      metrics.punctuation_f1 >= ACCEPTANCE_THRESHOLDS.punctuationF1,
    verbatim_p95:
      metrics.latency.verbatim.p95_ms !== null &&
      metrics.latency.verbatim.p95_ms <= ACCEPTANCE_THRESHOLDS.verbatimP95Ms,
    polish_p95:
      metrics.latency.polish.p95_ms !== null &&
      metrics.latency.polish.p95_ms <= ACCEPTANCE_THRESHOLDS.polishP95Ms,
  };
  return { metrics, checks, accepted: Object.values(checks).every(Boolean) };
}

export function modelReplacementEligible(candidateReport, baselineReport) {
  const candidate = candidateReport.metrics ?? candidateReport;
  const baseline = baselineReport.metrics ?? baselineReport;
  return (
    candidate.cer <= baseline.cer * 0.85 &&
    candidate.hotword_recall !== null &&
    candidate.hotword_recall >= ACCEPTANCE_THRESHOLDS.hotwordRecall &&
    baseline.hotword_recall !== null &&
    candidate.hotword_recall >= baseline.hotword_recall &&
    candidate.latency.verbatim.p95_ms !== null &&
    candidate.latency.verbatim.p95_ms <= ACCEPTANCE_THRESHOLDS.verbatimP95Ms &&
    candidate.latency.polish.p95_ms !== null &&
    candidate.latency.polish.p95_ms <= ACCEPTANCE_THRESHOLDS.polishP95Ms
  );
}

function readJsonLines(filePath) {
  return fs
    .readFileSync(filePath, "utf8")
    .split(/\r?\n/u)
    .map((line) => line.trim())
    .filter(Boolean)
    .map((line, index) => {
      try {
        return JSON.parse(line);
      } catch (error) {
        throw new Error(
          `${filePath}:${index + 1} JSON 无效：${error instanceof Error ? error.message : String(error)}`,
          { cause: error },
        );
      }
    });
}

function parseArguments(argumentsList) {
  const result = {
    manifest: "evals/asr/private/manifest.jsonl",
    predictions: "evals/asr/private/predictions.jsonl",
    minSamples: 80,
    output: null,
    baseline: null,
  };
  for (let index = 0; index < argumentsList.length; index += 1) {
    const argument = argumentsList[index];
    const value = argumentsList[index + 1];
    if (argument === "--manifest") result.manifest = value;
    else if (argument === "--predictions") result.predictions = value;
    else if (argument === "--min-samples") result.minSamples = Number(value);
    else if (argument === "--output") result.output = value;
    else if (argument === "--baseline") result.baseline = value;
    else throw new Error(`未知参数：${argument}`);
    index += 1;
  }
  return result;
}

function formatPercent(value) {
  return value === null ? "N/A" : `${(value * 100).toFixed(2)}%`;
}

function runCli() {
  const options = parseArguments(process.argv.slice(2));
  const manifestPath = path.resolve(options.manifest);
  const predictionPath = path.resolve(options.predictions);
  const manifest = readJsonLines(manifestPath);
  validateDataset(manifest, {
    minSamples: options.minSamples,
    manifestDir: path.dirname(manifestPath),
  });
  const report = evaluateBenchmark(manifest, readJsonLines(predictionPath));
  if (options.baseline) {
    const baseline = JSON.parse(
      fs.readFileSync(path.resolve(options.baseline), "utf8"),
    );
    report.model_replacement_eligible = modelReplacementEligible(
      report,
      baseline,
    );
  }
  if (options.output) {
    fs.mkdirSync(path.dirname(path.resolve(options.output)), {
      recursive: true,
    });
    fs.writeFileSync(
      path.resolve(options.output),
      `${JSON.stringify(report, null, 2)}\n`,
    );
  }

  console.log(`样本：${report.metrics.samples}`);
  console.log(`去标点 CER：${formatPercent(report.metrics.cer)}`);
  console.log(
    `热词精确召回率：${formatPercent(report.metrics.hotword_recall)}`,
  );
  console.log(`标点 F1：${formatPercent(report.metrics.punctuation_f1)}`);
  console.log(
    `延迟 P50/P95（原文）：${report.metrics.latency.verbatim.p50_ms ?? "N/A"}/${report.metrics.latency.verbatim.p95_ms ?? "N/A"} ms`,
  );
  console.log(
    `延迟 P50/P95（润色）：${report.metrics.latency.polish.p50_ms ?? "N/A"}/${report.metrics.latency.polish.p95_ms ?? "N/A"} ms`,
  );
  console.log(`第一阶段验收：${report.accepted ? "通过" : "未通过"}`);
  if ("model_replacement_eligible" in report) {
    console.log(
      `默认模型替换门槛：${report.model_replacement_eligible ? "满足" : "不满足"}`,
    );
  }
  if (!report.accepted) process.exitCode = 2;
}

if (
  process.argv[1] &&
  fileURLToPath(import.meta.url) === path.resolve(process.argv[1])
) {
  try {
    runCli();
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
