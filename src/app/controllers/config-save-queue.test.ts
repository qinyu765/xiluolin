import { afterEach, describe, expect, it, vi } from "vitest";

import { createConfigSaveQueue } from "./config-save-queue";

type TestConfig = { value: string };

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<T>((nextResolve, nextReject) => {
    resolve = nextResolve;
    reject = nextReject;
  });
  return { promise, resolve, reject };
}

describe("createConfigSaveQueue", () => {
  afterEach(() => {
    vi.useRealTimers();
  });

  it("文本修改等待 600ms 后保存最新版", async () => {
    vi.useFakeTimers();
    const save = vi.fn(async (config: TestConfig) => config);
    const queue = createConfigSaveQueue({ save, delayMs: 600 });

    queue.update({ value: "a" }, "debounced");
    queue.update({ value: "ab" }, "debounced");
    await vi.advanceTimersByTimeAsync(599);
    expect(save).not.toHaveBeenCalled();

    await vi.advanceTimersByTimeAsync(1);
    expect(save).toHaveBeenCalledTimes(1);
    expect(save).toHaveBeenCalledWith({ value: "ab" });
  });

  it("文本输入失焦时立即提交尚未到期的防抖修改", async () => {
    vi.useFakeTimers();
    const save = vi.fn(async (config: TestConfig) => config);
    const queue = createConfigSaveQueue({ save, delayMs: 600 });

    queue.update({ value: "draft" }, "debounced");
    await queue.flush();

    expect(save).toHaveBeenCalledOnce();
    expect(save).toHaveBeenCalledWith({ value: "draft" });
  });

  it("串行写入并合并飞行中的后续修改", async () => {
    const first = deferred<TestConfig>();
    const save = vi
      .fn<(config: TestConfig) => Promise<TestConfig>>()
      .mockReturnValueOnce(first.promise)
      .mockImplementation(async (config) => config);
    const saved = vi.fn();
    const queue = createConfigSaveQueue({ save, onSaved: saved });

    queue.update({ value: "a" }, "immediate");
    queue.update({ value: "ab" }, "immediate");
    queue.update({ value: "abc" }, "immediate");
    expect(save).toHaveBeenCalledTimes(1);

    first.resolve({ value: "a" });
    await queue.flush();

    expect(save).toHaveBeenCalledTimes(2);
    expect(save).toHaveBeenLastCalledWith({ value: "abc" });
    expect(saved).toHaveBeenNthCalledWith(1, { value: "a" }, false);
    expect(saved).toHaveBeenNthCalledWith(2, { value: "abc" }, true);
  });

  it("校验失败时不写入并在补全后继续", async () => {
    const states: string[] = [];
    const save = vi.fn(async (config: TestConfig) => config);
    const queue = createConfigSaveQueue({
      save,
      validate: (config) => (config.value ? null : "模型名不能为空。"),
      onStateChange: (state) => states.push(state.status),
    });

    queue.update({ value: "" }, "immediate");
    await queue.flush();
    expect(save).not.toHaveBeenCalled();
    expect(states[states.length - 1]).toBe("invalid");

    queue.update({ value: "ready" }, "immediate");
    await queue.flush();
    expect(save).toHaveBeenCalledWith({ value: "ready" });
    expect(states[states.length - 1]).toBe("saved");
  });

  it("保存失败后保留配置供手动重试", async () => {
    const states: string[] = [];
    const save = vi
      .fn<(config: TestConfig) => Promise<TestConfig>>()
      .mockRejectedValueOnce(new Error("磁盘不可用"))
      .mockImplementation(async (config) => config);
    const queue = createConfigSaveQueue({
      save,
      onStateChange: (state) => states.push(state.status),
    });

    queue.update({ value: "draft" }, "immediate");
    await queue.flush();
    expect(states[states.length - 1]).toBe("error");

    await queue.retry();
    expect(save).toHaveBeenCalledTimes(2);
    expect(save).toHaveBeenLastCalledWith({ value: "draft" });
    expect(states[states.length - 1]).toBe("saved");
  });
});
