import { useCallback, useEffect, useState } from "react";
import {
  CheckCircle2Icon,
  DownloadIcon,
  Loader2Icon,
  RefreshCwIcon,
  Trash2Icon,
} from "lucide-react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { commands, events } from "@/generated/tauri-bindings";
import type {
  RealtimeModelDownloadProgress,
  RealtimeModelInfo,
} from "@/generated/tauri-bindings";
import { useResource } from "@/shared/resource/useResource";

function formatSize(bytes: number) {
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

export function RealtimePreviewModelCard({
  onEnabledChange,
  onChanged,
}: {
  onEnabledChange: (enabled: boolean) => void;
  onChanged: () => void;
}) {
  const load = useCallback(() => commands.realtimeAsrModelInfo(), []);
  const resource = useResource(load);
  const [progress, setProgress] =
    useState<RealtimeModelDownloadProgress | null>(null);
  const [action, setAction] = useState<
    "download" | "verify" | "toggle" | "delete" | null
  >(null);

  useEffect(() => {
    let dispose: (() => void) | undefined;
    let disposed = false;
    void events.realtimeAsrDownloadProgress
      .listen((event) => {
        setProgress(event.payload);
      })
      .then((nextDispose) => {
        if (disposed) nextDispose();
        else dispose = nextDispose;
      });
    return () => {
      disposed = true;
      dispose?.();
    };
  }, []);

  const finish = async (info: RealtimeModelInfo, message: string) => {
    onEnabledChange(info.enabled);
    onChanged();
    await resource.reload();
    toast.success(message);
  };

  const download = async () => {
    setAction("download");
    setProgress(null);
    try {
      await finish(
        await commands.downloadRealtimeAsrModel(),
        "实时预览模型已安装并启用",
      );
    } catch (error) {
      toast.error(`下载失败：${String(error)}`);
    } finally {
      setAction(null);
      setProgress(null);
    }
  };

  const verify = async () => {
    setAction("verify");
    try {
      await commands.verifyRealtimeAsrModel();
      await resource.reload();
      toast.success("模型文件校验通过");
    } catch (error) {
      toast.error(`模型校验失败：${String(error)}`);
      await resource.reload();
    } finally {
      setAction(null);
    }
  };

  const toggle = async (enabled: boolean) => {
    setAction("toggle");
    try {
      await finish(
        await commands.setRealtimePreviewEnabled(enabled),
        enabled ? "实时预览已启用" : "实时预览已停用",
      );
    } catch (error) {
      toast.error(`更新失败：${String(error)}`);
    } finally {
      setAction(null);
    }
  };

  const remove = async () => {
    if (!window.confirm("确定删除实时预览模型吗？最终语音识别不受影响。"))
      return;
    setAction("delete");
    try {
      await finish(
        await commands.deleteRealtimeAsrModel(),
        "实时预览模型已删除",
      );
    } catch (error) {
      toast.error(`删除失败：${String(error)}`);
    } finally {
      setAction(null);
    }
  };

  const info = resource.data;
  const installed = info?.state === "ready";
  const busy = action !== null;

  return (
    <Card>
      <CardHeader>
        <div className="flex items-start justify-between gap-4">
          <div>
            <CardTitle>录音实时预览</CardTitle>
            <CardDescription className="mt-1.5">
              实验性功能，默认关闭。本地 Zipformer
              只生成增量字幕；最终结果仍由上方 ASR 服务生成。
            </CardDescription>
          </div>
          <Switch
            aria-label="启用实时预览"
            checked={info?.enabled ?? false}
            disabled={!installed || busy}
            onCheckedChange={(enabled) => void toggle(enabled)}
          />
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="grid gap-3 rounded-lg border bg-muted/25 p-4 sm:grid-cols-[1fr_auto] sm:items-center">
          <div className="min-w-0">
            <p className="flex items-center gap-2 text-sm font-medium">
              {installed ? (
                <CheckCircle2Icon className="size-4 text-emerald-600" />
              ) : null}
              {info?.name ?? "Zipformer 中英双语 INT8"}
            </p>
            <p className="mt-1 text-xs leading-5 text-muted-foreground">
              {resource.error
                ? `读取失败：${resource.error}`
                : info?.state === "invalid"
                  ? "模型文件损坏，请重新下载或删除。"
                  : installed
                    ? `已安装 · ${formatSize(info.total_size_bytes)} · 固定版本 ${info.revision.slice(0, 8)}`
                    : `需要下载约 ${formatSize(info?.total_size_bytes ?? 0)}，下载完成后自动启用。`}
            </p>
          </div>
          <div className="flex flex-wrap gap-2">
            {!installed ? (
              <Button
                size="sm"
                disabled={busy || resource.loading}
                onClick={() => void download()}
              >
                {action === "download" ? (
                  <Loader2Icon className="size-4 animate-spin" />
                ) : (
                  <DownloadIcon className="size-4" />
                )}
                下载模型
              </Button>
            ) : (
              <>
                <Button
                  variant="outline"
                  size="sm"
                  disabled={busy}
                  onClick={() => void verify()}
                >
                  {action === "verify" ? (
                    <Loader2Icon className="size-4 animate-spin" />
                  ) : (
                    <RefreshCwIcon className="size-4" />
                  )}
                  重新校验
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  disabled={busy}
                  onClick={() => void remove()}
                >
                  {action === "delete" ? (
                    <Loader2Icon className="size-4 animate-spin" />
                  ) : (
                    <Trash2Icon className="size-4" />
                  )}
                  删除
                </Button>
              </>
            )}
          </div>
        </div>

        {action === "download" ? (
          <div aria-live="polite">
            <div className="mb-2 flex justify-between text-xs text-muted-foreground">
              <span>
                {progress
                  ? `正在下载 ${progress.file_name}（${progress.file_index}/${progress.file_count}）`
                  : "正在准备下载…"}
              </span>
              <span>{progress?.percent ?? 0}%</span>
            </div>
            <div className="h-1.5 overflow-hidden rounded-full bg-muted">
              <div
                className="h-full rounded-full bg-primary transition-[width]"
                style={{ width: `${progress?.percent ?? 0}%` }}
              />
            </div>
          </div>
        ) : null}
        <p className="text-xs leading-5 text-muted-foreground">
          候选模型的训练数据许可链、真实录音质量和目标设备稳定性尚未完成验证；模型需显式下载。
          预览异常、模型缺失或队列积压时会自动降级为阶段提示，不影响录音、最终识别、历史和投递。
        </p>
      </CardContent>
    </Card>
  );
}
