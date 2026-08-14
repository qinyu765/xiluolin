import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type {
  RealtimeModelDownloadProgress,
  RealtimeModelInfo,
} from "@/generated/tauri-bindings";
import { RealtimePreviewModelCard } from "./RealtimePreviewModelCard";

const mocks = vi.hoisted(() => ({
  info: vi.fn(),
  download: vi.fn(),
  verify: vi.fn(),
  toggle: vi.fn(),
  remove: vi.fn(),
  progressListener: undefined as
    undefined | ((event: { payload: RealtimeModelDownloadProgress }) => void),
}));

vi.mock("@/generated/tauri-bindings", () => ({
  commands: {
    realtimeAsrModelInfo: mocks.info,
    downloadRealtimeAsrModel: mocks.download,
    verifyRealtimeAsrModel: mocks.verify,
    setRealtimePreviewEnabled: mocks.toggle,
    deleteRealtimeAsrModel: mocks.remove,
  },
  events: {
    realtimeAsrDownloadProgress: {
      listen: vi.fn(async (listener) => {
        mocks.progressListener = listener;
        return vi.fn();
      }),
    },
  },
}));

const model = (patch: Partial<RealtimeModelInfo> = {}): RealtimeModelInfo => ({
  name: "Zipformer 中英双语混合量化实验版",
  revision: "98590b7ed6443e77b714204da2757d75e1a642f4",
  path: "/models/realtime",
  state: "not_downloaded",
  enabled: false,
  total_size_bytes: 199_313_605,
  downloaded_size_bytes: 0,
  ...patch,
});

describe("RealtimePreviewModelCard", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.progressListener = undefined;
    mocks.info.mockResolvedValue(model());
  });

  it("renders typed multi-file download progress", async () => {
    mocks.download.mockReturnValue(new Promise(() => {}));
    render(<RealtimePreviewModelCard onChanged={vi.fn()} />);

    expect(await screen.findByText(/录音时显示增量字幕/)).toBeInTheDocument();

    fireEvent.click(await screen.findByRole("button", { name: "下载模型" }));
    await waitFor(() => expect(mocks.progressListener).toBeDefined());
    mocks.progressListener?.({
      payload: {
        file_name: "encoder.int8.onnx",
        file_index: 1,
        file_count: 6,
        downloaded_bytes: 42,
        total_bytes: 100,
        percent: 42,
      },
    });

    expect(
      await screen.findByText(/encoder\.int8\.onnx（1\/6）/),
    ).toBeInTheDocument();
    expect(screen.getByText("42%")).toBeInTheDocument();
  });

  it("confirms deletion and reports the disabled model state", async () => {
    const installed = model({
      state: "ready",
      enabled: true,
      downloaded_size_bytes: 199_313_605,
    });
    mocks.info.mockResolvedValue(installed);
    mocks.remove.mockResolvedValue(model());
    vi.spyOn(window, "confirm").mockReturnValue(true);
    render(<RealtimePreviewModelCard onChanged={vi.fn()} />);
    fireEvent.click(await screen.findByRole("button", { name: "删除" }));

    await waitFor(() => expect(mocks.remove).toHaveBeenCalledOnce());
    expect(screen.getByText(/下载约/)).toBeInTheDocument();
  });

  it("syncs a persisted toggle into the config controller snapshot", async () => {
    mocks.info.mockResolvedValue(model({ state: "ready" }));
    mocks.toggle.mockResolvedValue(model({ state: "ready", enabled: true }));
    const onConfigSync = vi.fn();
    const view = render(
      <RealtimePreviewModelCard
        onChanged={vi.fn()}
        onConfigSync={onConfigSync}
      />,
    );

    fireEvent.click(
      await within(view.container).findByRole("switch", {
        name: "启用实时预览",
      }),
    );

    await waitFor(() => {
      expect(mocks.toggle).toHaveBeenCalledWith(true);
      expect(onConfigSync).toHaveBeenCalledWith({
        realtime_preview_enabled: true,
      });
    });
  });
});
