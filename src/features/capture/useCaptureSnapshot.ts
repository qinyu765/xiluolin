import { useCallback, useEffect, useReducer, useState } from "react";

import { commands, events } from "@/generated/tauri-bindings";
import { acceptCaptureSnapshot, idleCaptureSnapshot } from "./captureSnapshot";

const SNAPSHOT_REFRESH_EVENT = "capture-snapshot-refresh";

export function useCaptureSnapshot() {
  const [snapshot, receive] = useReducer(
    acceptCaptureSnapshot,
    idleCaptureSnapshot,
  );
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    try {
      receive(await commands.readCaptureSnapshot());
      setError(null);
    } catch (nextError) {
      setError(String(nextError));
    }
  }, []);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    const handleRefresh = () => void reload();

    window.addEventListener(SNAPSHOT_REFRESH_EVENT, handleRefresh);
    void events.captureSnapshot
      .listen((event) => receive(event.payload))
      .then((dispose) => {
        if (disposed) dispose();
        else unlisten = dispose;
      });
    void reload();

    return () => {
      disposed = true;
      unlisten?.();
      window.removeEventListener(SNAPSHOT_REFRESH_EVENT, handleRefresh);
    };
  }, [reload]);

  return {
    data: snapshot,
    loading: snapshot.revision === 0 && error === null,
    error,
    reload,
  };
}

export async function startCapture() {
  return commands.startCapture();
}

export async function stopCapture() {
  return commands.stopCapture();
}
