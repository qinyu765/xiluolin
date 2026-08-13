import { useState } from "react";

import type { ConfigSaveState } from "@/app/controllers/config-save-queue";
import { ConfigSaveStatus } from "@/components/settings/ConfigSaveStatus";
import { FnHoldSetting } from "@/components/settings/FnHoldSetting";
import { InputReadinessCard } from "@/components/settings/InputReadinessCard";
import { ModelSettings } from "@/components/settings/ModelSettings";
import { RecordingStorageCard } from "@/components/settings/RecordingStorageCard";
import { SettingsFieldList } from "@/components/settings/SettingsFieldList";
import { settingsSchema } from "@/components/settings/settings-schema";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { ShortcutInput } from "@/components/ui/shortcut-input";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import type { AppConfig, AudioDevice } from "@/types";
import type { SettingsSaveMode } from "@/components/settings/settings-schema";

type SettingsPageProps = {
  appConfig: AppConfig | null;
  audioDevices: AudioDevice[];
  saveState: ConfigSaveState;
  onConfigChange: (
    patch: Partial<AppConfig>,
    saveMode: SettingsSaveMode,
  ) => void;
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
  onConfigBlur,
  onRetryConfigSave,
  configRevision,
  historyRevision,
}: SettingsPageProps) {
  const [activeTab, setActiveTab] = useState("general");
  const [modelRevision, setModelRevision] = useState(0);

  const renderGeneralSlot = (slot: string, saveMode: SettingsSaveMode) => {
    if (slot === "longpress-shortcut") {
      return (
        <div className="grid gap-2">
          <Label htmlFor="longpress-shortcut">长按模式快捷键</Label>
          <ShortcutInput
            value={appConfig?.longpress_shortcut ?? ""}
            defaultValue="CommandOrControl+Shift+R"
            onChange={(value) =>
              onConfigChange({ longpress_shortcut: value }, saveMode)
            }
            placeholder="点击后按下快捷键"
          />
          <p className="text-xs text-muted-foreground">
            按住快捷键录音，松开停止。默认：Ctrl+Shift+R
          </p>
        </div>
      );
    }
    if (slot === "toggle-shortcut") {
      return (
        <div className="grid gap-2">
          <Label htmlFor="toggle-shortcut">切换模式快捷键</Label>
          <ShortcutInput
            value={appConfig?.toggle_shortcut ?? ""}
            defaultValue="Alt+Space"
            onChange={(value) =>
              onConfigChange({ toggle_shortcut: value }, saveMode)
            }
            placeholder="点击后按下快捷键"
          />
          <p className="text-xs text-muted-foreground">
            按一次开始录音，再按一次停止。默认：Alt+空格
          </p>
        </div>
      );
    }
    if (slot === "fn-hold") {
      return (
        <FnHoldSetting
          enabled={appConfig?.fn_hold_enabled ?? false}
          onCheckedChange={(checked) =>
            onConfigChange({ fn_hold_enabled: checked }, saveMode)
          }
        />
      );
    }
    return null;
  };

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <h1 className="text-3xl font-bold">设置</h1>
          <p className="mt-2 text-sm text-muted-foreground">
            管理应用配置和模型服务，修改会自动保存
          </p>
        </div>
        <ConfigSaveStatus state={saveState} onRetry={onRetryConfigSave} />
      </div>

      <InputReadinessCard refreshRevision={configRevision + modelRevision} />

      <Tabs
        value={activeTab}
        onValueChange={setActiveTab}
        className="space-y-6"
      >
        <TabsList className="grid w-full grid-cols-2">
          <TabsTrigger value="general">通用</TabsTrigger>
          <TabsTrigger value="models">模型配置</TabsTrigger>
        </TabsList>

        <TabsContent value="general" className="space-y-6">
          {settingsSchema.general.map((section) =>
            section.type === "slot" ? (
              <RecordingStorageCard
                key={section.id}
                revision={historyRevision}
              />
            ) : (
              <Card key={section.id}>
                <CardHeader>
                  <CardTitle>{section.title}</CardTitle>
                  <CardDescription>{section.description}</CardDescription>
                </CardHeader>
                <CardContent>
                  <div className="grid gap-4">
                    {appConfig ? (
                      <SettingsFieldList
                        section={section}
                        config={appConfig}
                        context={{ audioDevices }}
                        onChange={onConfigChange}
                        onBlur={onConfigBlur}
                        renderSlot={renderGeneralSlot}
                      />
                    ) : null}
                  </div>
                </CardContent>
              </Card>
            ),
          )}
        </TabsContent>

        <TabsContent value="models" className="space-y-6">
          <ModelSettings
            appConfig={appConfig}
            updateConfig={onConfigChange}
            onConfigBlur={onConfigBlur}
            onModelChanged={() => setModelRevision((value) => value + 1)}
          />
        </TabsContent>
      </Tabs>
    </div>
  );
}
