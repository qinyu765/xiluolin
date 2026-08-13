import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { CaptureSnapshot } from "@/generated/tauri-bindings";
import { idleCaptureSnapshot } from "./captureSnapshot";
import { useCaptureSnapshot } from "./useCaptureSnapshot";

const mocks = vi.hoisted(() => ({
  read: vi.fn(),
  listener: undefined as
    undefined | ((event: { payload: CaptureSnapshot }) => void),
}));

vi.mock("@/generated/tauri-bindings", () => ({
  commands: { readCaptureSnapshot: mocks.read },
  events: {
    captureSnapshot: {
      listen: vi.fn(async (listener) => {
        mocks.listener = listener;
        return vi.fn();
      }),
    },
  },
}));

const snapshot = (
  sessionId: string,
  revision: number,
  phase: CaptureSnapshot["phase"],
): CaptureSnapshot => ({
  ...idleCaptureSnapshot,
  session_id: sessionId,
  revision,
  phase,
});

describe("useCaptureSnapshot", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.listener = undefined;
  });

  it("recovers an active session when the window mounts late", async () => {
    mocks.read.mockResolvedValue(snapshot("session-1", 7, "transcribing"));
    const { result } = renderHook(() => useCaptureSnapshot());

    await waitFor(() => expect(result.current.data.phase).toBe("transcribing"));
    expect(result.current.data.session_id).toBe("session-1");
  });

  it("does not let a stale initial read overwrite a newer event", async () => {
    let resolveRead: (value: CaptureSnapshot) => void = () => undefined;
    mocks.read.mockReturnValue(
      new Promise((resolve) => {
        resolveRead = resolve;
      }),
    );
    const { result } = renderHook(() => useCaptureSnapshot());
    await waitFor(() => expect(mocks.listener).toBeDefined());

    act(() =>
      mocks.listener?.({ payload: snapshot("session-2", 9, "refining") }),
    );
    await act(async () => resolveRead(snapshot("session-1", 8, "recording")));

    expect(result.current.data.phase).toBe("refining");
    expect(result.current.data.revision).toBe(9);
  });
});
