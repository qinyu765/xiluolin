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
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import type {
  AppConfig,
  LocalAsrDownloadProgress,
  LocalAsrModelInfo,
} from "@/types";

function formatBytes(bytes: number) {
  return bytes >= 1024 * 1024
    ? `${(bytes / 1024 / 1024).toFixed(1)} MB`
    : `${(bytes / 1024).toFixed(1)} KB`;
}

export function LocalAsrSettings({
  config,
  onChange,
  onModelChanged,
}: {
  config: AppConfig;
  onChange: (config: AppConfig) => void;
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
    <div className="grid gap-4 rounded-lg border p-4">
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

      <div className="flex items-center justify-between rounded-lg border p-3">
        <div className="space-y-0.5">
          <Label htmlFor="local-asr-fallback">允许云端降级</Label>
          <p className="text-xs text-muted-foreground">
            默认关闭。开启后仅在本地识别失败时发送音频到指定云端 Provider
          </p>
        </div>
        <Switch
          id="local-asr-fallback"
          checked={config.allow_cloud_fallback}
          onCheckedChange={(checked) =>
            onChange({ ...config, allow_cloud_fallback: checked })
          }
        />
      </div>

      {config.allow_cloud_fallback && (
        <div className="grid gap-2">
          <Label htmlFor="fallback-asr-provider">云端降级 Provider</Label>
          <Select
            value={config.fallback_asr_provider}
            onValueChange={(value) =>
              onChange({ ...config, fallback_asr_provider: value })
            }
          >
            <SelectTrigger id="fallback-asr-provider">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="zhipu">智谱</SelectItem>
              <SelectItem value="openai">OpenAI 兼容</SelectItem>
            </SelectContent>
          </Select>
        </div>
      )}
    </div>
  );
}
