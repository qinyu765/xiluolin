import type {
  AppConfig,
  AudioDevice,
  ProviderRoutingConfig,
  ProviderSettings,
} from "@/types";

export type SettingsSaveMode = "immediate" | "debounced";
export type SettingsSchemaContext = { audioDevices: AudioDevice[] };

type ConfigStringKey = Extract<keyof AppConfig, string>;
type StringConfigKey = {
  [Key in ConfigStringKey]: AppConfig[Key] extends string ? Key : never;
}[ConfigStringKey];
type BooleanConfigKey = {
  [Key in ConfigStringKey]: AppConfig[Key] extends boolean ? Key : never;
}[ConfigStringKey];

type FieldBase = {
  id: string;
  label?: string;
  description?: string;
  group?: "shortcuts" | "recording" | "history";
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
    "longpress-shortcut" | "fn-hold" | "toggle-shortcut" | "provider-catalog";
  configKeys: readonly (keyof AppConfig)[];
};

export type SettingsFieldSchema =
  | TextSettingsField
  | SelectSettingsField
  | SwitchSettingsField
  | CustomSettingsField;

export type SettingsFieldOption = { label: string; value: string };

export type SettingsSectionSchema = {
  type: "fields";
  id: string;
  title: string;
  description?: string;
  fields: readonly SettingsFieldSchema[];
};

export type SettingsSchema = {
  general: readonly SettingsSectionSchema[];
  models: readonly SettingsSectionSchema[];
};

const trim = (value: string) => value.trim();

export const settingsSchema: SettingsSchema = {
  general: [
    {
      type: "fields",
      id: "general",
      title: "通用",
      fields: [
        {
          id: "longpress-shortcut",
          control: "slot",
          slot: "longpress-shortcut",
          configKeys: ["longpress_shortcut"],
          saveMode: "immediate",
          group: "shortcuts",
          span: "full",
        },
        {
          id: "fn-hold",
          control: "slot",
          slot: "fn-hold",
          configKeys: ["fn_hold_enabled"],
          saveMode: "immediate",
          group: "shortcuts",
          span: "full",
        },
        {
          id: "toggle-shortcut",
          control: "slot",
          slot: "toggle-shortcut",
          configKeys: ["toggle_shortcut"],
          saveMode: "immediate",
          group: "shortcuts",
          span: "full",
        },
        {
          id: "selected-microphone",
          control: "select",
          key: "selected_microphone",
          label: "麦克风设备",
          description: "留空使用系统默认麦克风",
          options: ({ audioDevices }) => [
            { label: "使用默认麦克风", value: "" },
            ...audioDevices.map((device) => ({
              label: `${device.name}${device.is_default ? "（默认）" : ""}`,
              value: device.name,
            })),
          ],
          saveMode: "immediate",
          group: "recording",
          span: "full",
        },
        {
          id: "mute-system-audio",
          control: "switch",
          key: "mute_system_audio",
          label: "录音时静音其他应用",
          description: "输入完成后自动恢复",
          saveMode: "immediate",
          group: "recording",
          span: "full",
        },
        {
          id: "auto-save-history",
          control: "switch",
          key: "auto_save_history",
          label: "自动保存历史",
          saveMode: "immediate",
          group: "history",
          span: "full",
          toPatch: (value, config) => ({
            auto_save_history: value,
            retain_recordings: value ? config.retain_recordings : false,
          }),
        },
      ],
    },
  ],
  models: [
    {
      type: "fields",
      id: "models",
      title: "模型配置",
      fields: [
        {
          id: "provider-catalog",
          control: "slot",
          slot: "provider-catalog",
          configKeys: ["asr", "text", "realtime_preview_enabled"],
          saveMode: "immediate",
          span: "full",
        },
      ],
    },
  ],
};

export function collectSchemaConfigKeys(schema: SettingsSchema) {
  const keys = new Set<string>();
  for (const section of [...schema.general, ...schema.models]) {
    for (const field of section.fields) {
      if (field.control === "slot") {
        field.configKeys.forEach((key) => keys.add(String(key)));
      } else {
        keys.add(field.key);
      }
    }
  }
  return [...keys];
}

export function getVisibleFields(
  section: SettingsSectionSchema | undefined,
  config: AppConfig,
) {
  return (section?.fields ?? []).filter(
    (field) => !field.visible || field.visible(config),
  );
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

export function prepareSettingsConfig(config: AppConfig): AppConfig {
  return {
    ...config,
    longpress_shortcut: trim(config.longpress_shortcut),
    toggle_shortcut: trim(config.toggle_shortcut),
    selected_microphone: trim(config.selected_microphone),
    asr: normalizeRoute(config.asr),
    text: normalizeRoute(config.text),
  };
}

export function validateSettingsConfig(config: AppConfig): string | null {
  for (const [label, route] of [
    ["ASR", config.asr],
    ["文本", config.text],
  ] as const) {
    const primary = route.primary?.trim() ?? "";
    const fallbacks = route.fallbacks ?? [];
    const routeIds = [primary, ...fallbacks].filter(Boolean);
    if (!primary) return `${label} Provider 尚未选择。`;
    if (routeIds.length > 3) return `${label} Provider 调用链最多包含 3 项。`;
    if (new Set(routeIds).size !== routeIds.length) {
      return `${label} Provider 调用链不能包含重复项。`;
    }
    for (const provider of routeIds) {
      if (provider === "local") continue;
      const settings = route.settings?.[provider];
      if (!settings?.base_url?.trim()) {
        return `${provider} 的 Base URL 待补全。`;
      }
      if (!settings.model?.trim()) {
        return `${provider} 的模型名待补全。`;
      }
    }
  }
  return null;
}

function normalizeRoute(route: ProviderRoutingConfig): ProviderRoutingConfig {
  return {
    primary: route.primary?.trim() ?? "",
    fallbacks: [...(route.fallbacks ?? [])].map(trim),
    settings: Object.fromEntries(
      Object.entries(route.settings ?? {}).map(([provider, settings]) => [
        provider,
        normalizeProviderSettings(settings),
      ]),
    ),
  };
}

function normalizeProviderSettings(
  settings: ProviderSettings,
): ProviderSettings {
  return {
    ...settings,
    api_key: settings.api_key?.trim() ?? "",
    base_url: settings.base_url?.trim() ?? "",
    model: settings.model?.trim() ?? "",
    options: settings.options ?? {},
  };
}
