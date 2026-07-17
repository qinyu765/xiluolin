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
