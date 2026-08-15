import { useEffect, useState } from "react";
import { FileTextIcon, Mic2Icon } from "lucide-react";

import type { SettingsSaveMode } from "@/components/settings/settings-schema";
import { RealtimePreviewModelCard } from "@/features/settings/RealtimePreviewModelCard";
import { commands, type ProviderCatalog } from "@/generated/tauri-bindings";
import type { AppConfig } from "@/types";

import {
  ProviderCapabilityCard,
  type ProviderCapability,
} from "./ProviderCapabilityCard";

type ModelSettingsProps = {
  appConfig: AppConfig | null;
  updateConfig: (patch: Partial<AppConfig>, saveMode: SettingsSaveMode) => void;
  onConfigBlur: () => void;
  onModelChanged: () => void;
  onConfigSync?: (patch: Partial<AppConfig>) => void;
};

export function ModelSettings(props: ModelSettingsProps) {
  const [catalog, setCatalog] = useState<ProviderCatalog | null>(null);
  const [catalogError, setCatalogError] = useState("");

  useEffect(() => {
    let active = true;
    void commands
      .listProviderCatalog()
      .then((value) => {
        if (active) setCatalog(value);
      })
      .catch((error) => {
        if (active) setCatalogError(`Provider 列表读取失败：${String(error)}`);
      });
    return () => {
      active = false;
    };
  }, []);

  if (catalogError) {
    return <p className="text-sm text-destructive">{catalogError}</p>;
  }
  if (!catalog || !props.appConfig) {
    return (
      <div className="h-24 motion-safe:animate-pulse rounded-xl border bg-card" />
    );
  }

  const sections: Array<{
    capability: ProviderCapability;
    title: string;
    hint: string;
    icon: typeof Mic2Icon;
    descriptors: ProviderCatalog["asr"];
    routing: typeof props.appConfig.asr;
  }> = [
    {
      capability: "asr",
      title: "语音识别服务",
      hint: "把录音转换成文字，按调用链依次尝试。",
      icon: Mic2Icon,
      descriptors: catalog.asr,
      routing: props.appConfig.asr,
    },
    {
      capability: "text",
      title: "文本整理服务",
      hint: "对识别原文进行整理，失败时保留原文。",
      icon: FileTextIcon,
      descriptors: catalog.text,
      routing: props.appConfig.text,
    },
  ];

  return (
    <div className="space-y-6">
      <RealtimePreviewModelCard
        onChanged={props.onModelChanged}
        onConfigSync={props.onConfigSync}
      />

      <div className="grid gap-4 lg:grid-cols-2">
        {sections.map((section) => (
          <ProviderCapabilityCard
            key={section.capability}
            capability={section.capability}
            title={section.title}
            hint={section.hint}
            icon={section.icon}
            descriptors={section.descriptors}
            routing={section.routing}
            onChange={(routing, saveMode) =>
              props.updateConfig(
                section.capability === "asr"
                  ? { asr: routing }
                  : { text: routing },
                saveMode,
              )
            }
            onConfigBlur={props.onConfigBlur}
            onModelChanged={props.onModelChanged}
          />
        ))}
      </div>
    </div>
  );
}
