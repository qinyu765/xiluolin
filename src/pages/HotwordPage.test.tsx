import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { HotwordPage } from "./HotwordPage";

afterEach(cleanup);

function hotwords(count: number) {
  return Array.from({ length: count }, (_, index) => ({
    id: `hotword-${index}`,
    text: `词${index}`,
    category: "测试",
    enabled: true,
    created_at: "2026-01-01",
    updated_at: "2026-01-01",
  }));
}

function renderPage(count: number) {
  render(
    <HotwordPage
      hotwords={hotwords(count)}
      bulkText=""
      bulkCount={0}
      isLoading={false}
      isBulkDirty={false}
      isBulkSaving={false}
      onBulkTextChange={vi.fn()}
      onSaveBulk={vi.fn()}
      onClearBulk={vi.fn()}
      onCreateHotword={vi.fn()}
      onEditHotword={vi.fn()}
      onDeleteHotword={vi.fn()}
      onHotwordEnabledChange={vi.fn()}
    />,
  );
}

describe("HotwordPage", () => {
  it("使用独立的添加框，不回填已保存热词", () => {
    renderPage(2);

    expect(screen.getByRole("heading", { name: "热词" })).toBeInTheDocument();
    expect(screen.getByRole("textbox", { name: "添加热词" })).toHaveValue("");
  });

  it("在 101 个启用且稳定去重后的热词时显示智谱限定提示", () => {
    renderPage(101);

    expect(
      screen.getByText(/使用智谱 ASR 时仅前 100 个用于语音识别/),
    ).toBeInTheDocument();
  });

  it("在 100 个启用且稳定去重后的热词时不显示截断提示", () => {
    renderPage(100);

    expect(
      screen.queryByText(/使用智谱 ASR 时仅前 100 个用于语音识别/),
    ).not.toBeInTheDocument();
  });
});
