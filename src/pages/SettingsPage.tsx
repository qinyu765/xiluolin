import React, { useState } from "react";
import { Loader2Icon, SaveIcon } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ShortcutInput } from "@/components/ui/shortcut-input";
import { InputReadinessCard } from "@/components/settings/InputReadinessCard";
import { ModelSettings } from "@/components/settings/ModelSettings";
import { RecordingStorageCard } from "@/components/settings/RecordingStorageCard";
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

  const handleGeneralSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!appConfig) return;

    const nextConfig = {
      ...appConfig,
      longpress_shortcut: appConfig.longpress_shortcut.trim(),
      toggle_shortcut: appConfig.toggle_shortcut.trim(),
    };

    setIsGeneralSaving(true);
    onSaveConfig(nextConfig)
      .then(() => {
        toast.success("通用设置已保存，快捷键已生效");
      })
      .catch((error) => {
        toast.error(`保存通用设置失败：${String(error)}`);
      })
      .finally(() => {
        setIsGeneralSaving(false);
      });
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
          <Card>
            <CardHeader>
              <CardTitle>通用设置</CardTitle>
              <CardDescription>
                配置快捷键、录音模式、输出方式和历史记录保存选项
              </CardDescription>
            </CardHeader>
            <CardContent>
              <form className="grid gap-4" onSubmit={handleGeneralSubmit}>
                <div className="grid gap-2">
                  <Label htmlFor="longpress-shortcut">长按模式快捷键</Label>
                  <ShortcutInput
                    value={appConfig?.longpress_shortcut ?? ""}
                    defaultValue="CommandOrControl+Shift+R"
                    onChange={(value) =>
                      updateConfig({ longpress_shortcut: value })
                    }
                    placeholder="点击后按下快捷键"
                  />
                  <p className="text-xs text-muted-foreground">
                    按住快捷键录音，松开停止。默认：Ctrl+Shift+R
                  </p>
                </div>

                <div className="grid gap-2">
                  <Label htmlFor="toggle-shortcut">切换模式快捷键</Label>
                  <ShortcutInput
                    value={appConfig?.toggle_shortcut ?? ""}
                    defaultValue="Alt+Space"
                    onChange={(value) =>
                      updateConfig({ toggle_shortcut: value })
                    }
                    placeholder="点击后按下快捷键"
                  />
                  <p className="text-xs text-muted-foreground">
                    按一次开始录音，再按一次停止。默认：Alt+空格
                  </p>
                </div>

                <div className="grid gap-2">
                  <Label htmlFor="app-microphone">麦克风设备</Label>
                  <Select
                    value={appConfig?.selected_microphone || "__default__"}
                    onValueChange={(value) =>
                      updateConfig({
                        selected_microphone:
                          value === "__default__" ? "" : value,
                      })
                    }
                  >
                    <SelectTrigger id="app-microphone" className="h-10">
                      <SelectValue placeholder="使用默认麦克风" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="__default__">
                        使用默认麦克风
                      </SelectItem>
                      {audioDevices.map((device) => (
                        <SelectItem key={device.name} value={device.name}>
                          {device.name} {device.is_default ? "(默认)" : ""}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  <p className="text-xs text-muted-foreground">
                    选择用于录音的麦克风设备。留空则使用系统默认麦克风。
                  </p>
                </div>

                <div className="flex items-center justify-between rounded-lg border p-3">
                  <div className="space-y-0.5">
                    <Label htmlFor="app-mute-audio">录音时静音其他应用</Label>
                    <p className="text-xs text-muted-foreground">
                      开启后，语音输入时会暂停系统音频播放，输入完成后自动恢复
                    </p>
                  </div>
                  <Switch
                    id="app-mute-audio"
                    checked={appConfig?.mute_system_audio ?? false}
                    onCheckedChange={(checked) =>
                      updateConfig({ mute_system_audio: checked })
                    }
                  />
                </div>

                <div className="flex items-center justify-between rounded-lg border p-3">
                  <div className="space-y-0.5">
                    <Label htmlFor="app-auto-save">自动保存历史</Label>
                    <p className="text-xs text-muted-foreground">
                      每次语音输入完成后自动保存到历史记录
                    </p>
                  </div>
                  <Switch
                    id="app-auto-save"
                    checked={appConfig?.auto_save_history ?? true}
                    onCheckedChange={(checked) =>
                      updateConfig({
                        auto_save_history: checked,
                        retain_recordings: checked
                          ? (appConfig?.retain_recordings ?? false)
                          : false,
                      })
                    }
                  />
                </div>

                <div className="flex items-center justify-between rounded-lg border p-3">
                  <div className="space-y-0.5">
                    <Label htmlFor="app-retain-recordings">保留原始录音</Label>
                    <p className="text-xs text-muted-foreground">
                      默认关闭。仅在自动保存历史成功时保留应用录制的 WAV
                    </p>
                  </div>
                  <Switch
                    id="app-retain-recordings"
                    checked={appConfig?.retain_recordings ?? false}
                    disabled={!appConfig?.auto_save_history}
                    onCheckedChange={(checked) =>
                      updateConfig({ retain_recordings: checked })
                    }
                  />
                </div>

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
          <RecordingStorageCard revision={historyRevision} />
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
