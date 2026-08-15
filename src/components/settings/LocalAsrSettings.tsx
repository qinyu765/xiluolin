import { useCallback, useEffect, useState } from "react";
import {
  DownloadIcon,
  Loader2Icon,
  ShieldCheckIcon,
  Trash2Icon,
} from "lucide-react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { commands, events } from "@/generated/tauri-bindings";
import type { LocalAsrDownloadProgress, LocalAsrModelInfo } from "@/types";

function formatBytes(bytes: number) {
  return bytes >= 1024 * 1024
    ? `${(bytes / 1024 / 1024).toFixed(1)} MB`
    : `${(bytes / 1024).toFixed(1)} KB`;
}

export function LocalAsrSettings({
  onModelChanged,
}: {
  onModelChanged?: () => void;
}) {
  const [model, setModel] = useState<LocalAsrModelInfo | null>(null);
  const [progress, setProgress] = useState<LocalAsrDownloadProgress | null>(
    null,
  );
  const [action, setAction] = useState<"download" | "verify" | "delete" | null>(
    null,
  );

  const refresh = useCallback(async () => {
    setModel(await commands.localAsrModelInfo());
  }, []);

  useEffect(() => {
    void refresh().catch((error) =>
      toast.error(`读取本地模型失败：${String(error)}`),
    );
    let unlisten: (() => void) | undefined;
    void events.localAsrDownloadProgress
      .listen((event) => {
        setProgress(event.payload);
      })
      .then((dispose) => {
        unlisten = dispose;
      });
    return () => unlisten?.();
  }, [refresh]);

  const run = async (
    nextAction: Exclude<typeof action, null>,
    operation: () => Promise<LocalAsrModelInfo | null>,
    success: string,
  ) => {
    setAction(nextAction);
    try {
      const result = await operation();
      if (result) setModel(result);
      else await refresh();
      onModelChanged?.();
      toast.success(success);
    } catch (error) {
      toast.error(`${success.replace("成功", "失败")}：${String(error)}`);
    } finally {
      setAction(null);
      setProgress(null);
    }
  };

  return (
    <div className="grid gap-3 rounded-lg border p-3">
      <div>
        <p className="text-sm font-medium">Whisper Base Q5_1</p>
        <p className="mt-1 break-all text-xs text-muted-foreground">
          {model?.exists
            ? `已下载 · ${formatBytes(model.size_bytes)} · ${model.path}`
            : "模型尚未下载。下载后可在无网络环境完成语音识别。"}
        </p>
        {action === "download" && (
          <p className="mt-2 text-xs text-muted-foreground">
            下载进度：
            {progress?.percent != null ? `${progress.percent}%` : "准备中"}
          </p>
        )}
      </div>

      <div className="flex flex-wrap gap-2">
        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={action !== null || model?.exists}
          onClick={() =>
            void run("download", commands.downloadLocalAsrModel, "模型下载成功")
          }
        >
          {action === "download" ? (
            <Loader2Icon className="size-4 animate-spin" aria-hidden="true" />
          ) : (
            <DownloadIcon className="size-4" aria-hidden="true" />
          )}
          下载模型
        </Button>
        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={action !== null || !model?.exists}
          onClick={() =>
            void run(
              "verify",
              async () => {
                await commands.verifyLocalAsrModel();
                return null;
              },
              "模型验证成功",
            )
          }
        >
          {action === "verify" ? (
            <Loader2Icon className="size-4 animate-spin" aria-hidden="true" />
          ) : (
            <ShieldCheckIcon className="size-4" aria-hidden="true" />
          )}
          验证模型
        </Button>
        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={action !== null || !model?.exists}
          onClick={() => {
            if (window.confirm("确定删除本地 ASR 模型吗？")) {
              void run("delete", commands.deleteLocalAsrModel, "模型删除成功");
            }
          }}
        >
          <Trash2Icon className="size-4" aria-hidden="true" />
          删除模型
        </Button>
      </div>
    </div>
  );
}
