import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { commands } from "@/generated/tauri-bindings";
import type { InputReadiness, ReadinessCheck } from "@/types";

import { InputReadinessCard } from "./InputReadinessCard";

vi.mock("@/generated/tauri-bindings", () => ({
  commands: {
    readInputReadiness: vi.fn(),
    requestMacosPermission: vi.fn(),
    openMacosPrivacySettings: vi.fn(),
  },
}));

const readyCheck = (detail: string): ReadinessCheck => ({
  ready: true,
  blocking: false,
  detail,
  actions: [],
});

const readyState: InputReadiness = {
  platform: "macos",
  macos_permissions: null,
  microphone: readyCheck("麦克风已授权"),
  asr: readyCheck("ASR 可用"),
  text_processing: readyCheck("文本模型可用"),
  hotkey: readyCheck("快捷键已注册"),
  auto_paste: readyCheck("辅助功能已授权"),
  models_ready: true,
  can_process: true,
  can_dictate: true,
};

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("InputReadinessCard", () => {
  it("全部就绪时只显示紧凑摘要和五个状态胶囊", async () => {
    vi.mocked(commands.readInputReadiness).mockResolvedValue(readyState);
    render(<InputReadinessCard />);

    expect(await screen.findByText("语音输入已就绪")).toBeInTheDocument();
    for (const label of [
      "麦克风",
      "语音识别",
      "文本处理",
      "全局快捷键",
      "自动粘贴",
    ]) {
      expect(screen.getByText(label)).toBeInTheDocument();
    }
    expect(screen.queryByText("麦克风已授权")).not.toBeInTheDocument();
    expect(screen.queryByText("ASR 可用")).not.toBeInTheDocument();
  });

  it("存在异常时仅展开异常详情和处理动作", async () => {
    vi.mocked(commands.readInputReadiness).mockResolvedValue({
      ...readyState,
      microphone: {
        ready: false,
        blocking: true,
        detail: "尚未授予麦克风权限",
        actions: ["request_microphone"],
      },
      can_process: false,
      can_dictate: false,
    });
    render(<InputReadinessCard />);

    expect(await screen.findByText("尚未授予麦克风权限")).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "请求麦克风权限" }),
    ).toBeInTheDocument();
    expect(screen.queryByText("ASR 可用")).not.toBeInTheDocument();
    expect(screen.queryByText("快捷键已注册")).not.toBeInTheDocument();
  });
});
