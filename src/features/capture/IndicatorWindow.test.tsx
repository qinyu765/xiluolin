import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";

import { idleCaptureSnapshot } from "./captureSnapshot";
import { IndicatorContent } from "./IndicatorWindow";

describe("IndicatorContent", () => {
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

  it("shows the pipeline phase beside the frozen preview", () => {
    render(
      <IndicatorContent
        snapshot={{
          ...idleCaptureSnapshot,
          session_id: "session-1",
          revision: 4,
          phase: "refining",
          stable_text: "冻结结果",
          preview_state: "active",
        }}
      />,
    );

    expect(screen.getByText("润色中")).toBeInTheDocument();
    expect(screen.getByText("冻结结果")).toBeInTheDocument();
  });
});
