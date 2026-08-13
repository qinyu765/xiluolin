import { describe, expect, it } from "vitest";

import type { CaptureSnapshot } from "@/generated/tauri-bindings";
import { acceptCaptureSnapshot, idleCaptureSnapshot } from "./captureSnapshot";

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
