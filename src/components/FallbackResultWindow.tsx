import { useCallback, useEffect, useState } from "react";
import { ClipboardCheckIcon, CopyIcon, XIcon } from "lucide-react";
import { Toaster, toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { commands, type FallbackResult } from "@/generated/tauri-bindings";
import { toErrorMessage } from "@/utils/error";

export function FallbackResultWindow() {
  const [result, setResult] = useState<FallbackResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isCopying, setIsCopying] = useState(false);

  const refresh = useCallback(async () => {
    try {
      setResult(await commands.readFallbackResult());
      setError(null);
    } catch (cause) {
      setError(toErrorMessage(cause));
    }
  }, []);

  const dismiss = useCallback(async () => {
    try {
      await commands.dismissFallbackResult();
      setResult(null);
      setError(null);
    } catch (cause) {
      toast.error(`关闭结果窗口失败：${toErrorMessage(cause)}`);
    }
  }, []);

  useEffect(() => {
    void refresh();
    const handleUpdate = () => void refresh();
    window.addEventListener("fallback-result-updated", handleUpdate);
    return () =>
      window.removeEventListener("fallback-result-updated", handleUpdate);
  }, [refresh]);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        void dismiss();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [dismiss]);

  const copy = async () => {
    setIsCopying(true);
    try {
      await commands.copyFallbackResult();
      setResult((current) =>
        current ? { ...current, copied: true } : current,
      );
      toast.success("已复制到剪贴板");
    } catch (cause) {
      toast.error(`复制失败：${toErrorMessage(cause)}`);
    } finally {
      setIsCopying(false);
    }
  };

  if (!result) {
    return (
      <main className="flex min-h-screen items-center justify-center bg-background p-6">
        <Toaster position="top-center" richColors />
        <section className="w-full max-w-md space-y-4 rounded-xl border bg-card p-6 shadow-sm">
          <div>
            <h1 className="text-lg font-semibold">没有可用的失败结果</h1>
            <p className="mt-2 text-sm leading-6 text-muted-foreground">
              {error ?? "结果窗口正在同步，请稍候。"}
            </p>
          </div>
          <div className="flex justify-end">
            <Button
              type="button"
              variant="outline"
              onClick={() => void dismiss()}
            >
              <XIcon aria-hidden="true" />
              关闭
            </Button>
          </div>
        </section>
      </main>
    );
  }

  return (
    <main className="flex min-h-screen items-center justify-center bg-background p-6">
      <Toaster position="top-center" richColors />
      <section className="w-full max-w-2xl space-y-5 rounded-xl border bg-card p-6 shadow-sm">
        <div>
          <h1 className="text-lg font-semibold">未能自动输入</h1>
          <p className="mt-2 text-sm leading-6 text-muted-foreground">
            {result.reason}
          </p>
          <p className="mt-1 text-xs text-muted-foreground">
            {result.copied
              ? "结果已经复制到剪贴板，可以切回目标应用按 Command+V。"
              : "请使用下方按钮复制结果，再切回目标应用粘贴。"}
          </p>
        </div>

        <Textarea
          value={result.text}
          readOnly
          aria-label="失败结果文本"
          className="min-h-44 resize-y bg-background text-sm leading-6"
        />

        <div className="flex flex-wrap justify-end gap-2">
          <Button
            type="button"
            variant="outline"
            onClick={() => void dismiss()}
          >
            <XIcon aria-hidden="true" />
            关闭
          </Button>
          <Button
            type="button"
            onClick={() => void copy()}
            disabled={isCopying}
          >
            {result.copied ? (
              <ClipboardCheckIcon aria-hidden="true" />
            ) : (
              <CopyIcon aria-hidden="true" />
            )}
            {isCopying ? "复制中..." : result.copied ? "再次复制" : "复制结果"}
          </Button>
        </div>
      </section>
    </main>
  );
}
