import { describe, expect, it } from "vitest";

import type { AppConfig, AudioDevice } from "@/types";

import {
  collectSchemaConfigKeys,
  getVisibleFields,
  resolveFieldOptions,
  settingsSchema,
} from "./settings-schema";

const config: AppConfig = {
  default_persona_id: "general",
  asr_provider: "zhipu",
  asr_api_key: "",
  asr_base_url: "https://open.bigmodel.cn/api/paas/v4",
  asr_model: "glm-asr-2512",
  openai_asr_model: "whisper-1",
  openai_api_key: "",
  openai_base_url: "https://api.openai.com/v1",
  openai_model: "gpt-4o-mini",
  text_provider: "zhipu",
  zhipu_api_key: "",
  zhipu_base_url: "https://open.bigmodel.cn/api/paas/v4",
  zhipu_model: "glm-4.7-flash",
  longpress_shortcut: "CommandOrControl+Shift+R",
  toggle_shortcut: "Alt+Space",
  fn_hold_enabled: false,
  auto_save_history: true,
  mute_system_audio: false,
  selected_microphone: "",
  retain_recordings: false,
  local_asr_model: "whisper-base-q5_1",
  allow_cloud_fallback: false,
  fallback_asr_provider: "zhipu",
};

describe("settingsSchema", () => {
  it("覆盖设置页消费的全部配置字段", () => {
    expect(collectSchemaConfigKeys(settingsSchema).sort()).toEqual(
      [
        "allow_cloud_fallback",
        "asr_api_key",
        "asr_base_url",
        "asr_model",
        "asr_provider",
        "auto_save_history",
        "fallback_asr_provider",
        "fn_hold_enabled",
        "local_asr_model",
        "longpress_shortcut",
        "mute_system_audio",
        "openai_api_key",
        "openai_asr_model",
        "openai_base_url",
        "openai_model",
        "retain_recordings",
        "selected_microphone",
        "text_provider",
        "toggle_shortcut",
        "zhipu_api_key",
        "zhipu_base_url",
        "zhipu_model",
      ].sort(),
    );
  });

  it("只显示当前 Provider 对应的模型字段", () => {
    const asrSection = settingsSchema.models.find(
      (section) => section.id === "asr",
    );
    const zhipuFields = getVisibleFields(asrSection, config).map(
      (field) => field.id,
    );
    const openaiFields = getVisibleFields(asrSection, {
      ...config,
      asr_provider: "openai",
    }).map((field) => field.id);
    const localFields = getVisibleFields(asrSection, {
      ...config,
      asr_provider: "local",
    }).map((field) => field.id);

    expect(zhipuFields).toContain("asr-api-key");
    expect(zhipuFields).not.toContain("openai-asr-api-key");
    expect(openaiFields).toContain("openai-asr-api-key");
    expect(localFields).toContain("local-asr-settings");
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
      retainRecordings?.disabled?.({
        ...config,
        auto_save_history: false,
      }),
    ).toBe(true);
  });
});
