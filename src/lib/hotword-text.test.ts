import { describe, expect, it } from "vitest";

import { normalizeHotwordLines } from "./hotword-text";

describe("hotword text helpers", () => {
  it("cleans blank lines, whitespace and duplicates while preserving order", () => {
    expect(
      normalizeHotwordLines("  XiLuoLin  \n\nNext.js\nXiLuoLin\n Next.js "),
    ).toEqual(["XiLuoLin", "Next.js"]);
  });
});
