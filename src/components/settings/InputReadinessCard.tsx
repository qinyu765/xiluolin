import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  AlertTriangleIcon,
  CheckCircle2Icon,
  ExternalLinkIcon,
  Loader2Icon,
  RefreshCwIcon,
  ShieldCheckIcon,
  XCircleIcon,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import type {
  InputReadiness,
  ReadinessAction,
  ReadinessCheck,
} from "@/types";

const CHECK_LABELS: Array<{
  key: keyof Pick<
    InputReadiness,
    "microphone" | "asr" | "text_processing" | "hotkey" | "auto_paste"
  >;
  label: string;
}> = [
  { key: "microphone", label: "麦克风" },
  { key: "asr", label: "语音识别" },
  { key: "text_processing", label: "文本处理" },
  { key: "hotkey", label: "全局快捷键" },
  { key: "auto_paste", label: "自动粘贴" },
];

const ACTION_LABELS: Record<ReadinessAction, string> = {
  request_microphone: "请求麦克风权限",
  open_microphone_settings: "打开麦克风设置",
  request_accessibility: "请求辅助功能权限",
  open_accessibility_settings: "打开辅助功能设置",
};

function StatusIcon({ check }: { check: ReadinessCheck }) {
  if (check.ready) {
    return <CheckCircle2Icon className="size-5 text-emerald-600" aria-hidden="true" />;
  }
  if (check.blocking) {
    return <XCircleIcon className="size-5 text-destructive" aria-hidden="true" />;
  }
  return <AlertTriangleIcon className="size-5 text-amber-500" aria-hidden="true" />;
}

function actionPermission(action: ReadinessAction) {
  return action.includes("microphone") ? "microphone" : "accessibility";
}

export function InputReadinessCard() {
  const [readiness, setReadiness] = useState<InputReadiness | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [pendingAction, setPendingAction] = useState<ReadinessAction | null>(null);

  const refresh = useCallback(async (showLoading = false) => {
    if (showLoading) setIsLoading(true);
    try {
      const result = await invoke<InputReadiness>("read_input_readiness");
      setReadiness(result);
      setError(null);
    } catch (readinessError) {
      setError(String(readinessError));
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    const handleRefresh = () => void refresh();
    void refresh(true);
    window.addEventListener("xiluolin-config-saved", handleRefresh);
    window.addEventListener("focus", handleRefresh);
    return () => {
      window.removeEventListener("xiluolin-config-saved", handleRefresh);
      window.removeEventListener("focus", handleRefresh);
    };
  }, [refresh]);

  const runAction = useCallback(
    async (action: ReadinessAction) => {
      setPendingAction(action);
      setError(null);
      try {
        const permission = actionPermission(action);
        if (action.startsWith("request_")) {
          await invoke("request_macos_permission", { permission });
          await refresh();
        } else {
          await invoke("open_macos_privacy_settings", { permission });
        }
      } catch (actionError) {
        setError(String(actionError));
      } finally {
        setPendingAction(null);
      }
    },
    [refresh],
  );

  const summary = readiness?.can_dictate
    ? "快捷键语音输入已就绪"
    : readiness?.can_process
      ? "录音和模型已就绪，请检查全局快捷键"
      : "存在阻断项，请按下方提示完善配置";

  return (
    <Card>
      <CardHeader className="gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div>
          <CardTitle>语音输入就绪检查</CardTitle>
          <CardDescription className="mt-2">
            {error ? `检查失败：${error}` : summary}
          </CardDescription>
        </div>
        <Button
          type="button"
          variant="outline"
          size="sm"
          onClick={() => void refresh(true)}
          disabled={isLoading}
        >
          {isLoading ? (
            <Loader2Icon className="size-4 animate-spin" aria-hidden="true" />
          ) : (
            <RefreshCwIcon className="size-4" aria-hidden="true" />
          )}
          重新检查
        </Button>
      </CardHeader>
      <CardContent>
        {readiness ? (
          <div className="grid gap-3 sm:grid-cols-2">
            {CHECK_LABELS.map(({ key, label }) => {
              const check = readiness[key];
              return (
                <div key={key} className="flex gap-3 rounded-lg border p-3">
                  <StatusIcon check={check} />
                  <div className="min-w-0 flex-1">
                    <p className="text-sm font-medium">{label}</p>
                    <p className="mt-1 text-xs leading-5 text-muted-foreground">
                      {check.detail}
                    </p>
                    {check.actions.length > 0 && (
                      <div className="mt-2 flex flex-wrap gap-2">
                        {check.actions.map((action) => (
                          <Button
                            key={action}
                            type="button"
                            variant="outline"
                            size="sm"
                            className="h-7 px-2 text-xs"
                            disabled={pendingAction !== null}
                            onClick={() => void runAction(action)}
                          >
                            {pendingAction === action ? (
                              <Loader2Icon className="size-3.5 animate-spin" aria-hidden="true" />
                            ) : action.startsWith("request_") ? (
                              <ShieldCheckIcon className="size-3.5" aria-hidden="true" />
                            ) : (
                              <ExternalLinkIcon className="size-3.5" aria-hidden="true" />
                            )}
                            {ACTION_LABELS[action]}
                          </Button>
                        ))}
                      </div>
                    )}
                  </div>
                </div>
              );
            })}
          </div>
        ) : (
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <Loader2Icon className="size-4 animate-spin" aria-hidden="true" />
            正在检查语音输入环境...
          </div>
        )}
        <p className="mt-4 text-xs leading-5 text-muted-foreground">
          自动粘贴属于非阻断能力。即使不可用，识别结果仍会保存到历史并复制到剪贴板。
        </p>
      </CardContent>
    </Card>
  );
}
