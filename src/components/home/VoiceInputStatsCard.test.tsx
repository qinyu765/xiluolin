import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import type { HistoryRecord, HistoryStatistics } from "@/types";

import { VoiceInputStatsCard } from "./VoiceInputStatsCard";

const stats: HistoryStatistics = {
  total_count: 3,
  total_duration_ms: 120_000,
  total_output_chars: 128,
  estimated_saved_ms: 60_000,
  top_persona_name: "通用人格",
  top_persona_count: 2,
};

const record: HistoryRecord = {
  id: "history-1",
  raw_text: "原始识别文本",
  final_text: "这是一条历史结果",
  persona_id: "general",
  persona_name: "通用人格",
  duration_ms: 1_000,
  output_chars: 8,
  output_mode: "clipboard",
  source: "recording",
  asr_provider: "local",
  asr_model: "base",
  text_provider: "local",
  text_model: "default",
  text_processing_mode: "disabled",
  used_asr_fallback: false,
  used_fallback: false,
  delivery_method: "clipboard",
  audio_path: null,
  created_at: "2026-08-14T10:00:00Z",
};

describe("VoiceInputStatsCard", () => {
  it("保留首页历史区域但移除‘最近历史’标题", () => {
    render(
      <VoiceInputStatsCard
        historyStats={stats}
        historyRecords={[record]}
        onCopyHistoryText={() => undefined}
        onDeleteHistoryRecord={() => undefined}
        onPlayHistoryRecording={() => undefined}
        onReprocessHistoryAudio={() => undefined}
        onRefineHistoryText={() => undefined}
        formatDuration={(milliseconds) => `${milliseconds}ms`}
        formatCreatedAt={() => "今天 10:00"}
        groupHistoryByDate={(records) => ({
          todayRecords: records,
          yesterdayRecords: [],
          olderRecords: new Map(),
        })}
      />,
    );

    expect(screen.getByText("语音协作次数")).toBeInTheDocument();
    expect(screen.getByText(record.final_text)).toBeInTheDocument();
    expect(screen.queryByText("最近历史")).not.toBeInTheDocument();
  });
});
