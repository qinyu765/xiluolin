import type {
  AppConfig_Serialize,
  AudioDevice,
  InputReadiness,
  LocalAsrDownloadProgress,
  LocalAsrModelInfo,
  PermissionStatus,
  ReadinessAction,
  ReadinessCheck,
  RecordingStorageInfo,
} from "@/generated/tauri-bindings";

export type AppConfig = AppConfig_Serialize;
export type {
  AudioDevice,
  InputReadiness,
  LocalAsrDownloadProgress,
  LocalAsrModelInfo,
  PermissionStatus,
  ReadinessAction,
  ReadinessCheck,
  RecordingStorageInfo,
};
