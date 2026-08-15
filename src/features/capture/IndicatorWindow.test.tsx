import { act, cleanup, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { idleCaptureSnapshot } from "./captureSnapshot";
import { IndicatorContent } from "./IndicatorWindow";

describe("IndicatorContent", () => {
  afterEach(cleanup);

  beforeEach(() => {
    Object.defineProperty(HTMLElement.prototype, "scrollWidth", {
      configurable: true,
      get: () => 420,
    });
  });

  it("keeps the latest transcript tail visible in one horizontal line", async () => {
    render(
      <IndicatorContent
        snapshot={{
          ...idleCaptureSnapshot,
          session_id: "session-1",
          revision: 3,
          phase: "recording",
          preview_state: "active",
          stable_text: "这是一段已经确认的很长文本",
          tentative_text: "这里是最新识别尾部",
        }}
      />,
    );

    const transcript = screen.getByTestId("indicator-transcript");
    await waitFor(() => expect(transcript.scrollLeft).toBe(420));
    expect(transcript).toHaveTextContent(
      "这是一段已经确认的很长文本这里是最新识别尾部",
    );
  });

  it.each([
    ["transcribing", "识别中"],
    ["refining", "整理中"],
    ["delivering", "输入中"],
  ] as const)("shows the centered message for %s", (phase, message) => {
    render(
      <IndicatorContent
        snapshot={{
          ...idleCaptureSnapshot,
          session_id: "session-1",
          revision: 4,
          phase,
          stable_text: "冻结结果",
          preview_state: "active",
        }}
      />,
    );

    const indicator = screen.getByTestId("indicator-message");
    expect(indicator).toHaveTextContent(message);
    expect(
      screen.queryByTestId("indicator-transcript"),
    ).not.toBeInTheDocument();
    expect(screen.queryByText("冻结结果")).not.toBeInTheDocument();
  });

  it.each(["disabled", "loading", "unavailable"] as const)(
    "shows recognition while realtime preview is %s",
    (previewState) => {
      render(
        <IndicatorContent
          snapshot={{
            ...idleCaptureSnapshot,
            session_id: "session-1",
            revision: 5,
            phase: "recording",
            preview_state: previewState,
            stable_text: "已有文本",
          }}
        />,
      );

      expect(screen.getByTestId("indicator-message")).toHaveTextContent(
        "识别中",
      );
    },
  );

  it("shows recognition until an active preview has text", () => {
    render(
      <IndicatorContent
        snapshot={{
          ...idleCaptureSnapshot,
          session_id: "session-1",
          revision: 6,
          phase: "recording",
          preview_state: "active",
        }}
      />,
    );

    expect(screen.getByTestId("indicator-message")).toHaveTextContent("识别中");
  });

  it("keeps short terminal messages without side status or a timer", () => {
    const { rerender } = render(
      <IndicatorContent
        snapshot={{
          ...idleCaptureSnapshot,
          revision: 7,
          phase: "completed",
        }}
      />,
    );

    const completed = screen.getByTestId("indicator-message");
    expect(completed).toHaveTextContent("已输入");
    expect(screen.queryByText("00:00")).not.toBeInTheDocument();

    rerender(
      <IndicatorContent
        snapshot={{
          ...idleCaptureSnapshot,
          revision: 8,
          phase: "failed",
          failure: {
            code: "network",
            stage: "transcribing",
            recoverable: true,
            detail: "网络异常",
          },
        }}
      />,
    );

    const failed = screen.getByTestId("indicator-message");
    expect(failed).toHaveTextContent("网络异常");
  });

  it("shows a colored clipboard notice in the floating indicator", () => {
    render(
      <IndicatorContent
        snapshot={{
          ...idleCaptureSnapshot,
          revision: 9,
          phase: "completed",
        }}
      />,
    );

    act(() => {
      window.dispatchEvent(
        new CustomEvent("capture-indicator-notice", {
          detail: { text: "结果已复制", tone: "copied" },
        }),
      );
    });

    expect(screen.getByTestId("indicator-message")).toHaveTextContent(
      "结果已复制",
    );
    expect(screen.getByTestId("indicator-message").parentElement).toHaveClass(
      "indicator-shell--notice-copied",
    );
  });
});
