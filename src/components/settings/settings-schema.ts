import type { AppConfig, AudioDevice } from "@/types";

export type SettingsSaveMode = "immediate" | "debounced";
export type SettingsSchemaContext = { audioDevices: AudioDevice[] };

type StringConfigKey = {
  [Key in keyof AppConfig]: AppConfig[Key] extends string ? Key : never;
}[keyof AppConfig];

type BooleanConfigKey = {
  [Key in keyof AppConfig]: AppConfig[Key] extends boolean ? Key : never;
}[keyof AppConfig];

type FieldBase = {
  id: string;
  label?: string;
  description?: string;
  span?: "full";
  visible?: (config: AppConfig) => boolean;
  disabled?: (config: AppConfig) => boolean;
  saveMode: SettingsSaveMode;
};

export type TextSettingsField = FieldBase & {
  control: "text" | "password";
  key: StringConfigKey;
  placeholder?: string;
  normalize?: (value: string) => string;
  validate?: (value: string, config: AppConfig) => string | null;
};

export type SelectSettingsField = FieldBase & {
  control: "select";
  key: StringConfigKey;
  options:
    | readonly SettingsFieldOption[]
    | ((context: SettingsSchemaContext) => readonly SettingsFieldOption[]);
};

export type SwitchSettingsField = FieldBase & {
  control: "switch";
  key: BooleanConfigKey;
  toPatch?: (value: boolean, config: AppConfig) => Partial<AppConfig>;
};

export type CustomSettingsField = FieldBase & {
  control: "slot";
  slot:
    "longpress-shortcut" | "fn-hold" | "toggle-shortcut" | "local-asr-settings";
  configKeys: readonly (keyof AppConfig)[];
};

export type SettingsFieldSchema =
  | TextSettingsField
  | SelectSettingsField
  | SwitchSettingsField
  | CustomSettingsField;

export type SettingsFieldOption = { label: string; value: string };

export type SettingsSectionSchema =
  | {
      type: "fields";
      id: string;
      title: string;
      description: string;
      fields: readonly SettingsFieldSchema[];
    }
  | {
      type: "slot";
      id: string;
      slot: "recording-storage";
      configKeys: readonly (keyof AppConfig)[];
    };

export type SettingsSchema = {
  general: readonly SettingsSectionSchema[];
  models: readonly SettingsSectionSchema[];
};

const required = (label: string) => (value: string) =>
  value.trim() ? null : `${label}不能为空。`;

const trim = (value: string) => value.trim();

export const settingsSchema = {
  general: [
    {
      type: "fields",
      id: "general",
      title: "通用设置",
      description: "配置快捷键、录音模式、输出方式和历史记录保存选项",
      fields: [
        {
          id: "longpress-shortcut",
          control: "slot",
          slot: "longpress-shortcut",
          configKeys: ["longpress_shortcut"],
          saveMode: "immediate",
          span: "full",
        },
        {
          id: "fn-hold",
          control: "slot",
          slot: "fn-hold",
          configKeys: ["fn_hold_enabled"],
          saveMode: "immediate",
          span: "full",
        },
        {
          id: "toggle-shortcut",
          control: "slot",
          slot: "toggle-shortcut",
          configKeys: ["toggle_shortcut"],
          saveMode: "immediate",
          span: "full",
        },
        {
          id: "selected-microphone",
          control: "select",
          key: "selected_microphone",
          label: "麦克风设备",
          description: "选择用于录音的麦克风设备。留空则使用系统默认麦克风。",
          options: ({ audioDevices }) => [
            { label: "使用默认麦克风", value: "" },
            ...audioDevices.map((device) => ({
              label: `${device.name}${device.is_default ? "（默认）" : ""}`,
              value: device.name,
            })),
          ],
          saveMode: "immediate",
          span: "full",
        },
        {
          id: "mute-system-audio",
          control: "switch",
          key: "mute_system_audio",
          label: "录音时静音其他应用",
          description:
            "开启后，语音输入时会暂停系统音频播放，输入完成后自动恢复",
          saveMode: "immediate",
          span: "full",
        },
        {
          id: "auto-save-history",
          control: "switch",
          key: "auto_save_history",
          label: "自动保存历史",
          description: "每次语音输入完成后自动保存到历史记录",
          saveMode: "immediate",
          span: "full",
          toPatch: (value, config) => ({
            auto_save_history: value,
            retain_recordings: value ? config.retain_recordings : false,
          }),
        },
        {
          id: "retain-recordings",
          control: "switch",
          key: "retain_recordings",
          label: "保留原始录音",
          description: "默认关闭。仅在自动保存历史成功时保留应用录制的 WAV",
          disabled: (currentConfig) => !currentConfig.auto_save_history,
          saveMode: "immediate",
          span: "full",
        },
      ],
    },
    {
      type: "slot",
      id: "recording-storage",
      slot: "recording-storage",
      configKeys: [],
    },
  ],
  models: [
    {
      type: "fields",
      id: "asr",
      title: "语音识别服务",
      description: "配置 ASR 服务，用于把短音频转换为原始识别文本",
      fields: [
        {
          id: "asr-provider",
          control: "select",
          key: "asr_provider",
          label: "服务商",
          description: "模型名可自行配置；切换服务商不会覆盖已有模型名",
          options: [
            { label: "智谱 AI", value: "zhipu" },
            { label: "OpenAI 兼容", value: "openai" },
            { label: "本地（离线）", value: "local" },
          ],
          saveMode: "immediate",
          span: "full",
        },
        {
          id: "asr-api-key",
          control: "password",
          key: "asr_api_key",
          label: "智谱 API Key",
          placeholder: "本地保存，不写入仓库",
          visible: (currentConfig) => currentConfig.asr_provider === "zhipu",
          normalize: trim,
          saveMode: "debounced",
          span: "full",
        },
        {
          id: "asr-base-url",
          control: "text",
          key: "asr_base_url",
          label: "Base URL",
          visible: (currentConfig) => currentConfig.asr_provider === "zhipu",
          normalize: trim,
          validate: required("Base URL"),
          saveMode: "debounced",
        },
        {
          id: "asr-model",
          control: "text",
          key: "asr_model",
          label: "模型",
          visible: (currentConfig) => currentConfig.asr_provider === "zhipu",
          normalize: trim,
          validate: required("模型名"),
          saveMode: "debounced",
        },
        {
          id: "openai-asr-api-key",
          control: "password",
          key: "openai_api_key",
          label: "OpenAI API Key",
          placeholder: "本地保存，不写入仓库",
          visible: (currentConfig) => currentConfig.asr_provider === "openai",
          normalize: trim,
          saveMode: "debounced",
          span: "full",
        },
        {
          id: "openai-asr-base-url",
          control: "text",
          key: "openai_base_url",
          label: "Base URL",
          visible: (currentConfig) => currentConfig.asr_provider === "openai",
          normalize: trim,
          validate: required("Base URL"),
          saveMode: "debounced",
        },
        {
          id: "openai-asr-model",
          control: "text",
          key: "openai_asr_model",
          label: "模型",
          visible: (currentConfig) => currentConfig.asr_provider === "openai",
          normalize: trim,
          validate: required("模型名"),
          saveMode: "debounced",
        },
        {
          id: "local-asr-settings",
          control: "slot",
          slot: "local-asr-settings",
          configKeys: [
            "local_asr_model",
            "allow_cloud_fallback",
            "fallback_asr_provider",
          ],
          visible: (currentConfig) => currentConfig.asr_provider === "local",
          saveMode: "immediate",
          span: "full",
        },
      ],
    },
    {
      type: "fields",
      id: "text-processing",
      title: "文本整理服务",
      description: "配置文本处理 API，用于把原始识别文本整理成可直接使用的结果",
      fields: [
        {
          id: "text-provider",
          control: "select",
          key: "text_provider",
          label: "服务商",
          description: "模型名可自行配置；切换服务商不会覆盖已有模型名",
          options: [
            { label: "智谱 AI", value: "zhipu" },
            { label: "OpenAI 兼容", value: "openai" },
          ],
          saveMode: "immediate",
          span: "full",
        },
        {
          id: "zhipu-api-key",
          control: "password",
          key: "zhipu_api_key",
          label: "智谱 API Key",
          placeholder: "本地保存，不写入仓库",
          visible: (currentConfig) => currentConfig.text_provider === "zhipu",
          normalize: trim,
          saveMode: "debounced",
          span: "full",
        },
        {
          id: "zhipu-base-url",
          control: "text",
          key: "zhipu_base_url",
          label: "Base URL",
          visible: (currentConfig) => currentConfig.text_provider === "zhipu",
          normalize: trim,
          validate: required("Base URL"),
          saveMode: "debounced",
        },
        {
          id: "zhipu-model",
          control: "text",
          key: "zhipu_model",
          label: "模型",
          visible: (currentConfig) => currentConfig.text_provider === "zhipu",
          normalize: trim,
          validate: required("模型名"),
          saveMode: "debounced",
        },
        {
          id: "openai-api-key",
          control: "password",
          key: "openai_api_key",
          label: "OpenAI API Key",
          placeholder: "本地保存，不写入仓库",
          visible: (currentConfig) => currentConfig.text_provider === "openai",
          normalize: trim,
          saveMode: "debounced",
          span: "full",
        },
        {
          id: "openai-base-url",
          control: "text",
          key: "openai_base_url",
          label: "Base URL",
          visible: (currentConfig) => currentConfig.text_provider === "openai",
          normalize: trim,
          validate: required("Base URL"),
          saveMode: "debounced",
        },
        {
          id: "openai-model",
          control: "text",
          key: "openai_model",
          label: "模型",
          visible: (currentConfig) => currentConfig.text_provider === "openai",
          normalize: trim,
          validate: required("模型名"),
          saveMode: "debounced",
        },
      ],
    },
  ],
} satisfies SettingsSchema;

export function getVisibleFields(
  section: SettingsSectionSchema | undefined,
  config: AppConfig,
) {
  if (!section || section.type !== "fields") return [];
  return section.fields.filter((field) => field.visible?.(config) ?? true);
}

export function resolveFieldOptions(
  field: SettingsFieldSchema | undefined,
  context: SettingsSchemaContext,
) {
  if (!field || field.control !== "select") return [];
  return typeof field.options === "function"
    ? field.options(context)
    : field.options;
}

export function collectSchemaConfigKeys(schema: SettingsSchema) {
  const keys = new Set<keyof AppConfig>();
  for (const sections of [schema.general, schema.models]) {
    for (const section of sections) {
      if (section.type === "slot") {
        section.configKeys.forEach((key) => keys.add(key));
        continue;
      }
      for (const field of section.fields) {
        if (field.control === "slot") {
          field.configKeys.forEach((key) => keys.add(key));
        } else {
          keys.add(field.key);
        }
      }
    }
  }
  return [...keys];
}
