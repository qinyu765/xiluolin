import { Mic2Icon } from "lucide-react";

import { formatShortcutDisplay } from "@/utils/shortcut";

function greetingForHour(hour: number) {
  if (hour < 12) return "早上好";
  if (hour < 18) return "下午好";
  return "晚上好";
}

function shortcutHint(longpressShortcut?: string, toggleShortcut?: string) {
  if (longpressShortcut) {
    return `按住 ${formatShortcutDisplay(longpressShortcut)} 开始语音输入`;
  }
  if (toggleShortcut) {
    return `按 ${formatShortcutDisplay(toggleShortcut)} 开始/停止语音输入`;
  }
  return "前往设置配置快捷键";
}

export function HomeGreetingCard({
  hour = new Date().getHours(),
  personaName,
  personaDescription,
  longpressShortcut,
  toggleShortcut,
}: {
  hour?: number;
  personaName?: string;
  personaDescription?: string;
  longpressShortcut?: string;
  toggleShortcut?: string;
}) {
  return (
    <section className="relative isolate overflow-hidden rounded-xl border bg-card px-6 py-7 shadow-xs">
      <div
        className="absolute inset-y-0 left-0 w-1 bg-primary"
        aria-hidden="true"
      />
      <div
        className="pointer-events-none absolute right-0 top-5 hidden h-24 w-52 sm:block"
        aria-hidden="true"
      >
        <span className="absolute right-0 top-0 h-2.5 w-28 rounded-l-full bg-secondary" />
        <span className="absolute right-0 top-6 h-2.5 w-40 rounded-l-full bg-primary/25" />
        <span className="absolute right-0 top-12 h-2.5 w-24 rounded-l-full bg-primary/55" />
        <span className="absolute right-0 top-[4.5rem] h-2.5 w-32 rounded-l-full bg-secondary" />
      </div>

      <div className="relative max-w-full space-y-2 sm:max-w-[70%]">
        <p className="text-sm font-medium text-primary">
          {greetingForHour(hour)}！
        </p>
        <h2 className="text-3xl font-semibold tracking-tight">
          今天想说点什么？
        </h2>
        <p className="text-sm leading-6 text-muted-foreground">
          {personaName ? (
            <>
              当前使用「{personaName}」
              {personaDescription
                ? `：${personaDescription}`
                : "，随时可以开始。"}
            </>
          ) : (
            "还没有选择人格，先挑一个喜欢的表达方式吧。"
          )}
        </p>
      </div>

      <div className="relative mt-5 inline-flex min-h-8 items-center gap-2 rounded-full border bg-background/90 px-3 text-xs font-medium text-foreground shadow-xs">
        <Mic2Icon className="size-3.5 text-primary" aria-hidden="true" />
        {shortcutHint(longpressShortcut, toggleShortcut)}
      </div>
    </section>
  );
}
