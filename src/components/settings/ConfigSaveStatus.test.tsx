import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { ConfigSaveStatus } from "./ConfigSaveStatus";

afterEach(cleanup);

describe("ConfigSaveStatus", () => {
  it("保存成功时只显示轻量状态", () => {
    render(
      <ConfigSaveStatus
        state={{ status: "saved" }}
        onRetry={() => undefined}
      />,
    );

    expect(screen.getByText("已保存")).toBeInTheDocument();
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });

  it("保存失败时显示原因并允许重试", () => {
    const onRetry = vi.fn();
    render(
      <ConfigSaveStatus
        state={{ status: "error", message: "网络不可用" }}
        onRetry={onRetry}
      />,
    );

    expect(screen.getByText("保存失败：网络不可用")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "重试" }));
    expect(onRetry).toHaveBeenCalledOnce();
  });

  it("配置不完整时显示待补全且不提供重试", () => {
    render(
      <ConfigSaveStatus
        state={{ status: "invalid", message: "模型名不能为空" }}
        onRetry={() => undefined}
      />,
    );

    expect(screen.getByText("待补全：模型名不能为空")).toBeInTheDocument();
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });
});
