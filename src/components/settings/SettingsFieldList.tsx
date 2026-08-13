import type { ReactNode } from "react";

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
import { cn } from "@/lib/utils";
import type { AppConfig } from "@/types";

import {
  getVisibleFields,
  resolveFieldOptions,
  type SettingsFieldSchema,
  type SettingsSaveMode,
  type SettingsSchemaContext,
  type SettingsSectionSchema,
} from "./settings-schema";

const EMPTY_SELECT_VALUE = "__settings_empty__";

export function SettingsFieldList({
  section,
  config,
  context,
  onChange,
  onBlur,
  renderSlot,
}: {
  section: SettingsSectionSchema | undefined;
  config: AppConfig;
  context: SettingsSchemaContext;
  onChange: (patch: Partial<AppConfig>, saveMode: SettingsSaveMode) => void;
  onBlur: () => void;
  renderSlot: (
    slot: Extract<SettingsFieldSchema, { control: "slot" }>["slot"],
    saveMode: SettingsSaveMode,
  ) => ReactNode;
}) {
  return (
    <div className="grid gap-4 sm:grid-cols-2">
      {getVisibleFields(section, config).map((field) => {
        const wrapperClass = cn(field.span === "full" && "sm:col-span-2");

        if (field.control === "slot") {
          return (
            <div key={field.id} className={wrapperClass}>
              {renderSlot(field.slot, field.saveMode)}
            </div>
          );
        }

        if (field.control === "switch") {
          return (
            <div
              key={field.id}
              className={cn(
                "flex items-center justify-between gap-4 rounded-lg border p-3",
                wrapperClass,
              )}
            >
              <div className="space-y-0.5">
                <Label htmlFor={field.id}>{field.label}</Label>
                {field.description ? (
                  <p className="text-xs text-muted-foreground">
                    {field.description}
                  </p>
                ) : null}
              </div>
              <Switch
                id={field.id}
                checked={config[field.key]}
                disabled={field.disabled?.(config)}
                onCheckedChange={(checked) =>
                  onChange(
                    field.toPatch?.(checked, config) ?? {
                      [field.key]: checked,
                    },
                    field.saveMode,
                  )
                }
              />
            </div>
          );
        }

        if (field.control === "select") {
          const options = resolveFieldOptions(field, context);
          const value = config[field.key] || EMPTY_SELECT_VALUE;
          return (
            <div key={field.id} className={cn("grid gap-2", wrapperClass)}>
              <Label htmlFor={field.id}>{field.label}</Label>
              <Select
                value={value}
                onValueChange={(nextValue) =>
                  onChange(
                    {
                      [field.key]:
                        nextValue === EMPTY_SELECT_VALUE ? "" : nextValue,
                    },
                    field.saveMode,
                  )
                }
              >
                <SelectTrigger id={field.id} className="h-10">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  {options.map((option) => (
                    <SelectItem
                      key={option.value || EMPTY_SELECT_VALUE}
                      value={option.value || EMPTY_SELECT_VALUE}
                    >
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              {field.description ? (
                <p className="text-xs text-muted-foreground">
                  {field.description}
                </p>
              ) : null}
            </div>
          );
        }

        return (
          <div key={field.id} className={cn("grid gap-2", wrapperClass)}>
            <Label htmlFor={field.id}>{field.label}</Label>
            <Input
              id={field.id}
              type={field.control}
              value={config[field.key]}
              placeholder={field.placeholder}
              autoComplete={field.control === "password" ? "off" : undefined}
              onChange={(event) =>
                onChange({ [field.key]: event.target.value }, field.saveMode)
              }
              onBlur={onBlur}
            />
            {field.description ? (
              <p className="text-xs text-muted-foreground">
                {field.description}
              </p>
            ) : null}
          </div>
        );
      })}
    </div>
  );
}
