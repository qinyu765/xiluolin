import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { Hotword } from "@/types";

const mocks = vi.hoisted(() => ({
  commands: {
    listHotwords: vi.fn(),
    addHotwords: vi.fn(),
    createHotword: vi.fn(),
    updateHotword: vi.fn(),
    deleteHotword: vi.fn(),
  },
}));

vi.mock("@/generated/tauri-bindings", () => ({
  commands: mocks.commands,
}));

import { useHotwordController } from "./useHotwordController";

const existingHotword: Hotword = {
  id: "existing",
  text: "XiLuoLin",
  category: "产品名",
  enabled: false,
  created_at: "2026-01-01T00:00:00Z",
  updated_at: "2026-01-01T00:00:00Z",
};

describe("useHotwordController", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.commands.listHotwords.mockResolvedValue([existingHotword]);
    mocks.commands.addHotwords.mockResolvedValue([existingHotword]);
  });

  it("does not backfill saved hotwords and adds only the submitted draft", async () => {
    const { result } = renderHook(() => useHotwordController());

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    expect(result.current.bulkText).toBe("");

    act(() => {
      result.current.setBulkText(" Next.js \n\n Next.js\n XiLuoLin ");
    });

    expect(result.current.bulkCount).toBe(2);
    expect(mocks.commands.addHotwords).not.toHaveBeenCalled();

    await act(async () => {
      await result.current.saveBulk();
    });

    expect(mocks.commands.addHotwords).toHaveBeenCalledWith([
      "Next.js",
      "XiLuoLin",
    ]);
    expect(result.current.bulkText).toBe("");
  });

  it("keeps the add draft when batch saving fails", async () => {
    mocks.commands.addHotwords.mockRejectedValueOnce(new Error("写入失败"));
    const { result } = renderHook(() => useHotwordController());

    await waitFor(() => expect(result.current.isLoading).toBe(false));
    act(() => result.current.setBulkText("新词"));

    await act(async () => {
      await result.current.saveBulk();
    });

    expect(result.current.bulkText).toBe("新词");
  });
});
