import { useCallback } from "react";
import {
  CheckCircle2Icon,
  CircleAlertIcon,
  Mic2Icon,
  RadioTowerIcon,
  SparklesIcon,
  ZapIcon,
} from "lucide-react";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { commands } from "@/generated/tauri-bindings";
import { useResource } from "@/shared/resource/useResource";
import type { AppConfig, Persona } from "@/types";
import { formatShortcutDisplay } from "@/utils/shortcut";

type ReadinessData = Awaited<ReturnType<typeof commands.readInputReadiness>> & {
  realtime: Awaited<ReturnType<typeof commands.realtimeAsrModelInfo>>;
  catalog: Awaited<ReturnType<typeof commands.listProviderCatalog>>;
};

export function HomeReadinessCard({
  appConfig,
  persona,
}: {
  appConfig: AppConfig | null;
  persona: Persona | undefined;
}) {
  const load = useCallback(async (): Promise<ReadinessData> => {
    const [readiness, realtime, catalog] = await Promise.all([
      commands.readInputReadiness(),
      commands.realtimeAsrModelInfo(),
      commands.listProviderCatalog(),
    ]);
    return { ...readiness, realtime, catalog };
  }, []);
  const resource = useResource(load);
  const shortcut = appConfig?.longpress_shortcut || appConfig?.toggle_shortcut;
  const providerId = appConfig?.asr.primary;
  const provider =
    resource.data?.catalog.asr.find(({ id }) => id === providerId)?.name ||
    providerId ||
    "未配置";
  const items = [
    {
      icon: Mic2Icon,
      label: "麦克风",
      value: appConfig?.selected_microphone || "系统默认",
      ready: resource.data?.microphone.ready,
    },
    {
      icon: SparklesIcon,
      label: "最终识别",
      value: provider,
      ready: resource.data?.asr.ready,
    },
    {
      icon: RadioTowerIcon,
      label: "实时预览",
      value: resource.data?.realtime.enabled
        ? "Zipformer 已启用"
        : resource.data?.realtime.state === "ready"
          ? "已安装，未启用"
          : "未安装",
      ready: resource.data?.realtime.enabled,
    },
    {
      icon: ZapIcon,
      label: "快捷键",
      value: shortcut ? formatShortcutDisplay(shortcut) : "未配置",
      ready: resource.data?.hotkey.ready,
    },
    {
      icon: SparklesIcon,
      label: "当前人格",
      value: persona?.name || "未选择",
      ready: Boolean(persona),
    },
  ];

  return (
    <Card className="overflow-hidden border-primary/15 bg-[linear-gradient(135deg,var(--card),color-mix(in_srgb,var(--primary)_6%,var(--card)))]">
      <CardHeader className="gap-2 pb-0">
        <div className="flex items-center justify-between gap-4">
          <CardTitle className="text-lg">运行就绪</CardTitle>
          <span className="inline-flex items-center gap-1.5 rounded-full border bg-background/70 px-2.5 py-1 text-xs font-medium">
            {resource.data?.can_dictate ? (
              <CheckCircle2Icon className="size-3.5 text-emerald-600" />
            ) : (
              <CircleAlertIcon className="size-3.5 text-amber-500" />
            )}
            {resource.loading
              ? "检查中"
              : resource.data?.can_dictate
                ? "可开始口述"
                : "需要检查配置"}
          </span>
        </div>
        <div className="h-1 overflow-hidden rounded-full bg-muted">
          <div className="h-full w-2/3 rounded-full bg-primary shadow-[0_0_16px_color-mix(in_srgb,var(--primary)_55%,transparent)]" />
        </div>
      </CardHeader>
      <CardContent className="grid gap-2 sm:grid-cols-2 md:grid-cols-5">
        {items.map(({ icon: Icon, label, value, ready }) => (
          <div
            key={label}
            className="min-w-0 rounded-md border bg-background/55 p-3"
          >
            <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
              <Icon className="size-3.5" aria-hidden="true" />
              {label}
            </div>
            <p className="mt-1.5 truncate text-sm font-medium" title={value}>
              {value}
            </p>
            <span
              className={
                ready
                  ? "text-[10px] text-emerald-600"
                  : "text-[10px] text-muted-foreground"
              }
            >
              {ready ? "就绪" : "待确认"}
            </span>
          </div>
        ))}
      </CardContent>
    </Card>
  );
}
