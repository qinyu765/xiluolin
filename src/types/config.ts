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
  ProviderRoutingConfig,
  ProviderSettings,
} from "@/generated/tauri-bindings";

/** Runtime config is normalized after IPC, so all v2 fields are present. */
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
  ProviderRoutingConfig,
  ProviderSettings,
};
