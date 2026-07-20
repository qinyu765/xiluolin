import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  completedListener: undefined as
    ((event: { payload: RecordingPayload }) => void) | undefined,
  errorListener: undefined as
    ((event: { payload: string }) => void) | undefined,
  commands: {
    readInputReadiness: vi.fn(),
    startRecording: vi.fn(),
    stopRecording: vi.fn(),
    processRecordingFile: vi.fn(),
    deliverText: vi.fn(),
    abortCaptureSession: vi.fn(),
    openMacosPrivacySettings: vi.fn(),
    processUploadedAudio: vi.fn(),
  },
}));

type RecordingPayload = {
  session_id: string;
  file_path: string;
  duration_ms: number;
};

vi.mock("@/generated/tauri-bindings", () => ({
  commands: mocks.commands,
  events: {
    recordingCompleted: {
      listen: vi.fn(async (listener) => {
        mocks.completedListener = listener;
        return vi.fn();
      }),
    },
    recordingError: {
      listen: vi.fn(async (listener) => {
        mocks.errorListener = listener;
        return vi.fn();
      }),
    },
  },
}));

vi.mock("sonner", () => ({
  toast: {
    error: vi.fn(),
    success: vi.fn(),
    warning: vi.fn(),
  },
}));

import { useRecordingController } from "./useRecordingController";

const recording: RecordingPayload = {
  session_id: "session-1",
  file_path: "/managed/recording.wav",
  duration_ms: 1200,
};

const voiceResult = {
  raw_text: "raw",
  final_text: "final",
  actual_asr_provider: "local",
  actual_asr_model: "base",
  used_asr_fallback: false,
  used_text_fallback: false,
  history_record: null,
};

describe("useRecordingController", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.completedListener = undefined;
    mocks.errorListener = undefined;
    mocks.commands.readInputReadiness.mockResolvedValue({ can_process: true });
    mocks.commands.startRecording.mockResolvedValue({
      session_id: "session-1",
    });
    mocks.commands.stopRecording.mockResolvedValue(recording);
    mocks.commands.processRecordingFile.mockResolvedValue(voiceResult);
    mocks.commands.deliverText.mockResolvedValue({
      success: true,
      message: "已复制",
    });
    mocks.commands.abortCaptureSession.mockResolvedValue(null);
  });

  it("uses the shared completion pipeline for manual recording", async () => {
    const reloadHistory = vi.fn().mockResolvedValue(undefined);
    const { result } = renderHook(() => useRecordingController(reloadHistory));

    await act(() => result.current.startRecording());
    expect(result.current.phase).toBe("recording");

    await act(() => result.current.stopRecording());

    expect(mocks.commands.processRecordingFile).toHaveBeenCalledWith(
      "session-1",
      "/managed/recording.wav",
      1200,
    );
    expect(mocks.commands.deliverText).toHaveBeenCalledWith(
      "session-1",
      null,
      "final",
    );
    expect(reloadHistory).toHaveBeenCalledTimes(1);
    expect(result.current.phase).toBe("ready");
  });

  it("deduplicates repeated completion events by session id", async () => {
    let resolveProcessing: ((value: typeof voiceResult) => void) | undefined;
    mocks.commands.processRecordingFile.mockReturnValue(
      new Promise((resolve) => {
        resolveProcessing = resolve;
      }),
    );
    const { result } = renderHook(() =>
      useRecordingController(vi.fn().mockResolvedValue(undefined)),
    );

    await waitFor(() => expect(mocks.completedListener).toBeTypeOf("function"));
    act(() => {
      mocks.completedListener?.({ payload: recording });
      mocks.completedListener?.({ payload: recording });
    });

    expect(mocks.commands.processRecordingFile).toHaveBeenCalledTimes(1);
    await act(async () => resolveProcessing?.(voiceResult));
    await waitFor(() => expect(result.current.phase).toBe("ready"));
  });
});
