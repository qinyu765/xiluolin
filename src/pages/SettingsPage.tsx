import { useState } from "react";

import type { ConfigSaveState } from "@/app/controllers/config-save-queue";
import { ConfigSaveStatus } from "@/components/settings/ConfigSaveStatus";
import { FnHoldSetting } from "@/components/settings/FnHoldSetting";
import { InputReadinessCard } from "@/components/settings/InputReadinessCard";
import { ModelSettings } from "@/components/settings/ModelSettings";
import { RecordingStorageCard } from "@/components/settings/RecordingStorageCard";
import { SettingsFieldList } from "@/components/settings/SettingsFieldList";
import {
  settingsSchema,
  type SettingsSaveMode,
} from "@/components/settings/settings-schema";
import { usePreservedTabScroll } from "@/components/settings/usePreservedTabScroll";
import { Label } from "@/components/ui/label";
import { ShortcutInput } from "@/components/ui/shortcut-input";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import type { AppConfig, AudioDevice } from "@/types";

type SettingsPageProps = {
  appConfig: AppConfig | null;
  audioDevices: AudioDevice[];
  saveState: ConfigSaveState;
  onConfigChange: (
    patch: Partial<AppConfig>,
    saveMode: SettingsSaveMode,
  ) => void;
  onConfigSync: (patch: Partial<AppConfig>) => void;
  onConfigBlur: () => void;
  onRetryConfigSave: () => void;
  configRevision: number;
  historyRevision: number;
};

export function SettingsPage({
  appConfig,
  audioDevices,
  saveState,
  onConfigChange,
  onConfigSync,
  onConfigBlur,
  onRetryConfigSave,
  configRevision,
  historyRevision,
}: SettingsPageProps) {
  const { activeTab, rootRef, onTabChange } = usePreservedTabScroll("general");
  const [modelRevision, setModelRevision] = useState(0);

  const renderSlot = (slot: string, saveMode: SettingsSaveMode) => {
    if (slot === "longpress-shortcut") {
      return (
        <div className="flex min-h-16 flex-col items-stretch gap-3 border-b border-border/70 py-3 sm:flex-row sm:items-center sm:justify-between sm:gap-4">
          <div className="min-w-0 space-y-1">
            <Label
              htmlFor="longpress-shortcut"
              className="text-sm font-medium leading-5"
            >
              长按模式
            </Label>
            <p className="text-sm leading-5 text-muted-foreground">
              按住说话，松开停止 · 默认 Ctrl+Shift+R
            </p>
          </div>
          <div className="w-full sm:w-64">
            <ShortcutInput
              value={appConfig?.longpress_shortcut ?? ""}
              defaultValue="CommandOrControl+Shift+R"
              onChange={(value) =>
                onConfigChange({ longpress_shortcut: value }, saveMode)
              }
              placeholder="点击后按下快捷键"
            />
          </div>
        </div>
      );
    }
    if (slot === "toggle-shortcut") {
      return (
        <div className="flex min-h-16 flex-col items-stretch gap-3 border-b border-border/70 py-3 sm:flex-row sm:items-center sm:justify-between sm:gap-4">
          <div className="min-w-0 space-y-1">
            <Label
              htmlFor="toggle-shortcut"
              className="text-sm font-medium leading-5"
            >
              免按模式
            </Label>
            <p className="text-sm leading-5 text-muted-foreground">
              按一次开始，再按一次结束 · 默认 Alt+空格
            </p>
          </div>
          <div className="w-full sm:w-64">
            <ShortcutInput
              value={appConfig?.toggle_shortcut ?? ""}
              defaultValue="Alt+Space"
              onChange={(value) =>
                onConfigChange({ toggle_shortcut: value }, saveMode)
              }
              placeholder="点击后按下快捷键"
            />
          </div>
        </div>
      );
    }
    return (
      <FnHoldSetting
        enabled={appConfig?.fn_hold_enabled ?? false}
        onCheckedChange={(checked) =>
          onConfigChange({ fn_hold_enabled: checked }, saveMode)
        }
      />
    );
  };

  return (
    <div ref={rootRef} className="mx-auto max-w-4xl space-y-6">
      <div className="flex justify-end">
        <ConfigSaveStatus state={saveState} onRetry={onRetryConfigSave} />
      </div>

      <InputReadinessCard refreshRevision={configRevision + modelRevision} />

      <Tabs value={activeTab} onValueChange={onTabChange} className="space-y-6">
        <TabsList className="h-9 w-fit gap-0.5 rounded-lg border bg-muted/55 p-0.5">
          <TabsTrigger
            value="general"
            className="h-8 min-w-16 rounded-md border-transparent px-3 text-sm text-muted-foreground hover:bg-background/70 hover:text-foreground data-[state=active]:bg-primary data-[state=active]:text-primary-foreground data-[state=active]:shadow-sm"
          >
            通用
          </TabsTrigger>
          <TabsTrigger
            value="models"
            className="h-8 min-w-16 rounded-md border-transparent px-3 text-sm text-muted-foreground hover:bg-background/70 hover:text-foreground data-[state=active]:bg-primary data-[state=active]:text-primary-foreground data-[state=active]:shadow-sm"
          >
            模型配置
          </TabsTrigger>
        </TabsList>

        <TabsContent value="general" className="space-y-6">
          {appConfig
            ? settingsSchema.general.map((section) => (
                <SettingsFieldList
                  key={section.id}
                  section={section}
                  config={appConfig}
                  context={{ audioDevices }}
                  onChange={onConfigChange}
                  onBlur={onConfigBlur}
                  renderSlot={renderSlot}
                />
              ))
            : null}
          <RecordingStorageCard
            key={historyRevision}
            appConfig={appConfig}
            onConfigChange={onConfigChange}
          />
        </TabsContent>

        <TabsContent value="models" className="space-y-5">
          {settingsSchema.models.map((section) =>
            section.fields.map((field) =>
              field.control === "slot" && field.slot === "provider-catalog" ? (
                <ModelSettings
                  key={field.id}
                  appConfig={appConfig}
                  updateConfig={onConfigChange}
                  onConfigSync={onConfigSync}
                  onConfigBlur={onConfigBlur}
                  onModelChanged={() => setModelRevision((value) => value + 1)}
                />
              ) : null,
            ),
          )}
        </TabsContent>
      </Tabs>
    </div>
  );
}
