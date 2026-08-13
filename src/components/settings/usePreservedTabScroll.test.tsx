import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
} from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { usePreservedTabScroll } from "./usePreservedTabScroll";

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

function ScrollHarness() {
  const { activeTab, rootRef, onTabChange } = usePreservedTabScroll("general");

  return (
    <div data-app-scroll-container data-testid="scroller">
      <div ref={rootRef}>
        <button type="button" onClick={() => onTabChange("models")}>
          切换
        </button>
        <span>{activeTab}</span>
      </div>
    </div>
  );
}

describe("usePreservedTabScroll", () => {
  it("页签切换并完成布局后恢复右侧内容区位置", () => {
    const frames: FrameRequestCallback[] = [];
    vi.spyOn(window, "requestAnimationFrame").mockImplementation((callback) => {
      frames.push(callback);
      return frames.length;
    });
    const { container } = render(<ScrollHarness />);
    const scroller = screen.getByTestId("scroller");
    Object.defineProperties(scroller, {
      scrollHeight: { configurable: true, value: 1200 },
      clientHeight: { configurable: true, value: 500 },
    });
    scroller.scrollTop = 420;

    fireEvent.click(screen.getByRole("button", { name: "切换" }));
    scroller.scrollTop = 0;
    act(() => frames[frames.length - 1]?.(0));

    expect(container).toHaveTextContent("models");
    expect(scroller.scrollTop).toBe(420);
  });

  it("新内容较短时恢复到允许的最近位置", () => {
    const frames: FrameRequestCallback[] = [];
    vi.spyOn(window, "requestAnimationFrame").mockImplementation((callback) => {
      frames.push(callback);
      return frames.length;
    });
    render(<ScrollHarness />);
    const scroller = screen.getByTestId("scroller");
    Object.defineProperties(scroller, {
      scrollHeight: { configurable: true, value: 650 },
      clientHeight: { configurable: true, value: 500 },
    });
    scroller.scrollTop = 420;

    fireEvent.click(screen.getByRole("button", { name: "切换" }));
    scroller.scrollTop = 0;
    act(() => frames[frames.length - 1]?.(0));

    expect(scroller.scrollTop).toBe(150);
  });
});
