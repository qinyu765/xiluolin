import * as React from "react";
import * as LucideIcons from "lucide-react";
import { cn } from "@/lib/utils";

const AVAILABLE_ICONS = [
  "Bot",
  "ClipboardList",
  "Lightbulb",
  "Mail",
  "Languages",
  "Sparkles",
  "Zap",
  "Target",
  "Palette",
  "BookOpen",
  "Code",
  "Briefcase",
  "Heart",
  "Star",
  "Flame",
] as const;

type IconName = (typeof AVAILABLE_ICONS)[number];

interface IconPickerProps {
  value: string;
  onChange: (icon: string) => void;
}

export function IconPicker({ value, onChange }: IconPickerProps) {
  return (
    <div className="grid grid-cols-5 gap-2">
      {AVAILABLE_ICONS.map((iconName) => {
        const IconComponent =
          LucideIcons[iconName as keyof typeof LucideIcons];
        const isSelected = value === iconName;

        return (
          <button
            key={iconName}
            type="button"
            onClick={() => onChange(iconName)}
            className={cn(
              "flex items-center justify-center w-12 h-12 rounded-md border-2 transition-colors",
              isSelected
                ? "border-primary bg-primary/10"
                : "border-border hover:border-primary/50 hover:bg-accent"
            )}
            title={iconName}
          >
            {IconComponent && (
              <IconComponent className="w-5 h-5" strokeWidth={2} />
            )}
          </button>
        );
      })}
    </div>
  );
}
