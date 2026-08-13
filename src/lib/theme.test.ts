import { afterEach, describe, expect, it, vi } from "vitest";

import { enforceLightTheme } from "./theme";

describe("enforceLightTheme", () => {
  afterEach(() => {
    document.documentElement.className = "";
    document.documentElement.style.colorScheme = "";
  });

  it("同时固定 Web 内容与原生窗口为浅色", async () => {
    document.documentElement.classList.add("dark");
    const setTheme = vi.fn().mockResolvedValue(undefined);

    await enforceLightTheme(setTheme);

    expect(document.documentElement.classList.contains("dark")).toBe(false);
    expect(document.documentElement.style.colorScheme).toBe("light");
    expect(setTheme).toHaveBeenCalledWith("light");
  });
});
