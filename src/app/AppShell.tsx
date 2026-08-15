import type { ReactNode } from "react";

export function AppShell({
  sidebar,
  children,
}: {
  sidebar: ReactNode;
  children: ReactNode;
}) {
  return (
    <main className="flex h-screen overflow-hidden">
      {sidebar}
      <section
        data-app-scroll-container
        role="region"
        aria-label="页面内容"
        className="h-full min-w-0 flex-1 overflow-x-hidden overflow-y-auto overscroll-contain"
      >
        <div className="mx-auto max-w-6xl px-6 py-8">{children}</div>
      </section>
    </main>
  );
}
