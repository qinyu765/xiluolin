import { useEffect, useRef, useState } from "react";

import type { CaptureSnapshot } from "@/generated/tauri-bindings";
import { indicatorPresentation } from "./captureSnapshot";
import { useCaptureSnapshot } from "./useCaptureSnapshot";

type IndicatorNotice = {
  text: string;
  tone: "copied" | "failed";
};

function parseIndicatorNotice(value: unknown): IndicatorNotice | null {
  if (typeof value !== "object" || value === null) return null;
  const detail = value as Record<string, unknown>;
  if (typeof detail.text !== "string") return null;
  if (detail.tone !== "copied" && detail.tone !== "failed") return null;
  return { text: detail.text, tone: detail.tone };
}

export function IndicatorContent({ snapshot }: { snapshot: CaptureSnapshot }) {
  const transcriptRef = useRef<HTMLDivElement>(null);
  const [notice, setNotice] = useState<IndicatorNotice | null>(null);
  const presentation = indicatorPresentation(snapshot);

  useEffect(() => {
    const handleNotice = (event: Event) => {
      const nextNotice = parseIndicatorNotice(
        (event as CustomEvent<unknown>).detail,
      );
      if (nextNotice) setNotice(nextNotice);
    };
    window.addEventListener("capture-indicator-notice", handleNotice);
    return () =>
      window.removeEventListener("capture-indicator-notice", handleNotice);
  }, []);

  useEffect(() => {
    if (!notice) return;
    const timeout = window.setTimeout(() => setNotice(null), 1_500);
    return () => window.clearTimeout(timeout);
  }, [notice]);

  const mode = notice ? "notice" : presentation.mode;
  const text = notice?.text ?? presentation.text;
  const isTranscript = mode === "transcript";

  useEffect(() => {
    if (!isTranscript) return;
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
  }, [isTranscript, text]);

  return (
    <div
      className={`indicator-shell indicator-shell--${mode}${
        notice ? ` indicator-shell--notice-${notice.tone}` : ""
      }`}
    >
      <div
        ref={isTranscript ? transcriptRef : undefined}
        data-testid={
          isTranscript ? "indicator-transcript" : "indicator-message"
        }
        className={isTranscript ? "indicator-transcript" : "indicator-message"}
        aria-live="polite"
      >
        {text}
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
