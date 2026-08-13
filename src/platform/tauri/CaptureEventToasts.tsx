import { useEffect } from "react";
import { toast } from "sonner";

import { events } from "@/generated/tauri-bindings";

export function CaptureEventToasts() {
  useEffect(() => {
    let disposed = false;
    const disposers: Array<() => void> = [];
    const register = async (promise: Promise<() => void>) => {
      const dispose = await promise;
      if (disposed) dispose();
      else disposers.push(dispose);
    };

    void register(
      events.recordingError.listen((event) => toast.error(event.payload)),
    );
    void register(
      events.recordingLimitWarning.listen(() =>
        toast.warning("录音将在 3 秒后自动停止"),
      ),
    );

    return () => {
      disposed = true;
      disposers.forEach((dispose) => dispose());
    };
  }, []);

  return null;
}
