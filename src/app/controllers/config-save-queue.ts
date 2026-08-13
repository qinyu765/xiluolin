export type ConfigSaveState =
  | { status: "idle" }
  | { status: "pending" }
  | { status: "saving" }
  | { status: "saved" }
  | { status: "invalid" | "error"; message: string };

type SaveMode = "immediate" | "debounced";

export function createConfigSaveQueue<T>({
  save,
  prepare = (config) => config,
  validate = () => null,
  onSaved = () => undefined,
  onStateChange = () => undefined,
  delayMs = 600,
}: {
  save: (config: T) => Promise<T>;
  prepare?: (config: T) => T;
  validate?: (config: T) => string | null;
  onSaved?: (config: T, isLatest: boolean) => void;
  onStateChange?: (state: ConfigSaveState) => void;
  delayMs?: number;
}) {
  let pending: { config: T; version: number } | null = null;
  let latestVersion = 0;
  let timer: ReturnType<typeof setTimeout> | null = null;
  let running: Promise<void> | null = null;

  const clearTimer = () => {
    if (timer) clearTimeout(timer);
    timer = null;
  };

  const drain = async () => {
    while (pending) {
      const next = pending;
      const prepared = prepare(next.config);
      const validationError = validate(prepared);
      if (validationError) {
        onStateChange({ status: "invalid", message: validationError });
        return;
      }

      pending = null;
      onStateChange({ status: "saving" });
      try {
        const saved = await save(prepared);
        onSaved(saved, next.version === latestVersion && pending === null);
      } catch (error) {
        if (!pending) pending = next;
        onStateChange({
          status: "error",
          message: error instanceof Error ? error.message : String(error),
        });
        return;
      }
    }
    onStateChange({ status: "saved" });
  };

  const startDrain = () => {
    clearTimer();
    if (running) return running;
    running = drain().finally(() => {
      running = null;
    });
    return running;
  };

  return {
    update(config: T, mode: SaveMode) {
      pending = { config, version: ++latestVersion };
      onStateChange({ status: "pending" });
      clearTimer();
      if (mode === "immediate") {
        void startDrain();
      } else {
        timer = setTimeout(() => void startDrain(), delayMs);
      }
    },
    flush() {
      return startDrain();
    },
    retry() {
      return startDrain();
    },
    dispose() {
      clearTimer();
    },
  };
}
