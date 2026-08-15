import { describe, expect, it } from "vitest";

import type { CaptureSnapshot } from "@/generated/tauri-bindings";
import {
  acceptCaptureSnapshot,
  idleCaptureSnapshot,
  indicatorPresentation,
} from "./captureSnapshot";

const snapshot = (
  sessionId: string | null,
  revision: number,
  phase: CaptureSnapshot["phase"],
): CaptureSnapshot => ({
  ...idleCaptureSnapshot,
  session_id: sessionId,
  revision,
  phase,
});

describe("acceptCaptureSnapshot", () => {
  it("drops older revisions from the same session", () => {
    const current = snapshot("session-1", 4, "transcribing");
    const stale = snapshot("session-1", 3, "recording");

    expect(acceptCaptureSnapshot(current, stale)).toBe(current);
  });

  it("rejects a delayed snapshot from a previous session", () => {
    const current = snapshot("session-2", 12, "recording");
    const delayed = snapshot("session-1", 11, "completed");

    expect(acceptCaptureSnapshot(current, delayed)).toBe(current);
  });

  it("accepts a new session with the next global revision", () => {
    const completed = snapshot("session-1", 12, "completed");
    const next = snapshot("session-2", 13, "recording");

    expect(acceptCaptureSnapshot(completed, next)).toBe(next);
  });
});

describe("indicatorPresentation", () => {
  it("only exposes a transcript when active preview has text", () => {
    expect(
      indicatorPresentation({
        ...idleCaptureSnapshot,
        phase: "recording",
        preview_state: "active",
        stable_text: "确认文本",
      }),
    ).toEqual({ mode: "transcript", text: "确认文本" });

    expect(
      indicatorPresentation({
        ...idleCaptureSnapshot,
        phase: "recording",
        preview_state: "active",
      }),
    ).toEqual({ mode: "message", text: "识别中" });
  });

  it.each([
    ["transcribing", "识别中"],
    ["refining", "整理中"],
    ["delivering", "输入中"],
  ] as const)("maps %s to the short processing copy", (phase, text) => {
    expect(indicatorPresentation({ ...idleCaptureSnapshot, phase })).toEqual({
      mode: "message",
      text,
    });
  });

  it("uses the terminal copy and failure detail", () => {
    expect(
      indicatorPresentation({ ...idleCaptureSnapshot, phase: "completed" }),
    ).toEqual({ mode: "completed", text: "已输入" });
    expect(
      indicatorPresentation({
        ...idleCaptureSnapshot,
        phase: "failed",
        failure: {
          code: "network",
          stage: "transcribing",
          recoverable: true,
          detail: "网络异常",
        },
      }),
    ).toEqual({ mode: "failed", text: "网络异常" });
  });
});
