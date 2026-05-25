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
  longpress_shortcut: string;
  toggle_shortcut: string;
  output_mode: string;
  auto_save_history: boolean;
  mute_system_audio: boolean;
  selected_microphone: string;
  recording_mode: string;
};

export type AudioDevice = {
  name: string;
  is_default: boolean;
};
