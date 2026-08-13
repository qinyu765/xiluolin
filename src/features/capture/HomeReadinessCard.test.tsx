import { render, screen } from "@testing-library/react";
import { expect, test, vi } from "vitest";

import { HomeReadinessCard } from "./HomeReadinessCard";
import type { AppConfig } from "@/types";

vi.mock("@/generated/tauri-bindings", () => ({
  commands: {
    readInputReadiness: vi.fn().mockResolvedValue({
      microphone: { ready: true },
      asr: { ready: true },
      hotkey: { ready: true },
      can_dictate: true,
    }),
    realtimeAsrModelInfo: vi.fn().mockResolvedValue({
      state: "not_downloaded",
      enabled: false,
    }),
    listProviderCatalog: vi.fn().mockResolvedValue({
      asr: [
        {
          id: "qwen3-asr",
          name: "Qwen3-ASR",
        },
      ],
      text: [],
    }),
  },
}));

test("使用 Provider catalog 展示当前最终识别路由", async () => {
  const appConfig = {
    config_version: 2,
    asr: {
      primary: "qwen3-asr",
      fallbacks: [],
      settings: {},
    },
    text: {
      primary: "zhipu",
      fallbacks: [],
      settings: {},
    },
    default_persona_id: "default",
    longpress_shortcut: "",
    toggle_shortcut: "CommandOrControl+Shift+Space",
    fn_hold_enabled: false,
    auto_save_history: true,
    mute_system_audio: false,
    selected_microphone: "",
    retain_recordings: false,
    realtime_preview_enabled: false,
  } satisfies AppConfig;

  render(<HomeReadinessCard appConfig={appConfig} persona={undefined} />);

  expect(await screen.findByText("Qwen3-ASR")).toBeInTheDocument();
});
