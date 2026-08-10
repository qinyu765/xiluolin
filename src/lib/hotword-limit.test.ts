import { describe, expect, it } from "vitest";

import { getEnabledHotwordAsrLimitNotice } from "./hotword-limit";

describe("getEnabledHotwordAsrLimitNotice", () => {
  it("对稳定去重后超过一百个启用热词给出 ASR 限制提示", () => {
    const hotwords = [
      { enabled: true, text: " Alpha " },
      { enabled: false, text: "停用词" },
      { enabled: true, text: "Alpha" },
      ...Array.from({ length: 100 }, (_, index) => ({
        enabled: true,
        text: `词${index}`,
      })),
    ];

    expect(getEnabledHotwordAsrLimitNotice(hotwords)).toBe(
      "已启用 101 个去重热词。使用智谱 ASR 时仅前 100 个用于语音识别；全部热词仍会用于文本整理。",
    );
  });

  it("未超过限制时不显示提示", () => {
    expect(
      getEnabledHotwordAsrLimitNotice([
        { enabled: true, text: " Alpha " },
        { enabled: true, text: "Alpha" },
      ]),
    ).toBeNull();
  });
});
