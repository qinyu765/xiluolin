import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { AppSidebar } from "./AppSidebar";
import { AppShell } from "./AppShell";

describe("AppShell", () => {
  it("把侧栏固定在布局中并只允许内容区滚动", () => {
    render(
      <AppShell
        sidebar={<AppSidebar page="home" onPageChange={() => undefined} />}
      >
        <p>页面内容</p>
      </AppShell>,
    );

    expect(screen.getByRole("main")).toHaveClass("h-screen", "overflow-hidden");
    expect(screen.getByTestId("app-sidebar")).toHaveClass(
      "h-full",
      "shrink-0",
    );
    expect(screen.getByTestId("app-sidebar")).not.toHaveClass("fixed");
    expect(screen.getByRole("region", { name: "页面内容" })).toHaveClass(
      "h-full",
      "overflow-y-auto",
      "overscroll-contain",
    );
  });
});
