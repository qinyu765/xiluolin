import { useCallback, useEffect, useState } from "react";
import { FolderOpenIcon, Loader2Icon, Trash2Icon } from "lucide-react";
import { toast } from "sonner";

import { Button } from "@/components/ui/button";
import { commands } from "@/generated/tauri-bindings";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import type { SettingsSaveMode } from "@/components/settings/settings-schema";
import type { AppConfig, RecordingStorageInfo } from "@/types";

type RecordingStorageCardProps = {
  appConfig: AppConfig | null;
  onConfigChange: (
    patch: Partial<AppConfig>,
    saveMode: SettingsSaveMode,
  ) => void;
};

function formatBytes(bytes: number) {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

export function RecordingStorageCard({
  appConfig,
  onConfigChange,
}: RecordingStorageCardProps) {
  const [info, setInfo] = useState<RecordingStorageInfo | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isClearing, setIsClearing] = useState(false);

  const refresh = useCallback(async () => {
    try {
      setInfo(await commands.recordingStorageInfo());
    } catch (error) {
      toast.error(`读取录音存储失败：${String(error)}`);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const clearRecordings = async () => {
    if (!window.confirm("确定删除全部保留录音吗？历史文本不会被删除。")) return;
    setIsClearing(true);
    try {
      const next = await commands.clearRetainedRecordings();
      setInfo(next);
      toast.success("保留录音已清理");
    } catch (error) {
      toast.error(`清理录音失败：${String(error)}`);
    } finally {
      setIsClearing(false);
    }
  };

  return (
    <Card className="gap-4 py-4">
      <CardHeader className="flex flex-col gap-3 px-5 sm:flex-row sm:items-start sm:justify-between sm:gap-4">
        <div className="min-w-0 space-y-1.5">
          <CardTitle>录音存储</CardTitle>
          <CardDescription>保留后可在历史记录中试听</CardDescription>
        </div>
        <div className="flex items-center justify-between gap-3 sm:shrink-0">
          <Label htmlFor="retain-recordings">保留原始录音</Label>
          <Switch
            id="retain-recordings"
            checked={appConfig?.retain_recordings ?? false}
            disabled={!appConfig?.auto_save_history}
            onCheckedChange={(checked) =>
              onConfigChange({ retain_recordings: checked }, "immediate")
            }
          />
        </div>
      </CardHeader>
      <CardContent className="space-y-3 px-5">
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
              void commands
                .openRecordingsDirectory()
                .catch((error) => toast.error(`打开目录失败：${String(error)}`))
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
