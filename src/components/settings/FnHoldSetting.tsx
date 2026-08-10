import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";

type FnHoldSettingProps = {
  enabled: boolean;
  onCheckedChange: (enabled: boolean) => void;
  isMacOS?: boolean;
};

export function isMacOSUserAgent(userAgent: string) {
  return /Macintosh|Mac OS X/i.test(userAgent);
}

export function FnHoldSetting({
  enabled,
  onCheckedChange,
  isMacOS = isMacOSUserAgent(
    typeof navigator === "undefined" ? "" : navigator.userAgent,
  ),
}: FnHoldSettingProps) {
  if (!isMacOS) return null;

  return (
    <div className="flex items-center justify-between gap-4 rounded-lg border p-3">
      <div className="space-y-0.5">
        <Label htmlFor="app-fn-hold">按住 Fn 录音</Label>
        <p className="text-xs text-muted-foreground">
          仅处理独立 Fn：按下立即录音，松开识别；短于 300 毫秒会取消。需要 macOS
          辅助功能权限，权限不足时组合快捷键仍可使用。
        </p>
      </div>
      <Switch
        id="app-fn-hold"
        checked={enabled}
        onCheckedChange={onCheckedChange}
      />
    </div>
  );
}
