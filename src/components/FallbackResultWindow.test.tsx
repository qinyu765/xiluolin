import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  commands: {
    readFallbackResult: vi.fn(),
    copyFallbackResult: vi.fn(),
    dismissFallbackResult: vi.fn(),
  },
}));

vi.mock("@/generated/tauri-bindings", () => ({
  commands: mocks.commands,
}));

vi.mock("sonner", () => ({
  Toaster: () => null,
  toast: {
    error: vi.fn(),
    success: vi.fn(),
  },
}));

import { FallbackResultWindow } from "./FallbackResultWindow";

afterEach(cleanup);

describe("FallbackResultWindow", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.commands.readFallbackResult.mockResolvedValue({
      text: "请用 Node.js 写一个 HTTP 服务",
      reason: "自动粘贴未完成：目标应用已退出",
      copied: false,
    });
    mocks.commands.copyFallbackResult.mockResolvedValue(null);
    mocks.commands.dismissFallbackResult.mockResolvedValue(null);
  });

  it("展示失败结果并支持再次复制", async () => {
    render(<FallbackResultWindow />);

    expect(
      await screen.findByDisplayValue("请用 Node.js 写一个 HTTP 服务"),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "复制结果" }));

    await waitFor(() =>
      expect(mocks.commands.copyFallbackResult).toHaveBeenCalledTimes(1),
    );
    expect(
      screen.getByRole("button", { name: "再次复制" }),
    ).toBeInTheDocument();
  });

  it("按 Escape 会清除结果并关闭窗口", async () => {
    render(<FallbackResultWindow />);
    await screen.findByDisplayValue("请用 Node.js 写一个 HTTP 服务");

    fireEvent.keyDown(window, { key: "Escape" });

    await waitFor(() =>
      expect(mocks.commands.dismissFallbackResult).toHaveBeenCalledTimes(1),
    );
    expect(screen.getByText("没有可用的失败结果")).toBeInTheDocument();
  });
});
