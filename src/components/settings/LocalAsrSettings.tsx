import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { DownloadIcon, Loader2Icon, ShieldCheckIcon, Trash2Icon } from "lucide-react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import type { AppConfig, LocalAsrDownloadProgress, LocalAsrModelInfo } from "@/types";

function formatBytes(bytes: number) {
  return bytes >= 1024 * 1024
    ? `${(bytes / 1024 / 1024).toFixed(1)} MB`
    : `${(bytes / 1024).toFixed(1)} KB`;
}

export function LocalAsrSettings({
  config,
  onChange,
}: {
  config: AppConfig;
  onChange: (config: AppConfig) => void;
}) {
  const [model, setModel] = useState<LocalAsrModelInfo | null>(null);
  const [progress, setProgress] = useState<LocalAsrDownloadProgress | null>(null);
  const [action, setAction] = useState<"download" | "verify" | "delete" | null>(null);

  const refresh = useCallback(async () => {
    setModel(await invoke<LocalAsrModelInfo>("local_asr_model_info"));
  }, []);

  useEffect(() => {
    void refresh().catch((error) => toast.error(`读取本地模型失败：${String(error)}`));
    let unlisten: (() => void) | undefined;
    void listen<LocalAsrDownloadProgress>("local-asr-download-progress", (event) => {
      setProgress(event.payload);
    }).then((dispose) => {
      unlisten = dispose;
    });
    return () => unlisten?.();
  }, [refresh]);

  const run = async (nextAction: typeof action, command: string, success: string) => {
    setAction(nextAction);
    try {
      const result = await invoke<LocalAsrModelInfo | void>(command);
      if (result) setModel(result as LocalAsrModelInfo);
      else await refresh();
      window.dispatchEvent(new Event("xiluolin-config-saved"));
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
            下载进度：{progress?.percent != null ? `${progress.percent}%` : "准备中"}
          </p>
        )}
      </div>

      <div className="flex flex-wrap gap-2">
        <Button
          type="button"
          variant="outline"
          size="sm"
          disabled={action !== null || model?.exists}
          onClick={() => void run("download", "download_local_asr_model", "模型下载成功")}
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
          onClick={() => void run("verify", "verify_local_asr_model", "模型验证成功")}
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
              void run("delete", "delete_local_asr_model", "模型删除成功");
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
          onCheckedChange={(checked) => onChange({ ...config, allow_cloud_fallback: checked })}
        />
      </div>

      {config.allow_cloud_fallback && (
        <div className="grid gap-2">
          <Label htmlFor="fallback-asr-provider">云端降级 Provider</Label>
          <Select
            value={config.fallback_asr_provider}
            onValueChange={(value) => onChange({ ...config, fallback_asr_provider: value })}
          >
            <SelectTrigger id="fallback-asr-provider">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="zhipu">智谱 GLM-ASR</SelectItem>
              <SelectItem value="openai">OpenAI Whisper</SelectItem>
            </SelectContent>
          </Select>
        </div>
      )}
    </div>
  );
}
