import * as React from "react";
import { cn } from "@/lib/utils";

import { getPersonaIcon, PERSONA_ICON_NAMES } from "@/lib/persona-icons";

interface IconPickerProps {
  value: string;
  onChange: (icon: string) => void;
}

export function IconPicker({ value, onChange }: IconPickerProps) {
  return (
    <div className="grid grid-cols-5 gap-2">
      {PERSONA_ICON_NAMES.map((iconName) => {
        const IconComponent = getPersonaIcon(iconName);
        const isSelected = value === iconName;

        return (
          <button
            key={iconName}
            type="button"
            onClick={() => onChange(iconName)}
            className={cn(
              "flex h-12 w-12 cursor-pointer items-center justify-center rounded-md border-2 transition-[color,background-color,border-color,box-shadow,transform] duration-150 hover:-translate-y-px hover:shadow-sm active:translate-y-0 active:scale-95 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/50",
              isSelected
                ? "border-primary bg-primary/10 shadow-xs"
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
