import {
  AlertCircleIcon,
  CheckIcon,
  Clock3Icon,
  Loader2Icon,
} from "lucide-react";

import type { ConfigSaveState } from "@/app/controllers/config-save-queue";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

export function ConfigSaveStatus({
  state,
  onRetry,
}: {
  state: ConfigSaveState;
  onRetry: () => void;
}) {
  if (state.status === "idle") return null;

  const isError = state.status === "error";
  const isInvalid = state.status === "invalid";
  let label: string;
  if (state.status === "pending") label = "等待保存";
  else if (state.status === "saving") label = "正在保存…";
  else if (state.status === "saved") label = "已保存";
  else if (state.status === "invalid") label = `待补全：${state.message}`;
  else label = `保存失败：${state.message}`;

  return (
    <div
      role="status"
      aria-live="polite"
      className={cn(
        "flex min-h-8 min-w-0 items-center gap-2 text-sm text-muted-foreground",
        (isError || isInvalid) && "text-destructive",
      )}
    >
      {state.status === "saving" ? (
        <Loader2Icon className="size-4 animate-spin" aria-hidden="true" />
      ) : state.status === "saved" ? (
        <CheckIcon className="size-4" aria-hidden="true" />
      ) : state.status === "pending" ? (
        <Clock3Icon className="size-4" aria-hidden="true" />
      ) : (
        <AlertCircleIcon className="size-4 shrink-0" aria-hidden="true" />
      )}
      <span className="min-w-0 truncate">{label}</span>
      {isError ? (
        <Button
          type="button"
          variant="outline"
          size="sm"
          className="ml-1 h-7 px-2"
          onClick={onRetry}
        >
          重试
        </Button>
      ) : null}
    </div>
  );
}
