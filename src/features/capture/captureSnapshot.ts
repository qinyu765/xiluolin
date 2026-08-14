import type { CaptureSnapshot } from "@/generated/tauri-bindings";

export const idleCaptureSnapshot: CaptureSnapshot = {
  session_id: null,
  revision: 0,
  source: null,
  phase: "idle",
  elapsed_ms: 0,
  stable_text: "",
  tentative_text: "",
  preview_state: "disabled",
  history_id: null,
  failure: null,
};

export function acceptCaptureSnapshot(
  current: CaptureSnapshot,
  incoming: CaptureSnapshot,
) {
  // Rust owns a process-wide monotonic revision. Comparing it globally also
  // prevents a delayed event from the previous session replacing a new one.
  if (incoming.revision <= current.revision) {
    return current;
  }
  return incoming;
}

export function captureDisplayText(snapshot: CaptureSnapshot) {
  const stable = snapshot.stable_text.trim();
  const tentative = snapshot.tentative_text.trim();
  if (!stable) return tentative;
  if (!tentative) return stable;
  const needsSpace =
    /[A-Za-z0-9]$/.test(stable) && /^[A-Za-z0-9]/.test(tentative);
  return `${stable}${needsSpace ? " " : ""}${tentative}`;
}
