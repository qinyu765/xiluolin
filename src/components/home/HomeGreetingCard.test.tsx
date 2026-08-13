import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { HomeGreetingCard } from "./HomeGreetingCard";

describe("HomeGreetingCard", () => {
  it("根据时间展示问候、当前人格和长按快捷键", () => {
    render(
      <HomeGreetingCard
        hour={9}
        personaName="翻译官"
        personaDescription="把中文翻译成自然流畅的英文"
        longpressShortcut="CommandOrControl+Shift+R"
        toggleShortcut="Alt+Space"
      />,
    );

    expect(screen.getByText("早上好 👋")).toBeInTheDocument();
    expect(
      screen.getByRole("heading", { name: "今天想说点什么？" }),
    ).toBeInTheDocument();
    expect(screen.getByText(/当前使用「翻译官」/)).toBeInTheDocument();
    expect(
      screen.getByText("按住 Ctrl + Shift + R 开始语音输入"),
    ).toBeInTheDocument();
  });

  it("在未选择人格和快捷键时给出可行动提示", () => {
    render(<HomeGreetingCard hour={21} />);

    expect(screen.getByText("晚上好 👋")).toBeInTheDocument();
    expect(screen.getByText(/还没有选择人格/)).toBeInTheDocument();
    expect(screen.getByText("前往设置配置快捷键")).toBeInTheDocument();
  });
});
