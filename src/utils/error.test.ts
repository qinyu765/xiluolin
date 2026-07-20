import { describe, expect, it } from "vitest";

import { toErrorMessage } from "./error";

describe("toErrorMessage", () => {
  it("returns Error messages", () => {
    expect(toErrorMessage(new Error("失败"))).toBe("失败");
  });

  it("preserves string errors", () => {
    expect(toErrorMessage("失败")).toBe("失败");
  });

  it("stringifies unknown values", () => {
    expect(toErrorMessage(42)).toBe("42");
  });
});
