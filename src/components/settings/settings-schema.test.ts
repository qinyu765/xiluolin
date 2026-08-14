import { describe, expect, it } from "vitest";

import type { AppConfig, AudioDevice } from "@/types";

import {
  collectSchemaConfigKeys,
  getVisibleFields,
  prepareSettingsConfig,
  resolveFieldOptions,
  settingsSchema,
  validateSettingsConfig,
} from "./settings-schema";

const config: AppConfig = {
  config_version: 2,
  default_persona_id: "general",
  asr: {
    primary: "zhipu",
    fallbacks: [],
    settings: {
      zhipu: {
        api_key: "",
        base_url: "https://open.bigmodel.cn/api/paas/v4",
        model: "glm-asr-2512",
        options: {},
      },
    },
  },
  text: {
    primary: "zhipu",
    fallbacks: [],
    settings: {
      zhipu: {
        api_key: "",
        base_url: "https://open.bigmodel.cn/api/paas/v4",
        model: "glm-4.7-flash",
        options: {},
      },
    },
  },
  longpress_shortcut: "CommandOrControl+Shift+R",
  toggle_shortcut: "Alt+Space",
  fn_hold_enabled: false,
  auto_save_history: true,
  mute_system_audio: false,
  selected_microphone: "",
  retain_recordings: true,
  realtime_preview_enabled: false,
};

describe("settingsSchema", () => {
  it("覆盖通用设置消费的配置字段", () => {
    expect(collectSchemaConfigKeys(settingsSchema).sort()).toEqual(
      [
        "asr",
        "auto_save_history",
        "fn_hold_enabled",
        "longpress_shortcut",
        "mute_system_audio",
        "retain_recordings",
        "realtime_preview_enabled",
        "selected_microphone",
        "text",
        "toggle_shortcut",
      ].sort(),
    );
  });

  it("从运行时设备生成麦克风选项并声明录音依赖", () => {
    const devices: AudioDevice[] = [
      { name: "Studio Mic", is_default: true },
      { name: "USB Mic", is_default: false },
    ];
    const generalSection = settingsSchema.general.find(
      (section) => section.id === "general",
    );
    const microphone = getVisibleFields(generalSection, config).find(
      (field) => field.id === "selected-microphone",
    );
    const retainRecordings = getVisibleFields(generalSection, config).find(
      (field) => field.id === "retain-recordings",
    );

    expect(resolveFieldOptions(microphone, { audioDevices: devices })).toEqual([
      { label: "使用默认麦克风", value: "" },
      { label: "Studio Mic（默认）", value: "Studio Mic" },
      { label: "USB Mic", value: "USB Mic" },
    ]);
    expect(
      retainRecordings?.disabled?.({ ...config, auto_save_history: false }),
    ).toBe(true);
  });

  it("保存前清理通用字段和嵌套 Provider 设置", () => {
    const prepared = prepareSettingsConfig({
      ...config,
      longpress_shortcut: "  Alt+Space  ",
      asr: {
        ...config.asr,
        settings: {
          zhipu: {
            ...config.asr.settings?.zhipu,
            api_key: "  secret  ",
            base_url: "  https://example.com  ",
            model: "  glm-asr  ",
          },
        },
      },
    });

    expect(prepared.longpress_shortcut).toBe("Alt+Space");
    expect(prepared.asr.settings?.zhipu?.api_key).toBe("secret");
    expect(prepared.asr.settings?.zhipu?.base_url).toBe("https://example.com");
    expect(validateSettingsConfig(prepared)).toBeNull();
  });

  it("拒绝重复 route、空 primary 和不完整的云 Provider", () => {
    expect(
      validateSettingsConfig({
        ...config,
        asr: { ...config.asr, primary: "", fallbacks: [] },
      }),
    ).toContain("尚未选择");
    expect(
      validateSettingsConfig({
        ...config,
        asr: { ...config.asr, fallbacks: ["zhipu"] },
      }),
    ).toContain("重复");
    expect(
      validateSettingsConfig({
        ...config,
        asr: {
          ...config.asr,
          settings: { zhipu: { base_url: "", model: "glm-asr" } },
        },
      }),
    ).toContain("Base URL");
  });
});
