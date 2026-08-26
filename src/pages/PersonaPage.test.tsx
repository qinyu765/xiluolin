import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { PersonaPage } from "./PersonaPage";

describe("PersonaPage", () => {
  it("为所有人格显示删除入口并禁用通用人格", () => {
    const onSelectPersona = vi.fn();
    const onRequestDeletePersona = vi.fn();
    render(
      <PersonaPage
        personas={[
          {
            id: "general",
            name: "通用人格",
            description: "保持自然",
            icon: "BookOpen",
            is_default: false,
            processing_mode: "verbatim",
            created_at: "2026-01-01",
            updated_at: "2026-01-01",
          },
          {
            id: "custom",
            name: "自定义人格",
            description: "自定义",
            icon: "Sparkles",
            is_default: true,
            processing_mode: "polish",
            created_at: "2026-01-01",
            updated_at: "2026-01-01",
          },
          {
            id: "other",
            name: "其他人格",
            description: "其他",
            icon: "Sparkles",
            is_default: false,
            processing_mode: "polish",
            created_at: "2026-01-01",
            updated_at: "2026-01-01",
          },
        ]}
        onCreatePersona={vi.fn()}
        onEditPersona={vi.fn()}
        onRequestDeletePersona={onRequestDeletePersona}
        onSelectPersona={onSelectPersona}
      />,
    );

    expect(
      screen.getByRole("button", { name: "删除 通用人格" }),
    ).toBeDisabled();
    expect(
      screen.getByRole("button", { name: "删除 自定义人格" }),
    ).toBeEnabled();
    expect(screen.getByRole("button", { name: "删除 其他人格" })).toBeEnabled();
    expect(screen.queryByText("设为默认")).not.toBeInTheDocument();

    screen.getByRole("button", { name: "删除 其他人格" }).click();
    expect(onRequestDeletePersona).toHaveBeenCalledWith(
      expect.objectContaining({ id: "other" }),
    );

    screen.getByRole("button", { name: "选择 其他人格 作为默认人格" }).click();
    expect(onSelectPersona).toHaveBeenCalledWith("other");
  });
});
