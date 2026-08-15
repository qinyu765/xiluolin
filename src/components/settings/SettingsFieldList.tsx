import { Fragment, type ReactNode } from "react";
import { HardDriveIcon, KeyboardIcon, Mic2Icon } from "lucide-react";

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

const GROUPS = {
  shortcuts: { label: "键盘快捷键", icon: KeyboardIcon },
  recording: { label: "录音", icon: Mic2Icon },
  history: { label: "历史与存储", icon: HardDriveIcon },
} as const;

function GroupHeading({ group }: { group: keyof typeof GROUPS }) {
  const { label, icon: Icon } = GROUPS[group];

  return (
    <div className="flex items-center gap-2 pb-2 pt-5 first:pt-1">
      <Icon
        className="size-5 text-foreground"
        strokeWidth={2.2}
        aria-hidden="true"
      />
      <h3 className="text-base font-semibold tracking-tight text-foreground">
        {label}
      </h3>
    </div>
  );
}

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
  const fields = getVisibleFields(section, config);

  return (
    <div>
      {fields.map((field, index) => {
        const group = field.group ? GROUPS[field.group] : null;
        const isFirstInGroup =
          index === 0 || fields[index - 1]?.group !== field.group;

        if (field.control === "slot") {
          return (
            <Fragment key={field.id}>
              {group && isFirstInGroup ? (
                <GroupHeading group={field.group!} />
              ) : null}
              {renderSlot(field.slot, field.saveMode)}
            </Fragment>
          );
        }

        if (field.control === "switch") {
          return (
            <Fragment key={field.id}>
              {group && isFirstInGroup ? (
                <GroupHeading group={field.group!} />
              ) : null}
              <div className="flex min-h-16 flex-col items-start justify-between gap-3 border-b border-border/70 py-3 sm:flex-row sm:items-center sm:gap-4">
                <div className="min-w-0 space-y-1">
                  <Label
                    htmlFor={field.id}
                    className="text-sm font-medium leading-5"
                  >
                    {field.label}
                  </Label>
                  {field.description ? (
                    <p className="text-sm leading-5 text-muted-foreground">
                      {field.description}
                    </p>
                  ) : null}
                </div>
                <Switch
                  id={field.id}
                  checked={config[field.key]}
                  disabled={field.disabled?.(config)}
                  className="self-end sm:self-auto"
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
            </Fragment>
          );
        }

        if (field.control === "select") {
          const options = resolveFieldOptions(field, context);
          const value = config[field.key] || EMPTY_SELECT_VALUE;
          return (
            <Fragment key={field.id}>
              {group && isFirstInGroup ? (
                <GroupHeading group={field.group!} />
              ) : null}
              <div className="flex min-h-16 flex-col items-stretch gap-3 border-b border-border/70 py-3 sm:flex-row sm:items-center sm:justify-between sm:gap-4">
                <div className="min-w-0 space-y-1">
                  <Label
                    htmlFor={field.id}
                    className="text-sm font-medium leading-5"
                  >
                    {field.label}
                  </Label>
                  {field.description ? (
                    <p className="text-sm leading-5 text-muted-foreground">
                      {field.description}
                    </p>
                  ) : null}
                </div>
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
                  <SelectTrigger id={field.id} className="w-full sm:w-64">
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
              </div>
            </Fragment>
          );
        }

        return (
          <Fragment key={field.id}>
            {group && isFirstInGroup ? (
              <GroupHeading group={field.group!} />
            ) : null}
            <div className="flex min-h-16 flex-col items-stretch gap-3 border-b border-border/70 py-3 sm:flex-row sm:items-center sm:justify-between sm:gap-4">
              <div className="min-w-0 space-y-1">
                <Label
                  htmlFor={field.id}
                  className="text-sm font-medium leading-5"
                >
                  {field.label}
                </Label>
                {field.description ? (
                  <p className="text-sm leading-5 text-muted-foreground">
                    {field.description}
                  </p>
                ) : null}
              </div>
              <Input
                id={field.id}
                type={field.control}
                value={config[field.key]}
                placeholder={field.placeholder}
                autoComplete={field.control === "password" ? "off" : undefined}
                className="w-full sm:w-64"
                onChange={(event) =>
                  onChange({ [field.key]: event.target.value }, field.saveMode)
                }
                onBlur={onBlur}
              />
            </div>
          </Fragment>
        );
      })}
    </div>
  );
}
