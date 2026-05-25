import React, { useState } from "react";
import { Loader2Icon, SaveIcon } from "lucide-react";
import { toast } from "sonner";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
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
import type { AppConfig, AudioDevice } from "@/types";

type SettingsPageProps = {
  appConfig: AppConfig | null;
  audioDevices: AudioDevice[];
  asrStatus: string;
  openaiStatus: string;
  isAsrSaving: boolean;
  isOpenaiSaving: boolean;
  onSaveAsrConfig: (event: React.FormEvent<HTMLFormElement>) => void;
  onSaveOpenaiConfig: (event: React.FormEvent<HTMLFormElement>) => void;
  onConfigChange: (config: AppConfig) => void;
  onConfigSaved: (config: AppConfig) => void;
};

export function SettingsPage({
  appConfig,
  audioDevices,
  asrStatus,
  openaiStatus,
  isAsrSaving,
  isOpenaiSaving,
  onSaveAsrConfig,
  onSaveOpenaiConfig,
  onConfigChange,
  onConfigSaved,
}: SettingsPageProps) {
  const [activeTab, setActiveTab] = useState("general");
  const [isGeneralSaving, setIsGeneralSaving] = useState(false);

  const handleGeneralSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!appConfig) return;

    const nextConfig = {
      ...appConfig,
      longpress_shortcut: appConfig.longpress_shortcut.trim(),
      toggle_shortcut: appConfig.toggle_shortcut.trim(),
    };

    if (!nextConfig.longpress_shortcut) {
      toast.error("长按模式快捷键不能为空");
      return;
    }

    if (!nextConfig.toggle_shortcut) {
      toast.error("切换模式快捷键不能为空");
      return;
    }

    setIsGeneralSaving(true);
    invoke<AppConfig>("update_app_config", { config: nextConfig })
      .then((savedConfig) => {
        onConfigSaved(savedConfig);
        toast.success("通用设置已保存");
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

      <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-6">
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
                  <Label htmlFor="longpress-shortcut">
                    长按模式快捷键 <span className="text-destructive">*</span>
                  </Label>
                  <Input
                    id="longpress-shortcut"
                    value={appConfig?.longpress_shortcut ?? ""}
                    onChange={(event) =>
                      onConfigChange(
                        appConfig
                          ? { ...appConfig, longpress_shortcut: event.target.value }
                          : appConfig!,
                      )
                    }
                    placeholder="例如：RightControl"
                    required
                  />
                  <p className="text-xs text-muted-foreground">
                    按住快捷键录音，松开停止。默认：右Ctrl
                  </p>
                </div>

                <div className="grid gap-2">
                  <Label htmlFor="toggle-shortcut">
                    切换模式快捷键 <span className="text-destructive">*</span>
                  </Label>
                  <Input
                    id="toggle-shortcut"
                    value={appConfig?.toggle_shortcut ?? ""}
                    onChange={(event) =>
                      onConfigChange(
                        appConfig
                          ? { ...appConfig, toggle_shortcut: event.target.value }
                          : appConfig!,
                      )
                    }
                    placeholder="例如：Alt+Space"
                    required
                  />
                  <p className="text-xs text-muted-foreground">
                    按一次开始录音，再按一次停止。默认：左Alt+Space
                  </p>
                </div>

                <div className="grid gap-2">
                  <Label htmlFor="app-recording-mode">录音模式</Label>
                  <Select
                    value={appConfig?.recording_mode ?? "toggle"}
                    onValueChange={(value) =>
                      onConfigChange(
                        appConfig
                          ? { ...appConfig, recording_mode: value }
                          : appConfig!,
                      )
                    }
                  >
                    <SelectTrigger id="app-recording-mode" className="h-10">
                      <SelectValue placeholder="选择录音模式" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="long_press">长按录音</SelectItem>
                      <SelectItem value="toggle">切换式录音</SelectItem>
                    </SelectContent>
                  </Select>
                  <p className="text-xs text-muted-foreground">
                    长按：按住快捷键录音，松开停止。切换：按一次开始，再按一次停止。
                  </p>
                </div>

                <div className="grid gap-2">
                  <Label htmlFor="app-microphone">麦克风设备</Label>
                  <Select
                    value={appConfig?.selected_microphone || "__default__"}
                    onValueChange={(value) =>
                      onConfigChange(
                        appConfig
                          ? { ...appConfig, selected_microphone: value === "__default__" ? "" : value }
                          : appConfig!,
                      )
                    }
                  >
                    <SelectTrigger id="app-microphone" className="h-10">
                      <SelectValue placeholder="使用默认麦克风" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="__default__">使用默认麦克风</SelectItem>
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
                      onConfigChange(
                        appConfig
                          ? { ...appConfig, mute_system_audio: checked }
                          : appConfig!,
                      )
                    }
                  />
                </div>

                <div className="grid gap-2">
                  <Label htmlFor="app-output-mode">输出方式</Label>
                  <Select
                    value={appConfig?.output_mode ?? "copy"}
                    onValueChange={(value) =>
                      onConfigChange(
                        appConfig
                          ? { ...appConfig, output_mode: value }
                          : appConfig!,
                      )
                    }
                  >
                    <SelectTrigger id="app-output-mode" className="h-10">
                      <SelectValue placeholder="选择输出方式" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="copy">复制到剪贴板</SelectItem>
                      <SelectItem value="paste">自动粘贴</SelectItem>
                    </SelectContent>
                  </Select>
                  <p className="text-xs text-muted-foreground">
                    复制：结果复制到剪贴板。自动粘贴：尝试模拟粘贴到当前输入位置。
                  </p>
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
                      onConfigChange(
                        appConfig
                          ? { ...appConfig, auto_save_history: checked }
                          : appConfig!,
                      )
                    }
                  />
                </div>

                <div className="flex justify-end border-t pt-4">
                  <Button type="submit" size="sm" disabled={!appConfig || isGeneralSaving}>
                    {isGeneralSaving ? (
                      <Loader2Icon className="size-4 animate-spin" aria-hidden="true" />
                    ) : (
                      <SaveIcon className="size-4" aria-hidden="true" />
                    )}
                    保存通用设置
                  </Button>
                </div>
              </form>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="models" className="space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>语音识别服务</CardTitle>
              <CardDescription>
                配置智谱 GLM-ASR-2512，用于把短音频转换为原始识别文本
              </CardDescription>
            </CardHeader>

            <CardContent>
              <form className="grid gap-4" onSubmit={onSaveAsrConfig}>
                <div className="grid gap-2">
                  <Label htmlFor="asr-api-key">
                    智谱 API Key <span className="text-destructive">*</span>
                  </Label>
                  <Input
                    id="asr-api-key"
                    type="password"
                    value={appConfig?.asr_api_key ?? ""}
                    onChange={(event) =>
                      onConfigChange(
                        appConfig
                          ? { ...appConfig, asr_api_key: event.target.value }
                          : appConfig!,
                      )
                    }
                    placeholder="本地保存，不写入仓库"
                    autoComplete="off"
                    required
                  />
                </div>

                <div className="grid gap-4 sm:grid-cols-[1fr_180px]">
                  <div className="grid gap-2">
                    <Label htmlFor="asr-base-url">Base URL</Label>
                    <Input
                      id="asr-base-url"
                      value={appConfig?.asr_base_url ?? ""}
                      onChange={(event) =>
                        onConfigChange(
                          appConfig
                            ? { ...appConfig, asr_base_url: event.target.value }
                            : appConfig!,
                        )
                      }
                    />
                  </div>
                  <div className="grid gap-2">
                    <Label htmlFor="asr-model">模型</Label>
                    <Input
                      id="asr-model"
                      value={appConfig?.asr_model ?? ""}
                      onChange={(event) =>
                        onConfigChange(
                          appConfig
                            ? { ...appConfig, asr_model: event.target.value }
                            : appConfig!,
                        )
                      }
                    />
                  </div>
                </div>

                <div className="flex flex-col gap-3 border-t pt-4 sm:flex-row sm:items-center sm:justify-between">
                  <p className="text-sm leading-6 text-muted-foreground">
                    {asrStatus}
                  </p>
                  <Button type="submit" size="sm" disabled={!appConfig || isAsrSaving}>
                    {isAsrSaving ? (
                      <Loader2Icon className="size-4 animate-spin" aria-hidden="true" />
                    ) : (
                      <SaveIcon className="size-4" aria-hidden="true" />
                    )}
                    保存 ASR 配置
                  </Button>
                </div>
              </form>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>文本整理服务</CardTitle>
              <CardDescription>
                配置 OpenAI Responses API，用于把原始识别文本整理成可直接使用的结果
              </CardDescription>
            </CardHeader>

            <CardContent>
              <form className="grid gap-4" onSubmit={onSaveOpenaiConfig}>
                <div className="grid gap-2">
                  <Label htmlFor="openai-api-key">
                    OpenAI API Key <span className="text-destructive">*</span>
                  </Label>
                  <Input
                    id="openai-api-key"
                    type="password"
                    value={appConfig?.openai_api_key ?? ""}
                    onChange={(event) =>
                      onConfigChange(
                        appConfig
                          ? { ...appConfig, openai_api_key: event.target.value }
                          : appConfig!,
                      )
                    }
                    placeholder="本地保存，不写入仓库"
                    autoComplete="off"
                    required
                  />
                </div>

                <div className="grid gap-4 sm:grid-cols-[1fr_180px]">
                  <div className="grid gap-2">
                    <Label htmlFor="openai-base-url">Base URL</Label>
                    <Input
                      id="openai-base-url"
                      value={appConfig?.openai_base_url ?? ""}
                      onChange={(event) =>
                        onConfigChange(
                          appConfig
                            ? { ...appConfig, openai_base_url: event.target.value }
                            : appConfig!,
                        )
                      }
                    />
                  </div>
                  <div className="grid gap-2">
                    <Label htmlFor="openai-model">模型</Label>
                    <Input
                      id="openai-model"
                      value={appConfig?.openai_model ?? ""}
                      onChange={(event) =>
                        onConfigChange(
                          appConfig
                            ? { ...appConfig, openai_model: event.target.value }
                            : appConfig!,
                        )
                      }
                    />
                  </div>
                </div>

                <div className="flex flex-col gap-3 border-t pt-4 sm:flex-row sm:items-center sm:justify-between">
                  <p className="text-sm leading-6 text-muted-foreground">
                    {openaiStatus}
                  </p>
                  <Button
                    type="submit"
                    size="sm"
                    disabled={!appConfig || isOpenaiSaving}
                  >
                    {isOpenaiSaving ? (
                      <Loader2Icon className="size-4 animate-spin" aria-hidden="true" />
                    ) : (
                      <SaveIcon className="size-4" aria-hidden="true" />
                    )}
                    保存 OpenAI 配置
                  </Button>
                </div>
              </form>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
