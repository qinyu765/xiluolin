import React from "react";
import { Loader2Icon, SaveIcon } from "lucide-react";
import { toast } from "sonner";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
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
import type { AppConfig, AudioDevice } from "@/types";

type AppSettingsDialogProps = {
  open: boolean;
  appConfig: AppConfig | null;
  audioDevices: AudioDevice[];
  onOpenChange: (open: boolean) => void;
  onConfigChange: (config: AppConfig) => void;
  onConfigSaved: (config: AppConfig) => void;
};

export function AppSettingsDialog({
  open,
  appConfig,
  audioDevices,
  onOpenChange,
  onConfigChange,
  onConfigSaved,
}: AppSettingsDialogProps) {
  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
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

    invoke<AppConfig>("update_app_config", { config: nextConfig })
      .then((savedConfig) => {
        onConfigSaved(savedConfig);
        toast.success("应用设置已保存");
        onOpenChange(false);
      })
      .catch((error) => {
        toast.error(`保存应用设置失败：${String(error)}`);
      });
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-2xl">
        <form className="grid gap-4" onSubmit={handleSubmit}>
          <DialogHeader>
            <DialogTitle>应用设置</DialogTitle>
            <DialogDescription>
              配置快捷键、录音模式、输出方式和历史记录保存选项。
            </DialogDescription>
          </DialogHeader>

          <div className="grid gap-4">
            <div className="grid gap-2">
              <Label htmlFor="longpress-shortcut">长按模式快捷键</Label>
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
                placeholder="例如：CommandOrControl+Shift+R"
              />
              <p className="text-xs text-muted-foreground">
                按住快捷键录音，松开停止。格式：CommandOrControl+Shift+R 或 Alt+Space
              </p>
            </div>

            <div className="grid gap-2">
              <Label htmlFor="toggle-shortcut">切换模式快捷键</Label>
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
              />
              <p className="text-xs text-muted-foreground">
                按一次开始录音，再按一次停止。格式：Alt+Space 或 CommandOrControl+R
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
          </div>

          <DialogFooter>
            <Button type="submit" size="sm" disabled={!appConfig}>
              <SaveIcon className="size-4" aria-hidden="true" />
              保存设置
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
