import type { HistoryRecord } from "./history";

export type VoiceInputResult = {
  raw_text: string;
  final_text: string;
  used_text_fallback: boolean;
  history_record: HistoryRecord | null;
};

export type RecordingStartResult = {
  session_id: string;
};

export type RecordingResult = RecordingStartResult & {
  file_path: string;
  duration_ms: number;
};

export type DeliveryResult = {
  method: "paste" | "clipboard" | "manual";
  success: boolean;
  message: string;
  target_restored: boolean;
  clipboard_restored: boolean;
  used_fallback: boolean;
};
