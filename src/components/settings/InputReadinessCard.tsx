import { useCallback, useEffect, useState } from "react";
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
import { Card, CardContent } from "@/components/ui/card";
import { commands } from "@/generated/tauri-bindings";
import { cn } from "@/lib/utils";
import type { InputReadiness, ReadinessAction, ReadinessCheck } from "@/types";

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
    return (
      <CheckCircle2Icon
        className="size-4 text-emerald-600"
        aria-hidden="true"
      />
    );
  }
  if (check.blocking) {
    return (
      <XCircleIcon className="size-4 text-destructive" aria-hidden="true" />
    );
  }
  return (
    <AlertTriangleIcon className="size-4 text-amber-500" aria-hidden="true" />
  );
}

function actionPermission(action: ReadinessAction) {
  return action.includes("microphone") ? "microphone" : "accessibility";
}

export function InputReadinessCard() {
  const [readiness, setReadiness] = useState<InputReadiness | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [pendingAction, setPendingAction] = useState<ReadinessAction | null>(
    null,
  );

  const refresh = useCallback(async (showLoading = false) => {
    if (showLoading) setIsLoading(true);
    try {
      const result = await commands.readInputReadiness();
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
    window.addEventListener("focus", handleRefresh);
    window.addEventListener("xiluolin:readiness-changed", handleRefresh);
    return () => {
      window.removeEventListener("focus", handleRefresh);
      window.removeEventListener("xiluolin:readiness-changed", handleRefresh);
    };
  }, [refresh]);

  const runAction = useCallback(
    async (action: ReadinessAction) => {
      setPendingAction(action);
      setError(null);
      try {
        const permission = actionPermission(action);
        if (action.startsWith("request_")) {
          await commands.requestMacosPermission(permission);
          await refresh();
        } else {
          await commands.openMacosPrivacySettings(permission);
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
    ? "语音输入已就绪"
    : readiness?.can_process
      ? "录音和模型已就绪，请检查全局快捷键"
      : "存在阻断项，请按下方提示完善配置";
  const checks = readiness
    ? CHECK_LABELS.map(({ key, label }) => ({
        key,
        label,
        check: readiness[key],
      }))
    : [];
  const problemChecks = checks.filter(({ check }) => !check.ready);

  return (
    <Card className="gap-0 overflow-hidden py-0">
      <CardContent className="p-4 sm:p-5">
        <div className="flex items-start gap-3">
          <div
            className={cn(
              "flex size-9 shrink-0 items-center justify-center rounded-xl",
              readiness?.can_dictate
                ? "bg-emerald-500/10 text-emerald-600"
                : "bg-amber-500/10 text-amber-600",
            )}
          >
            {readiness?.can_dictate ? (
              <CheckCircle2Icon className="size-5" aria-hidden="true" />
            ) : (
              <AlertTriangleIcon className="size-5" aria-hidden="true" />
            )}
          </div>
          <div className="min-w-0 flex-1">
            <p className="text-sm font-semibold">语音输入就绪检查</p>
            <p
              className={cn(
                "mt-0.5 text-xs leading-5 text-muted-foreground",
                error && "text-destructive",
              )}
            >
              {error
                ? `检查失败：${error}`
                : isLoading && !readiness
                  ? "正在检查语音输入环境…"
                  : summary}
            </p>
          </div>
          <Button
            type="button"
            variant="ghost"
            size="icon"
            className="size-8"
            aria-label="重新检查"
            onClick={() => void refresh(true)}
            disabled={isLoading}
          >
            {isLoading ? (
              <Loader2Icon className="size-4 animate-spin" aria-hidden="true" />
            ) : (
              <RefreshCwIcon className="size-4" aria-hidden="true" />
            )}
          </Button>
        </div>

        {readiness ? (
          <div className="mt-3 flex flex-wrap gap-2">
            {checks.map(({ key, label, check }) => (
              <div
                key={key}
                className={cn(
                  "inline-flex items-center gap-1.5 rounded-full border px-2.5 py-1 text-xs font-medium",
                  check.ready
                    ? "border-emerald-200 bg-emerald-50 text-emerald-700"
                    : check.blocking
                      ? "border-destructive/20 bg-destructive/5 text-destructive"
                      : "border-amber-200 bg-amber-50 text-amber-700",
                )}
              >
                <StatusIcon check={check} />
                {label}
              </div>
            ))}
          </div>
        ) : null}

        {problemChecks.length > 0 ? (
          <div className="mt-4 grid gap-3 border-t pt-4 sm:grid-cols-2">
            {problemChecks.map(({ key, label, check }) => (
              <div
                key={key}
                className="flex gap-2.5 rounded-lg bg-muted/45 p-3"
              >
                <StatusIcon check={check} />
                <div className="min-w-0 flex-1">
                  <p className="text-sm font-medium">{label}</p>
                  <p className="mt-0.5 text-xs leading-5 text-muted-foreground">
                    {check.detail}
                  </p>
                  {check.actions.length > 0 ? (
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
                            <Loader2Icon
                              className="size-3.5 animate-spin"
                              aria-hidden="true"
                            />
                          ) : action.startsWith("request_") ? (
                            <ShieldCheckIcon
                              className="size-3.5"
                              aria-hidden="true"
                            />
                          ) : (
                            <ExternalLinkIcon
                              className="size-3.5"
                              aria-hidden="true"
                            />
                          )}
                          {ACTION_LABELS[action]}
                        </Button>
                      ))}
                    </div>
                  ) : null}
                </div>
              </div>
            ))}
          </div>
        ) : null}
      </CardContent>
    </Card>
  );
}
