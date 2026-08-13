import { useEffect, useRef, useState } from "react";
import { AudioLinesIcon, CheckIcon, LoaderCircleIcon } from "lucide-react";

import type { CaptureSnapshot } from "@/generated/tauri-bindings";
import { captureDisplayText } from "./captureSnapshot";
import { useCaptureSnapshot } from "./useCaptureSnapshot";

const PHASE_LABEL: Record<CaptureSnapshot["phase"], string> = {
  idle: "等待输入",
  recording: "聆听中",
  transcribing: "转写中",
  refining: "润色中",
  delivering: "正在输入",
  completed: "已输入",
  failed: "处理失败",
};

function formatElapsed(elapsedMs: number) {
  const seconds = Math.max(0, Math.floor(elapsedMs / 1000));
  return `${String(Math.floor(seconds / 60)).padStart(2, "0")}:${String(seconds % 60).padStart(2, "0")}`;
}

export function IndicatorContent({ snapshot }: { snapshot: CaptureSnapshot }) {
  const transcriptRef = useRef<HTMLDivElement>(null);
  const text = captureDisplayText(snapshot);
  const [liveElapsed, setLiveElapsed] = useState(snapshot.elapsed_ms);

  useEffect(() => {
    setLiveElapsed(snapshot.elapsed_ms);
    if (snapshot.phase !== "recording") return;
    const startedAt = Date.now() - snapshot.elapsed_ms;
    const timer = window.setInterval(() => {
      setLiveElapsed(Date.now() - startedAt);
    }, 250);
    return () => window.clearInterval(timer);
  }, [snapshot.elapsed_ms, snapshot.phase, snapshot.session_id]);

  useEffect(() => {
    const node = transcriptRef.current;
    if (!node) return;
    const reducedMotion = window.matchMedia?.(
      "(prefers-reduced-motion: reduce)",
    ).matches;
    if (typeof node.scrollTo === "function") {
      node.scrollTo({
        left: node.scrollWidth,
        behavior: reducedMotion ? "auto" : "smooth",
      });
    } else {
      node.scrollLeft = node.scrollWidth;
    }
  }, [text]);

  const completed = snapshot.phase === "completed";
  const failed = snapshot.phase === "failed";
  const processing = ["transcribing", "refining", "delivering"].includes(
    snapshot.phase,
  );

  return (
    <div
      className={`indicator-shell ${completed ? "indicator-completed" : ""} ${failed ? "indicator-failed" : ""}`}
    >
      <span className="indicator-signal" aria-hidden="true">
        {completed ? (
          <CheckIcon />
        ) : processing ? (
          <LoaderCircleIcon className="indicator-spinner" />
        ) : (
          <AudioLinesIcon />
        )}
      </span>

      <div
        ref={transcriptRef}
        data-testid="indicator-transcript"
        className="indicator-transcript"
        aria-live="polite"
      >
        {failed && snapshot.failure?.detail
          ? snapshot.failure.detail
          : text ||
            (snapshot.preview_state === "unavailable"
              ? "实时预览不可用，录音仍在继续"
              : snapshot.phase === "recording"
                ? "请开始说话"
                : "正在处理语音")}
      </div>

      <div className="indicator-status">
        <span>{PHASE_LABEL[snapshot.phase]}</span>
        {snapshot.phase === "recording" ? (
          <time>{formatElapsed(liveElapsed)}</time>
        ) : null}
      </div>
    </div>
  );
}

export function IndicatorWindow() {
  const { data } = useCaptureSnapshot();
  return (
    <main className="indicator-stage">
      <IndicatorContent snapshot={data} />
    </main>
  );
}
