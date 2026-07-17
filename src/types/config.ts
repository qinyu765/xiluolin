export type AppConfig = {
  default_persona_id: string;
  asr_provider: string;
  asr_api_key: string;
  asr_base_url: string;
  asr_model: string;
  openai_asr_model: string;
  openai_api_key: string;
  openai_base_url: string;
  openai_model: string;
  text_provider: string;
  zhipu_api_key: string;
  zhipu_base_url: string;
  zhipu_model: string;
  longpress_shortcut: string;
  toggle_shortcut: string;
  auto_save_history: boolean;
  mute_system_audio: boolean;
  selected_microphone: string;
  retain_recordings: boolean;
  local_asr_model: string;
  allow_cloud_fallback: boolean;
  fallback_asr_provider: string;
};

export type AudioDevice = {
  name: string;
  is_default: boolean;
};

export type ReadinessCheck = {
  ready: boolean;
  blocking: boolean;
  detail: string;
};

export type InputReadiness = {
  microphone: ReadinessCheck;
  asr: ReadinessCheck;
  text_processing: ReadinessCheck;
  hotkey: ReadinessCheck;
  auto_paste: ReadinessCheck;
  models_ready: boolean;
  can_process: boolean;
  can_dictate: boolean;
};

export type RecordingStorageInfo = {
  file_count: number;
  total_bytes: number;
  directory: string;
};

export type LocalAsrModelInfo = {
  name: string;
  path: string;
  exists: boolean;
  size_bytes: number;
};

export type LocalAsrDownloadProgress = {
  downloaded_bytes: number;
  total_bytes: number | null;
  percent: number | null;
};
