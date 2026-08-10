import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { PersonaPage } from "./PersonaPage";

describe("PersonaPage", () => {
  it("不为当前默认人格提供删除入口", () => {
    render(
      <PersonaPage
        personas={[
          {
            id: "verbatim",
            name: "原文听写",
            description: "保留原文",
            icon: "BookOpen",
            is_default: true,
            processing_mode: "verbatim",
            created_at: "2026-01-01",
            updated_at: "2026-01-01",
          },
          {
            id: "custom",
            name: "自定义人格",
            description: "自定义",
            icon: "Sparkles",
            is_default: false,
            processing_mode: "polish",
            created_at: "2026-01-01",
            updated_at: "2026-01-01",
          },
        ]}
        status="已加载"
        onCreatePersona={vi.fn()}
        onEditPersona={vi.fn()}
        onDeletePersona={vi.fn()}
        onSetDefaultPersona={vi.fn()}
      />,
    );

    expect(
      screen.queryByRole("button", { name: "删除 原文听写" }),
    ).not.toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "删除 自定义人格" }),
    ).toBeInTheDocument();
  });
});
