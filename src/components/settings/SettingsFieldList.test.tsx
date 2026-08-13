import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import type { AppConfig } from "@/types";

import { SettingsFieldList } from "./SettingsFieldList";
import { settingsSchema } from "./settings-schema";

const config = {
  asr_provider: "zhipu",
  text_provider: "zhipu",
  zhipu_api_key: "",
  auto_save_history: true,
  retain_recordings: true,
} as AppConfig;

describe("SettingsFieldList", () => {
  it("按 Schema 渲染当前 Provider 字段并传递保存策略", () => {
    const onChange = vi.fn();
    const section = settingsSchema.models.find(
      (candidate) => candidate.id === "text-processing",
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

    const apiKey = screen.getByLabelText("智谱 API Key");
    expect(apiKey).toBeInTheDocument();
    expect(screen.queryByLabelText("OpenAI API Key")).not.toBeInTheDocument();

    fireEvent.change(apiKey, { target: { value: "secret" } });
    expect(onChange).toHaveBeenCalledWith(
      { zhipu_api_key: "secret" },
      "debounced",
    );
  });

  it("使用 Schema 的依赖补丁更新开关", () => {
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
        renderSlot={(slot) => <span>{slot}</span>}
      />,
    );

    fireEvent.click(screen.getByRole("switch", { name: "自动保存历史" }));
    expect(onChange).toHaveBeenCalledWith(
      { auto_save_history: false, retain_recordings: false },
      "immediate",
    );
  });
});
