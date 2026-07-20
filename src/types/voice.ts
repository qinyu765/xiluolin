import type {
  CaptureSessionStart,
  OutputResult,
  RecordingResult,
  VoiceInputResult,
} from "@/generated/tauri-bindings";

export type RecordingStartResult = CaptureSessionStart;
export type DeliveryResult = OutputResult;
export type { RecordingResult, VoiceInputResult };
