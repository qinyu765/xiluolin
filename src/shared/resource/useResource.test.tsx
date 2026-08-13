import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { useResource } from "./useResource";

describe("useResource", () => {
  it("exposes data, loading, error and an explicit reload", async () => {
    const loader = vi
      .fn()
      .mockResolvedValueOnce("first")
      .mockResolvedValueOnce("second");
    const { result } = renderHook(() => useResource(loader));

    await waitFor(() => expect(result.current.data).toBe("first"));
    await act(() => result.current.reload());

    expect(result.current.data).toBe("second");
    expect(result.current.loading).toBe(false);
    expect(result.current.error).toBeNull();
  });
});
