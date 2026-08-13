import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { AppConfig, Persona } from "@/types";

const mocks = vi.hoisted(() => ({
  commands: {
    listPersonas: vi.fn(),
    createPersona: vi.fn(),
    updatePersona: vi.fn(),
    deletePersona: vi.fn(),
    setDefaultPersona: vi.fn(),
    readAppConfig: vi.fn(),
  },
}));

vi.mock("@/generated/tauri-bindings", () => ({
  commands: mocks.commands,
}));

import { usePersonaController } from "./usePersonaController";

const verbatimPersona: Persona = {
  id: "verbatim",
  name: "原文听写",
  description: "保留原文",
  icon: "📝",
  is_default: true,
  processing_mode: "verbatim",
  created_at: "2026-01-01T00:00:00Z",
  updated_at: "2026-01-01T00:00:00Z",
};

const refreshedConfig = {
  default_persona_id: "verbatim",
} as AppConfig;

describe("usePersonaController", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.commands.listPersonas.mockResolvedValue([verbatimPersona]);
  });

  it("uses the authoritative default-persona update without rereading config", async () => {
    let resolveSetDefault:
      ((value: { personas: Persona[]; config: AppConfig }) => void) | undefined;
    mocks.commands.setDefaultPersona.mockReturnValue(
      new Promise<{ personas: Persona[]; config: AppConfig }>((resolve) => {
        resolveSetDefault = resolve;
      }),
    );
    mocks.commands.readAppConfig.mockResolvedValue(refreshedConfig);
    const onConfigLoaded = vi.fn();
    const { result } = renderHook(() => usePersonaController(onConfigLoaded));

    await waitFor(() =>
      expect(mocks.commands.listPersonas).toHaveBeenCalledTimes(1),
    );

    let pending: Promise<void>;
    act(() => {
      pending = result.current.setDefault("verbatim");
    });

    expect(mocks.commands.readAppConfig).not.toHaveBeenCalled();

    await act(async () => {
      resolveSetDefault?.({
        personas: [verbatimPersona],
        config: refreshedConfig,
      });
      await pending!;
    });

    expect(mocks.commands.readAppConfig).not.toHaveBeenCalled();
    expect(onConfigLoaded).toHaveBeenCalledWith(refreshedConfig);
  });

  it("keeps the current UI configuration when setting the default persona fails", async () => {
    const generalPersona = {
      ...verbatimPersona,
      id: "general",
      is_default: true,
    };
    mocks.commands.listPersonas.mockResolvedValue([
      generalPersona,
      verbatimPersona,
    ]);
    mocks.commands.setDefaultPersona.mockRejectedValue(
      new Error("默认人格配置保存失败，已回滚"),
    );
    const onConfigLoaded = vi.fn();
    const { result } = renderHook(() => usePersonaController(onConfigLoaded));
    await waitFor(() => expect(result.current.selectedId).toBe("general"));

    await act(() => result.current.setDefault("verbatim"));

    expect(result.current.selectedId).toBe("general");
    expect(onConfigLoaded).not.toHaveBeenCalled();
    expect(mocks.commands.readAppConfig).not.toHaveBeenCalled();
  });
});
