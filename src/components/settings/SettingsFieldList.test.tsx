import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import type { AppConfig } from "@/types";

import { SettingsFieldList } from "./SettingsFieldList";
import { settingsSchema } from "./settings-schema";

const config: AppConfig = {
  config_version: 2,
  default_persona_id: "general",
  asr: { primary: "local", fallbacks: [], settings: {} },
  text: { primary: "zhipu", fallbacks: [], settings: {} },
  longpress_shortcut: "",
  toggle_shortcut: "",
  fn_hold_enabled: false,
  auto_save_history: true,
  mute_system_audio: false,
  selected_microphone: "",
  retain_recordings: true,
  realtime_preview_enabled: false,
};

describe("SettingsFieldList", () => {
  it("渲染通用 Schema 并传递开关保存策略", () => {
    const onChange = vi.fn();
    const section = settingsSchema.general.find(
      (candidate) => candidate.id === "general",
    );

    render(
      <SettingsFieldList
        section={section}
        config={config}
        context={{ audioDevices: [] }}
        onChange={onChange}
        onBlur={() => undefined}
        renderSlot={() => null}
      />,
    );

    fireEvent.click(screen.getByRole("switch", { name: "自动保存历史" }));
    expect(onChange).toHaveBeenCalledWith(
      { auto_save_history: false, retain_recordings: false },
      "immediate",
    );
  });

  it("渲染动态麦克风选项", () => {
    const section = settingsSchema.general.find(
      (candidate) => candidate.id === "general",
    );
    render(
      <SettingsFieldList
        section={section}
        config={config}
        context={{ audioDevices: [{ name: "USB Mic", is_default: true }] }}
        onChange={vi.fn()}
        onBlur={() => undefined}
        renderSlot={() => null}
      />,
    );

    expect(screen.getByLabelText("麦克风设备")).toBeInTheDocument();
  });
});
