import type { HistoryRecord } from "./history";

export type VoiceInputResult = {
  raw_text: string;
  final_text: string;
  used_text_fallback: boolean;
  history_record: HistoryRecord | null;
};
