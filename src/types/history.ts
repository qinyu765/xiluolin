export type HistoryRecord = {
  id: string;
  raw_text: string;
  final_text: string;
  persona_id: string;
  persona_name: string;
  duration_ms: number;
  output_chars: number;
  output_mode: string;
  source: string;
  asr_provider: string;
  asr_model: string;
  text_provider: string;
  text_model: string;
  used_fallback: boolean;
  delivery_method: string;
  audio_path: string | null;
  created_at: string;
};

export type HistoryStatistics = {
  total_count: number;
  total_duration_ms: number;
  total_output_chars: number;
  estimated_saved_ms: number;
  top_persona_name: string | null;
  top_persona_count: number;
};

export type GroupedHistory = {
  todayRecords: HistoryRecord[];
  yesterdayRecords: HistoryRecord[];
  olderRecords: Map<string, HistoryRecord[]>;
};
