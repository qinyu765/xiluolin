import { useCallback, useLayoutEffect, useRef, useState } from "react";

const SCROLL_CONTAINER_SELECTOR = "[data-app-scroll-container]";

export function usePreservedTabScroll(initialTab: string) {
  const [activeTab, setActiveTab] = useState(initialTab);
  const rootRef = useRef<HTMLDivElement>(null);
  const pendingScrollTopRef = useRef<number | null>(null);

  const onTabChange = useCallback((nextTab: string) => {
    const scrollContainer = rootRef.current?.closest<HTMLElement>(
      SCROLL_CONTAINER_SELECTOR,
    );
    pendingScrollTopRef.current = scrollContainer?.scrollTop ?? null;
    setActiveTab(nextTab);
  }, []);

  useLayoutEffect(() => {
    const requestedScrollTop = pendingScrollTopRef.current;
    const scrollContainer = rootRef.current?.closest<HTMLElement>(
      SCROLL_CONTAINER_SELECTOR,
    );
    if (requestedScrollTop === null || !scrollContainer) return;

    const frame = window.requestAnimationFrame(() => {
      const maxScrollTop = Math.max(
        0,
        scrollContainer.scrollHeight - scrollContainer.clientHeight,
      );
      scrollContainer.scrollTop = Math.min(requestedScrollTop, maxScrollTop);
      pendingScrollTopRef.current = null;
    });

    return () => window.cancelAnimationFrame(frame);
  }, [activeTab]);

  return { activeTab, rootRef, onTabChange };
}
