import { Loader2Icon, SaveIcon } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import type { AppConfig } from "@/types";

import { LocalAsrSettings } from "./LocalAsrSettings";
import { SettingsFieldList } from "./SettingsFieldList";
import { settingsSchema, type SettingsSaveMode } from "./settings-schema";

type ModelSettingsProps = {
  appConfig: AppConfig | null;
  asrStatus: string;
  textProcessingStatus: string;
  isAsrSaving: boolean;
  isTextProcessingSaving: boolean;
  onSaveAsrConfig: (event: React.FormEvent<HTMLFormElement>) => void;
  onSaveTextProcessingConfig: (event: React.FormEvent<HTMLFormElement>) => void;
  updateConfig: (
    patch: Partial<AppConfig>,
    saveMode?: SettingsSaveMode,
  ) => void;
  onModelChanged: () => void;
};

export function ModelSettings({
  appConfig,
  asrStatus,
  textProcessingStatus,
  isAsrSaving,
  isTextProcessingSaving,
  onSaveAsrConfig,
  onSaveTextProcessingConfig,
  updateConfig,
  onModelChanged,
}: ModelSettingsProps) {
  if (!appConfig) return null;

  return settingsSchema.models.map((section) => {
    const isAsr = section.id === "asr";
    const status = isAsr ? asrStatus : textProcessingStatus;
    const isSaving = isAsr ? isAsrSaving : isTextProcessingSaving;
    const onSubmit = isAsr ? onSaveAsrConfig : onSaveTextProcessingConfig;
    const buttonLabel = isAsr ? "保存 ASR 配置" : "保存文本处理配置";

    return (
      <Card key={section.id}>
        <CardHeader>
          <CardTitle>{section.title}</CardTitle>
          <CardDescription>{section.description}</CardDescription>
        </CardHeader>
        <CardContent>
          <form className="grid gap-4" onSubmit={onSubmit}>
            <SettingsFieldList
              section={section}
              config={appConfig}
              context={{ audioDevices: [] }}
              onChange={updateConfig}
              onBlur={() => undefined}
              renderSlot={(slot) =>
                slot === "local-asr-settings" ? (
                  <LocalAsrSettings
                    config={appConfig}
                    onChange={(config) => updateConfig(config)}
                    onModelChanged={onModelChanged}
                  />
                ) : null
              }
            />
            <div className="flex flex-col gap-3 border-t pt-4 sm:flex-row sm:items-center sm:justify-between">
              <p className="text-sm leading-6 text-muted-foreground">
                {status}
              </p>
              <Button type="submit" size="sm" disabled={isSaving}>
                {isSaving ? (
                  <Loader2Icon
                    className="size-4 animate-spin"
                    aria-hidden="true"
                  />
                ) : (
                  <SaveIcon className="size-4" aria-hidden="true" />
                )}
                {buttonLabel}
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>
    );
  });
}
