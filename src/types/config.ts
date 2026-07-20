import type {
  AppConfig as GeneratedAppConfig,
  AudioDevice,
  InputReadiness,
  LocalAsrDownloadProgress,
  LocalAsrModelInfo,
  PermissionStatus,
  ReadinessAction,
  ReadinessCheck,
  RecordingStorageInfo,
} from "@/generated/tauri-bindings";

export type AppConfig = Required<GeneratedAppConfig>;
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
