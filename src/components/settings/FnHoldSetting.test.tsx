import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { FnHoldSetting, isMacOSUserAgent } from "./FnHoldSetting";

afterEach(cleanup);

describe("FnHoldSetting", () => {
  it("只在 macOS 显示并把开关变化交给配置层", () => {
    const onCheckedChange = vi.fn();
    const { rerender } = render(
      <FnHoldSetting
        enabled={false}
        onCheckedChange={onCheckedChange}
        isMacOS={false}
      />,
    );
    expect(screen.queryByText("按住 Fn 录音")).not.toBeInTheDocument();

    rerender(
      <FnHoldSetting
        enabled={false}
        onCheckedChange={onCheckedChange}
        isMacOS
      />,
    );
    fireEvent.click(screen.getByRole("switch", { name: "按住 Fn 录音" }));
    expect(onCheckedChange).toHaveBeenCalledWith(true);
  });

  it("识别桌面 macOS 用户代理", () => {
    expect(
      isMacOSUserAgent(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
      ),
    ).toBe(true);
    expect(isMacOSUserAgent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")).toBe(
      false,
    );
  });
});
