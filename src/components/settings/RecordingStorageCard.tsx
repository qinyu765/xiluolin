import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { FolderOpenIcon, Loader2Icon, Trash2Icon } from "lucide-react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import type { RecordingStorageInfo } from "@/types";

function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

export function RecordingStorageCard() {
  const [info, setInfo] = useState<RecordingStorageInfo | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isClearing, setIsClearing] = useState(false);

  const refresh = useCallback(async () => {
    try {
      setInfo(await invoke<RecordingStorageInfo>("recording_storage_info"));
    } catch (error) {
      toast.error(`读取录音存储失败：${String(error)}`);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
    const handleHistoryChanged = () => void refresh();
    window.addEventListener("xiluolin-history-changed", handleHistoryChanged);
    return () =>
      window.removeEventListener("xiluolin-history-changed", handleHistoryChanged);
  }, [refresh]);

  const clearRecordings = async () => {
    if (!window.confirm("确定删除全部保留录音吗？历史文本不会被删除。")) return;
    setIsClearing(true);
    try {
      const next = await invoke<RecordingStorageInfo>("clear_retained_recordings");
      setInfo(next);
      window.dispatchEvent(new Event("xiluolin-history-changed"));
      toast.success("保留录音已清理");
    } catch (error) {
      toast.error(`清理录音失败：${String(error)}`);
    } finally {
      setIsClearing(false);
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>录音存储</CardTitle>
        <CardDescription>
          默认不保留录音；开启后仅保留成功关联到历史记录的应用录音
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="rounded-lg border p-3">
          <p className="text-sm font-medium">
            {isLoading
              ? "正在读取..."
              : `${info?.file_count ?? 0} 个录音 · ${formatBytes(info?.total_bytes ?? 0)}`}
          </p>
          <p className="mt-1 break-all text-xs text-muted-foreground">
            {info?.directory ?? "应用录音目录"}
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() =>
              void invoke("open_recordings_directory").catch((error) =>
                toast.error(`打开目录失败：${String(error)}`),
              )
            }
          >
            <FolderOpenIcon className="size-4" aria-hidden="true" />
            打开目录
          </Button>
          <Button
            type="button"
            variant="outline"
            size="sm"
            onClick={() => void clearRecordings()}
            disabled={isClearing || !info?.file_count}
          >
            {isClearing ? (
              <Loader2Icon className="size-4 animate-spin" aria-hidden="true" />
            ) : (
              <Trash2Icon className="size-4" aria-hidden="true" />
            )}
            清理全部录音
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
