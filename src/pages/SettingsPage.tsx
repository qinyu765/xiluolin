import React, { useState } from "react";
import { Loader2Icon, SaveIcon } from "lucide-react";
import { toast } from "sonner";

import { FnHoldSetting } from "@/components/settings/FnHoldSetting";
import { InputReadinessCard } from "@/components/settings/InputReadinessCard";
import { ModelSettings } from "@/components/settings/ModelSettings";
import { RecordingStorageCard } from "@/components/settings/RecordingStorageCard";
import { SettingsFieldList } from "@/components/settings/SettingsFieldList";
import { settingsSchema } from "@/components/settings/settings-schema";
import { Button } from "@/components/ui/button";
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

type SettingsPageProps = {
  appConfig: AppConfig | null;
  audioDevices: AudioDevice[];
  asrStatus: string;
  textProcessingStatus: string;
  isAsrSaving: boolean;
  isTextProcessingSaving: boolean;
  onSaveAsrConfig: (event: React.FormEvent<HTMLFormElement>) => void;
  onSaveTextProcessingConfig: (event: React.FormEvent<HTMLFormElement>) => void;
  onConfigChange: (config: AppConfig) => void;
  onSaveConfig: (config: AppConfig) => Promise<AppConfig>;
  configRevision: number;
  historyRevision: number;
};

export function SettingsPage({
  appConfig,
  audioDevices,
  asrStatus,
  textProcessingStatus,
  isAsrSaving,
  isTextProcessingSaving,
  onSaveAsrConfig,
  onSaveTextProcessingConfig,
  onConfigChange,
  onSaveConfig,
  configRevision,
  historyRevision,
}: SettingsPageProps) {
  const [activeTab, setActiveTab] = useState("general");
  const [isGeneralSaving, setIsGeneralSaving] = useState(false);
  const [modelRevision, setModelRevision] = useState(0);

  const updateConfig = (patch: Partial<AppConfig>) => {
    if (appConfig) onConfigChange({ ...appConfig, ...patch });
  };

  const handleGeneralSubmit = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!appConfig) return;

    const nextConfig = {
      ...appConfig,
      longpress_shortcut: appConfig.longpress_shortcut.trim(),
      toggle_shortcut: appConfig.toggle_shortcut.trim(),
    };

    setIsGeneralSaving(true);
    onSaveConfig(nextConfig)
      .then(() => toast.success("通用设置已保存"))
      .catch((error) => toast.error(`保存通用设置失败：${String(error)}`))
      .finally(() => setIsGeneralSaving(false));
  };

  const renderGeneralSlot = (slot: string) => {
    if (slot === "longpress-shortcut") {
      return (
        <div className="grid gap-2">
          <Label htmlFor="longpress-shortcut">长按模式快捷键</Label>
          <ShortcutInput
            value={appConfig?.longpress_shortcut ?? ""}
            defaultValue="CommandOrControl+Shift+R"
            onChange={(value) => updateConfig({ longpress_shortcut: value })}
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
            onChange={(value) => updateConfig({ toggle_shortcut: value })}
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
            updateConfig({ fn_hold_enabled: checked })
          }
        />
      );
    }
    return null;
  };

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-3xl font-bold">设置</h1>
        <p className="mt-2 text-sm text-muted-foreground">
          管理应用配置和模型服务
        </p>
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
                  <form className="grid gap-4" onSubmit={handleGeneralSubmit}>
                    {appConfig ? (
                      <SettingsFieldList
                        section={section}
                        config={appConfig}
                        context={{ audioDevices }}
                        onChange={updateConfig}
                        onBlur={() => undefined}
                        renderSlot={renderGeneralSlot}
                      />
                    ) : null}
                    <div className="flex justify-end border-t pt-4">
                      <Button
                        type="submit"
                        size="sm"
                        disabled={!appConfig || isGeneralSaving}
                      >
                        {isGeneralSaving ? (
                          <Loader2Icon
                            className="size-4 animate-spin"
                            aria-hidden="true"
                          />
                        ) : (
                          <SaveIcon className="size-4" aria-hidden="true" />
                        )}
                        保存通用设置
                      </Button>
                    </div>
                  </form>
                </CardContent>
              </Card>
            ),
          )}
        </TabsContent>

        <TabsContent value="models" className="space-y-6">
          <ModelSettings
            appConfig={appConfig}
            asrStatus={asrStatus}
            textProcessingStatus={textProcessingStatus}
            isAsrSaving={isAsrSaving}
            isTextProcessingSaving={isTextProcessingSaving}
            onSaveAsrConfig={onSaveAsrConfig}
            onSaveTextProcessingConfig={onSaveTextProcessingConfig}
            updateConfig={updateConfig}
            onModelChanged={() => setModelRevision((value) => value + 1)}
          />
        </TabsContent>
      </Tabs>
    </div>
  );
}
