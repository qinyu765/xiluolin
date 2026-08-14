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
    <div className="flex min-h-16 flex-col items-start justify-between gap-3 border-b border-border/70 py-3 sm:flex-row sm:items-center sm:gap-4">
      <div className="min-w-0 space-y-1">
        <Label htmlFor="app-fn-hold" className="text-sm font-medium leading-5">
          按住 Fn 录音
        </Label>
        <p className="text-sm leading-5 text-muted-foreground">
          需要辅助功能权限；组合快捷键仍可使用
        </p>
      </div>
      <Switch
        id="app-fn-hold"
        checked={enabled}
        className="self-end sm:self-auto"
        onCheckedChange={onCheckedChange}
      />
    </div>
  );
}
