import { useEffect, useMemo, useState } from "react";
import {
  AlertCircleIcon,
  ArrowDownIcon,
  ArrowUpIcon,
  CheckCircle2Icon,
  ChevronRightIcon,
  ListOrderedIcon,
  PlusIcon,
  Settings2Icon,
  Trash2Icon,
  type LucideIcon,
} from "lucide-react";

import { LocalAsrSettings } from "@/components/settings/LocalAsrSettings";
import type { SettingsSaveMode } from "@/components/settings/settings-schema";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
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
import {
  type ProviderDescriptor,
  type ProviderFieldDescriptor,
  type ProviderOptionValue,
  type ProviderRoutingConfig,
  type ProviderSettings,
} from "@/generated/tauri-bindings";

export type ProviderCapability = "asr" | "text";

type ProviderCapabilityCardProps = {
  capability: ProviderCapability;
  title: string;
  hint: string;
  icon: LucideIcon;
  descriptors: ProviderDescriptor[];
  routing: ProviderRoutingConfig;
  onChange: (
    routing: ProviderRoutingConfig,
    saveMode: SettingsSaveMode,
  ) => void;
  onConfigBlur: () => void;
  onModelChanged: () => void;
};

type ProviderEditorDialogProps = ProviderCapabilityCardProps & {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  selectedProviderId: string | null;
  onSelectedProviderChange: (providerId: string | null) => void;
};

export function ProviderCapabilityCard({
  capability,
  title,
  hint,
  icon: Icon,
  descriptors,
  routing,
  onChange,
  onConfigBlur,
  onModelChanged,
}: ProviderCapabilityCardProps) {
  const [open, setOpen] = useState(false);
  const primary = routing.primary ?? "";
  const fallbacks = useMemo(() => routing.fallbacks ?? [], [routing.fallbacks]);
  const routeIds = useMemo(
    () => [primary, ...fallbacks].filter(Boolean),
    [fallbacks, primary],
  );
  const [selectedProviderId, setSelectedProviderId] = useState<string | null>(
    primary || fallbacks[0] || null,
  );

  useEffect(() => {
    if (
      selectedProviderId &&
      (routeIds.includes(selectedProviderId) || !open)
    ) {
      return;
    }
    setSelectedProviderId(primary || fallbacks[0] || null);
  }, [descriptors, fallbacks, open, primary, routeIds, selectedProviderId]);

  const openEditor = (providerId?: string) => {
    setSelectedProviderId(providerId || primary || fallbacks[0] || null);
    setOpen(true);
  };

  return (
    <section
      className={
        capability === "asr"
          ? "overflow-hidden rounded-xl border border-sky-200/80 bg-card shadow-sm"
          : "overflow-hidden rounded-xl border border-violet-200/80 bg-card shadow-sm"
      }
    >
      <div className="flex items-start justify-between gap-3 border-b px-4 py-4 sm:px-5">
        <div className="flex min-w-0 items-start gap-3">
          <div
            className={
              capability === "asr"
                ? "flex size-9 shrink-0 items-center justify-center rounded-lg bg-sky-500/10 text-sky-700"
                : "flex size-9 shrink-0 items-center justify-center rounded-lg bg-violet-500/10 text-violet-700"
            }
          >
            <Icon className="size-5" strokeWidth={2.1} aria-hidden="true" />
          </div>
          <div className="min-w-0">
            <h2 className="text-base font-semibold tracking-tight">{title}</h2>
            <p className="mt-1 text-sm leading-5 text-muted-foreground">
              {hint}
            </p>
          </div>
        </div>
        <Button
          type="button"
          variant="outline"
          size="sm"
          className="shrink-0"
          onClick={() => openEditor()}
        >
          <Settings2Icon className="size-4" aria-hidden="true" />
          管理
        </Button>
      </div>

      <div className="space-y-2 p-4 sm:p-5">
        {routeIds.length > 0 ? (
          routeIds.map((providerId, index) => {
            const descriptor = descriptors.find(
              (item) => item.id === providerId,
            );
            if (!descriptor) return null;
            return (
              <ProviderSummaryRow
                key={providerId}
                capability={capability}
                descriptor={descriptor}
                settings={
                  routing.settings?.[providerId] ?? defaultSettings(descriptor)
                }
                routeLabel={index === 0 ? "主" : `备用 ${index}`}
                onClick={() => openEditor(providerId)}
              />
            );
          })
        ) : (
          <button
            type="button"
            className="flex w-full items-center justify-between rounded-lg border border-dashed px-3 py-3 text-left transition-colors hover:border-primary/50 hover:bg-primary/5 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            onClick={() => openEditor()}
          >
            <span>
              <span className="block text-sm font-medium">
                尚未选择 Provider
              </span>
              <span className="mt-0.5 block text-xs text-muted-foreground">
                点击配置调用链
              </span>
            </span>
            <ChevronRightIcon className="size-4 text-muted-foreground" />
          </button>
        )}
      </div>

      <ProviderEditorDialog
        open={open}
        onOpenChange={setOpen}
        selectedProviderId={selectedProviderId}
        onSelectedProviderChange={setSelectedProviderId}
        capability={capability}
        title={title}
        hint={hint}
        icon={Icon}
        descriptors={descriptors}
        routing={routing}
        onChange={onChange}
        onConfigBlur={onConfigBlur}
        onModelChanged={onModelChanged}
      />
    </section>
  );
}

function ProviderSummaryRow({
  capability,
  descriptor,
  settings,
  routeLabel,
  onClick,
}: {
  capability: ProviderCapability;
  descriptor: ProviderDescriptor;
  settings: ProviderSettings;
  routeLabel: string;
  onClick: () => void;
}) {
  const status = providerConfigurationStatus(descriptor, settings);
  const model =
    settings.model?.trim() || descriptor.default_model || "默认模型";

  return (
    <button
      type="button"
      className="group flex w-full items-center gap-3 rounded-lg border bg-background px-3 py-2.5 text-left transition-colors hover:border-primary/40 hover:bg-primary/5 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
      onClick={onClick}
    >
      <span
        className={
          capability === "asr"
            ? "w-14 shrink-0 text-xs font-semibold uppercase tracking-wide text-sky-700"
            : "w-14 shrink-0 text-xs font-semibold uppercase tracking-wide text-violet-700"
        }
      >
        {routeLabel}
      </span>
      <span className="min-w-0 flex-1">
        <span className="block truncate text-sm font-medium">
          {descriptor.name}
        </span>
        <span className="mt-0.5 block truncate text-xs text-muted-foreground">
          {model}
        </span>
      </span>
      <ProviderStatus status={status} />
      <ChevronRightIcon className="size-4 shrink-0 text-muted-foreground transition-transform group-hover:translate-x-0.5" />
    </button>
  );
}

function ProviderStatus({
  status,
}: {
  status: ReturnType<typeof providerConfigurationStatus>;
}) {
  return (
    <span
      className={
        status.ready
          ? "inline-flex shrink-0 items-center gap-1 text-xs text-emerald-700"
          : "inline-flex shrink-0 items-center gap-1 text-xs text-amber-700"
      }
    >
      {status.ready ? (
        <CheckCircle2Icon className="size-3.5" aria-hidden="true" />
      ) : (
        <AlertCircleIcon className="size-3.5" aria-hidden="true" />
      )}
      <span className="hidden sm:inline">{status.label}</span>
      <span className="sr-only">{status.label}</span>
    </span>
  );
}

function ProviderEditorDialog({
  open,
  onOpenChange,
  selectedProviderId,
  onSelectedProviderChange,
  capability,
  title,
  hint,
  descriptors,
  routing,
  onChange,
  onConfigBlur,
  onModelChanged,
}: ProviderEditorDialogProps) {
  const primary = routing.primary ?? "";
  const fallbacks = routing.fallbacks ?? [];
  const settings = routing.settings ?? {};
  const routeIds = [primary, ...fallbacks].filter(Boolean);
  const availableFallbacks = descriptors.filter(
    (descriptor) => !routeIds.includes(descriptor.id),
  );
  const selectedDescriptor = descriptors.find(
    (descriptor) => descriptor.id === selectedProviderId,
  );

  const selectProvider = (providerId: string | null) => {
    onSelectedProviderChange(providerId);
  };

  const updateProvider = (descriptor: ProviderDescriptor) => {
    const nextFallbacks = fallbacks.filter(
      (providerId) => providerId !== descriptor.id,
    );
    if (
      capability === "asr" &&
      descriptor.id === "local" &&
      nextFallbacks.some((providerId) => providerId !== "local") &&
      !confirmCloudFallback("切换为本地 Whisper 后")
    ) {
      return;
    }
    onChange(
      {
        ...routing,
        primary: descriptor.id,
        fallbacks: nextFallbacks,
        settings: ensureSettings(settings, descriptor),
      },
      "immediate",
    );
    selectProvider(descriptor.id);
  };

  const setFallbacks = (next: string[]) => {
    onChange({ ...routing, fallbacks: next }, "immediate");
    if (
      selectedProviderId &&
      ![primary, ...next].includes(selectedProviderId)
    ) {
      selectProvider(primary || next[0] || null);
    }
  };

  const addFallback = (providerId: string) => {
    const descriptor = descriptors.find((item) => item.id === providerId);
    if (!descriptor || fallbacks.length >= 2) return;
    if (
      capability === "asr" &&
      primary === "local" &&
      descriptor.id !== "local" &&
      !confirmCloudFallback(`添加 ${descriptor.name} 后`)
    ) {
      return;
    }
    onChange(
      {
        ...routing,
        primary,
        fallbacks: [...fallbacks, descriptor.id],
        settings: ensureSettings(settings, descriptor),
      },
      "immediate",
    );
    selectProvider(descriptor.id);
  };

  const selectedSettings = selectedDescriptor
    ? (settings[selectedDescriptor.id] ?? defaultSettings(selectedDescriptor))
    : null;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="!flex h-[min(90vh,48rem)] max-h-[calc(100dvh-2rem)] min-h-0 max-w-5xl flex-col gap-0 overflow-hidden p-0">
        <DialogHeader className="shrink-0 border-b px-5 py-4 pr-12 sm:px-6">
          <DialogTitle className="flex items-center gap-2 text-base">
            <span>{title}</span>
            <span className="text-muted-foreground">·</span>
            <span className="font-normal text-muted-foreground">
              Provider 配置
            </span>
          </DialogTitle>
          <DialogDescription>{hint} 修改会自动保存。</DialogDescription>
        </DialogHeader>

        <div
          data-testid="provider-editor-body"
          className="grid min-h-0 min-w-0 flex-1 grid-rows-[minmax(0,auto)_minmax(0,1fr)] overflow-hidden lg:grid-cols-[17rem_minmax(0,1fr)] lg:grid-rows-none"
        >
          <aside
            data-testid="provider-editor-sidebar"
            className="min-h-0 overflow-y-auto overscroll-contain border-b bg-muted/25 p-4 lg:border-r lg:border-b-0 sm:p-5"
          >
            <div className="flex items-center justify-between">
              <div>
                <p className="text-sm font-semibold">调用链</p>
                <p className="mt-0.5 text-xs text-muted-foreground">
                  最多 1 个主 Provider + 2 个备用
                </p>
              </div>
              <ListOrderedIcon
                className="size-4 text-muted-foreground"
                aria-hidden="true"
              />
            </div>

            <div className="mt-4 grid gap-2">
              <Label
                htmlFor={`${capability}-dialog-primary`}
                className="text-xs text-muted-foreground"
              >
                主 Provider
              </Label>
              <Select
                value={primary}
                onValueChange={(value) => {
                  const descriptor = descriptors.find(
                    (item) => item.id === value,
                  );
                  if (descriptor) updateProvider(descriptor);
                }}
              >
                <SelectTrigger id={`${capability}-dialog-primary`}>
                  <SelectValue placeholder="选择 Provider" />
                </SelectTrigger>
                <SelectContent>
                  {descriptors.map((descriptor) => (
                    <SelectItem key={descriptor.id} value={descriptor.id}>
                      {descriptor.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <div className="mt-5 grid gap-2">
              <p className="text-xs text-muted-foreground">备用顺序</p>
              {fallbacks.map((providerId, index) => {
                const descriptor = descriptors.find(
                  (item) => item.id === providerId,
                );
                return (
                  <div
                    key={providerId}
                    className={
                      selectedProviderId === providerId
                        ? "rounded-lg border border-primary/40 bg-primary/5 p-2"
                        : "rounded-lg border bg-background p-2"
                    }
                  >
                    <button
                      type="button"
                      className="flex w-full items-center gap-2 text-left focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                      onClick={() => selectProvider(providerId)}
                    >
                      <span className="flex size-5 shrink-0 items-center justify-center rounded-full bg-muted text-[11px] font-semibold text-muted-foreground">
                        {index + 1}
                      </span>
                      <span className="min-w-0 flex-1 truncate text-sm font-medium">
                        {descriptor?.name ?? providerId}
                      </span>
                      <ChevronRightIcon className="size-4 shrink-0 text-muted-foreground" />
                    </button>
                    <div className="mt-1 flex justify-end gap-0.5">
                      <Button
                        type="button"
                        size="icon"
                        variant="ghost"
                        className="size-7"
                        disabled={index === 0}
                        aria-label={`上移 ${providerId}`}
                        onClick={() =>
                          setFallbacks(move(fallbacks, index, index - 1))
                        }
                      >
                        <ArrowUpIcon className="size-3.5" />
                      </Button>
                      <Button
                        type="button"
                        size="icon"
                        variant="ghost"
                        className="size-7"
                        disabled={index === fallbacks.length - 1}
                        aria-label={`下移 ${providerId}`}
                        onClick={() =>
                          setFallbacks(move(fallbacks, index, index + 1))
                        }
                      >
                        <ArrowDownIcon className="size-3.5" />
                      </Button>
                      <Button
                        type="button"
                        size="icon"
                        variant="ghost"
                        className="size-7"
                        aria-label={`删除 ${providerId}`}
                        onClick={() =>
                          setFallbacks(
                            fallbacks.filter((id) => id !== providerId),
                          )
                        }
                      >
                        <Trash2Icon className="size-3.5" />
                      </Button>
                    </div>
                  </div>
                );
              })}
              {fallbacks.length < 2 && availableFallbacks.length > 0 ? (
                <Select value="" onValueChange={addFallback}>
                  <SelectTrigger
                    id={`${capability}-dialog-add-fallback`}
                    aria-label={`添加 ${capability === "asr" ? "ASR" : "文本"} 备用 Provider`}
                  >
                    <PlusIcon className="size-4" aria-hidden="true" />
                    <SelectValue placeholder="添加备用 Provider" />
                  </SelectTrigger>
                  <SelectContent>
                    {availableFallbacks.map((descriptor) => (
                      <SelectItem key={descriptor.id} value={descriptor.id}>
                        {descriptor.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              ) : null}
            </div>
          </aside>

          <div
            data-testid="provider-editor-scroll"
            className="min-h-0 min-w-0 overflow-y-auto overscroll-contain p-4 sm:p-6"
          >
            {selectedDescriptor && selectedSettings ? (
              <div>
                <div className="flex items-start justify-between gap-3 border-b pb-4">
                  <div className="min-w-0">
                    <p className="text-xs font-semibold uppercase tracking-[0.12em] text-muted-foreground">
                      {selectedProviderId === primary && primary
                        ? "主 Provider"
                        : fallbacks.includes(selectedProviderId ?? "")
                          ? `备用 ${fallbacks.indexOf(selectedProviderId ?? "") + 1}`
                          : "待加入调用链"}
                    </p>
                    <h3 className="mt-1 text-lg font-semibold tracking-tight">
                      {selectedDescriptor.name}
                    </h3>
                    <p className="mt-1 text-sm leading-5 text-muted-foreground">
                      {selectedDescriptor.protocol}
                      {selectedDescriptor.capabilities.native_hotwords
                        ? ` · 原生热词最多 ${selectedDescriptor.capabilities.max_hotwords ?? "不限"} 个`
                        : selectedDescriptor.capabilities.supports_prompt
                          ? " · 热词作为软提示"
                          : ""}
                    </p>
                  </div>
                  <ProviderStatus
                    status={providerConfigurationStatus(
                      selectedDescriptor,
                      selectedSettings,
                    )}
                  />
                </div>

                <ProviderFields
                  capability={capability}
                  descriptor={selectedDescriptor}
                  settings={selectedSettings}
                  onChange={(next, saveMode) =>
                    onChange(
                      {
                        ...routing,
                        settings: {
                          ...settings,
                          [selectedDescriptor.id]: next,
                        },
                      },
                      saveMode,
                    )
                  }
                  onConfigBlur={onConfigBlur}
                  onModelChanged={onModelChanged}
                />
              </div>
            ) : (
              <div className="flex min-h-56 items-center justify-center rounded-lg border border-dashed bg-muted/20 p-6 text-center">
                <div>
                  <p className="text-sm font-medium">先选择一个 Provider</p>
                  <p className="mt-1 text-xs text-muted-foreground">
                    选择主 Provider 后，这里会显示对应配置项。
                  </p>
                </div>
              </div>
            )}
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}

function ProviderFields({
  capability,
  descriptor,
  settings,
  onChange,
  onConfigBlur,
  onModelChanged,
}: {
  capability: ProviderCapability;
  descriptor: ProviderDescriptor;
  settings: ProviderSettings;
  onChange: (settings: ProviderSettings, saveMode: SettingsSaveMode) => void;
  onConfigBlur: () => void;
  onModelChanged: () => void;
}) {
  return (
    <div className="grid gap-4 pt-5">
      {descriptor.capabilities.local_model_management &&
      capability === "asr" ? (
        <LocalAsrSettings onModelChanged={onModelChanged} />
      ) : null}

      {descriptor.fields.length > 0 ? (
        <div className="grid gap-4 lg:grid-cols-2">
          {descriptor.fields.map((field) => (
            <ProviderField
              key={field.key}
              capability={capability}
              field={field}
              providerId={descriptor.id}
              settings={settings}
              onChange={onChange}
              onConfigBlur={onConfigBlur}
            />
          ))}
        </div>
      ) : !descriptor.capabilities.local_model_management ? (
        <p className="rounded-lg border border-dashed bg-muted/20 px-3 py-2 text-sm text-muted-foreground">
          该 Provider 使用默认参数，无额外配置项。
        </p>
      ) : null}
    </div>
  );
}

function ProviderField({
  capability,
  field,
  providerId,
  settings,
  onChange,
  onConfigBlur,
}: {
  capability: ProviderCapability;
  field: ProviderFieldDescriptor;
  providerId: string;
  settings: ProviderSettings;
  onChange: (settings: ProviderSettings, saveMode: SettingsSaveMode) => void;
  onConfigBlur: () => void;
}) {
  const id = `${capability}-${providerId}-${field.key}`;
  const options = settings.options ?? {};
  const update = (value: string | boolean | string[]) => {
    if (
      field.key === "api_key" ||
      field.key === "base_url" ||
      field.key === "model"
    ) {
      onChange(
        { ...settings, [field.key]: String(value) },
        field.kind === "text" || field.kind === "api_key"
          ? "debounced"
          : "immediate",
      );
      return;
    }
    const option: ProviderOptionValue =
      typeof value === "boolean"
        ? { type: "boolean", value }
        : Array.isArray(value)
          ? { type: "string_list", value }
          : { type: "text", value };
    onChange(
      { ...settings, options: { ...options, [field.key]: option } },
      field.kind === "multi_select" ? "debounced" : "immediate",
    );
  };
  const value = fieldValue(field, settings);

  return (
    <div className="grid gap-2">
      <Label htmlFor={id}>
        {field.label}
        {field.required && <span className="text-destructive">*</span>}
      </Label>
      {field.kind === "switch" ? (
        <Switch id={id} checked={Boolean(value)} onCheckedChange={update} />
      ) : field.kind === "select" ? (
        <Select value={String(value)} onValueChange={update}>
          <SelectTrigger id={id}>
            <SelectValue placeholder={field.placeholder || "自动"} />
          </SelectTrigger>
          <SelectContent>
            {field.choices.map((choice) => (
              <SelectItem key={choice.value} value={choice.value}>
                {choice.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      ) : (
        <Input
          id={id}
          type={field.secret ? "password" : "text"}
          value={Array.isArray(value) ? value.join(", ") : String(value)}
          placeholder={field.placeholder}
          autoComplete={field.secret ? "off" : undefined}
          required={field.required}
          onBlur={onConfigBlur}
          onChange={(event) =>
            update(
              field.kind === "multi_select"
                ? event.target.value
                    .split(",")
                    .map((item) => item.trim())
                    .filter(Boolean)
                    .slice(0, field.max_items ?? undefined)
                : event.target.value,
            )
          }
        />
      )}
      {field.help && (
        <p className="text-xs leading-5 text-muted-foreground">{field.help}</p>
      )}
    </div>
  );
}

function fieldValue(
  field: ProviderFieldDescriptor,
  settings: ProviderSettings,
) {
  if (
    field.key === "api_key" ||
    field.key === "base_url" ||
    field.key === "model"
  ) {
    return settings[field.key] ?? "";
  }
  const option = settings.options?.[field.key];
  return option?.value ?? (field.kind === "switch" ? false : "");
}

function providerConfigurationStatus(
  descriptor: ProviderDescriptor,
  settings: ProviderSettings,
) {
  const ready = descriptor.fields
    .filter((field) => field.required)
    .every((field) => {
      const value = fieldValue(field, settings);
      if (typeof value === "boolean") return value;
      if (Array.isArray(value)) return value.length > 0;
      return String(value).trim().length > 0;
    });
  return { ready, label: ready ? "已配置" : "待补充" };
}

function defaultSettings(descriptor: ProviderDescriptor): ProviderSettings {
  return {
    api_key: "",
    base_url: descriptor.default_base_url,
    model: descriptor.default_model,
    options: {},
  };
}

function ensureSettings(
  settings: Record<string, ProviderSettings>,
  descriptor: ProviderDescriptor,
) {
  return settings[descriptor.id]
    ? settings
    : { ...settings, [descriptor.id]: defaultSettings(descriptor) };
}

function move(values: string[], from: number, to: number) {
  if (to < 0 || to >= values.length) return values;
  const result = [...values];
  const [value] = result.splice(from, 1);
  result.splice(to, 0, value);
  return result;
}

function confirmCloudFallback(action: string) {
  return window.confirm(
    `${action}，音频会在本地识别失败时发送到云端。是否确认授权？`,
  );
}
