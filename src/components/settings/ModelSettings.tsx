import { useEffect, useState } from "react";
import {
  ArrowDownIcon,
  ArrowUpIcon,
  Loader2Icon,
  SaveIcon,
  Trash2Icon,
} from "lucide-react";

import { LocalAsrSettings } from "@/components/settings/LocalAsrSettings";
import { RealtimePreviewModelCard } from "@/features/settings/RealtimePreviewModelCard";
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
import {
  commands,
  type ProviderCatalog,
  type ProviderDescriptor,
  type ProviderFieldDescriptor,
  type ProviderOptionValue,
  type ProviderRoutingConfig,
  type ProviderSettings,
} from "@/generated/tauri-bindings";
import type { AppConfig } from "@/types";

type ModelSettingsProps = {
  appConfig: AppConfig | null;
  asrStatus: string;
  textProcessingStatus: string;
  isAsrSaving: boolean;
  isTextProcessingSaving: boolean;
  onSaveAsrConfig: (event: React.FormEvent<HTMLFormElement>) => void;
  onSaveTextProcessingConfig: (event: React.FormEvent<HTMLFormElement>) => void;
  updateConfig: (patch: Partial<AppConfig>) => void;
  onModelChanged: () => void;
};

type Capability = "asr" | "text";

export function ModelSettings(props: ModelSettingsProps) {
  const [catalog, setCatalog] = useState<ProviderCatalog | null>(null);
  const [catalogError, setCatalogError] = useState("");

  useEffect(() => {
    let active = true;
    void commands
      .listProviderCatalog()
      .then((value) => {
        if (active) setCatalog(value);
      })
      .catch((error) => {
        if (active) setCatalogError(`Provider 列表读取失败：${String(error)}`);
      });
    return () => {
      active = false;
    };
  }, []);

  if (catalogError) {
    return <p className="text-sm text-destructive">{catalogError}</p>;
  }
  if (!catalog || !props.appConfig) {
    return (
      <p className="text-sm text-muted-foreground">正在读取 Provider 列表...</p>
    );
  }

  return (
    <>
      <RealtimePreviewModelCard
        onEnabledChange={(enabled) =>
          props.updateConfig({ realtime_preview_enabled: enabled })
        }
        onChanged={props.onModelChanged}
      />
      <ProviderCard
        capability="asr"
        title="语音识别服务"
        description="按顺序配置 ASR primary 与 fallback；每个 Provider 每次只尝试一次"
        descriptors={catalog.asr}
        routing={props.appConfig.asr}
        status={props.asrStatus}
        saving={props.isAsrSaving}
        onSubmit={props.onSaveAsrConfig}
        onChange={(routing) => props.updateConfig({ asr: routing })}
        onModelChanged={props.onModelChanged}
      />
      <ProviderCard
        capability="text"
        title="文本整理服务"
        description="所有文本 Provider 失败时保留 ASR 原文，不影响历史保存"
        descriptors={catalog.text}
        routing={props.appConfig.text}
        status={props.textProcessingStatus}
        saving={props.isTextProcessingSaving}
        onSubmit={props.onSaveTextProcessingConfig}
        onChange={(routing) => props.updateConfig({ text: routing })}
        onModelChanged={props.onModelChanged}
      />
    </>
  );
}

function ProviderCard({
  capability,
  title,
  description,
  descriptors,
  routing,
  status,
  saving,
  onSubmit,
  onChange,
  onModelChanged,
}: {
  capability: Capability;
  title: string;
  description: string;
  descriptors: ProviderDescriptor[];
  routing: ProviderRoutingConfig;
  status: string;
  saving: boolean;
  onSubmit: (event: React.FormEvent<HTMLFormElement>) => void;
  onChange: (routing: ProviderRoutingConfig) => void;
  onModelChanged: () => void;
}) {
  const primary = routing.primary ?? "";
  const fallbacks = routing.fallbacks ?? [];
  const settings = routing.settings ?? {};
  const routeIds = [primary, ...fallbacks].filter(Boolean);
  const availableFallbacks = descriptors.filter(
    (descriptor) => !routeIds.includes(descriptor.id),
  );

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
    onChange({
      ...routing,
      primary: descriptor.id,
      fallbacks: nextFallbacks,
      settings: ensureSettings(settings, descriptor),
    });
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
    onChange({
      ...routing,
      primary,
      fallbacks: [...fallbacks, descriptor.id],
      settings: ensureSettings(settings, descriptor),
    });
  };

  const setFallbacks = (next: string[]) =>
    onChange({ ...routing, fallbacks: next });

  return (
    <Card>
      <CardHeader>
        <CardTitle>{title}</CardTitle>
        <CardDescription>{description}</CardDescription>
      </CardHeader>
      <CardContent>
        <form className="grid gap-5" onSubmit={onSubmit}>
          <div className="grid gap-2">
            <Label htmlFor={`${capability}-primary`}>Primary Provider</Label>
            <Select
              value={primary}
              onValueChange={(value) => {
                const descriptor = descriptors.find(
                  (item) => item.id === value,
                );
                if (descriptor) updateProvider(descriptor);
              }}
            >
              <SelectTrigger id={`${capability}-primary`}>
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

          {fallbacks.length > 0 && (
            <div className="grid gap-2">
              <Label>Fallback 顺序</Label>
              {fallbacks.map((providerId, index) => {
                const descriptor = descriptors.find(
                  (item) => item.id === providerId,
                );
                return (
                  <div
                    key={providerId}
                    className="flex items-center justify-between rounded-md border px-3 py-2"
                  >
                    <span className="text-sm">
                      {index + 1}. {descriptor?.name ?? providerId}
                    </span>
                    <div className="flex gap-1">
                      <Button
                        type="button"
                        size="icon"
                        variant="ghost"
                        disabled={index === 0}
                        aria-label={`上移 ${providerId}`}
                        onClick={() =>
                          setFallbacks(move(fallbacks, index, index - 1))
                        }
                      >
                        <ArrowUpIcon className="size-4" />
                      </Button>
                      <Button
                        type="button"
                        size="icon"
                        variant="ghost"
                        disabled={index === fallbacks.length - 1}
                        aria-label={`下移 ${providerId}`}
                        onClick={() =>
                          setFallbacks(move(fallbacks, index, index + 1))
                        }
                      >
                        <ArrowDownIcon className="size-4" />
                      </Button>
                      <Button
                        type="button"
                        size="icon"
                        variant="ghost"
                        aria-label={`删除 ${providerId}`}
                        onClick={() =>
                          setFallbacks(
                            fallbacks.filter((id) => id !== providerId),
                          )
                        }
                      >
                        <Trash2Icon className="size-4" />
                      </Button>
                    </div>
                  </div>
                );
              })}
            </div>
          )}

          {fallbacks.length < 2 && availableFallbacks.length > 0 && (
            <div className="grid gap-2">
              <Label htmlFor={`${capability}-add-fallback`}>
                添加 {capability === "asr" ? "ASR" : "文本"} fallback
              </Label>
              <Select value="" onValueChange={addFallback}>
                <SelectTrigger
                  id={`${capability}-add-fallback`}
                  aria-label={`添加 ${capability === "asr" ? "ASR" : "文本"} fallback`}
                >
                  <SelectValue placeholder="选择要追加的 Provider" />
                </SelectTrigger>
                <SelectContent>
                  {availableFallbacks.map((descriptor) => (
                    <SelectItem key={descriptor.id} value={descriptor.id}>
                      {descriptor.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              <p className="text-xs text-muted-foreground">
                primary 加 fallback 总计最多 3 项，不自动重试。
              </p>
            </div>
          )}

          {routeIds.map((providerId, index) => {
            const descriptor = descriptors.find(
              (item) => item.id === providerId,
            );
            if (!descriptor) return null;
            return (
              <ProviderFields
                key={providerId}
                capability={capability}
                descriptor={descriptor}
                settings={settings[providerId] ?? defaultSettings(descriptor)}
                routeLabel={index === 0 ? "Primary" : `Fallback ${index}`}
                onChange={(next) =>
                  onChange({
                    ...routing,
                    settings: { ...settings, [providerId]: next },
                  })
                }
                onModelChanged={onModelChanged}
              />
            );
          })}

          <div className="flex flex-col gap-3 border-t pt-4 sm:flex-row sm:items-center sm:justify-between">
            <p className="text-sm leading-6 text-muted-foreground">{status}</p>
            <Button type="submit" size="sm" disabled={saving || !primary}>
              {saving ? (
                <Loader2Icon
                  className="size-4 animate-spin"
                  aria-hidden="true"
                />
              ) : (
                <SaveIcon className="size-4" aria-hidden="true" />
              )}
              保存{capability === "asr" ? " ASR" : "文本处理"}配置
            </Button>
          </div>
        </form>
      </CardContent>
    </Card>
  );
}

function ProviderFields({
  capability,
  descriptor,
  settings,
  routeLabel,
  onChange,
  onModelChanged,
}: {
  capability: Capability;
  descriptor: ProviderDescriptor;
  settings: ProviderSettings;
  routeLabel: string;
  onChange: (settings: ProviderSettings) => void;
  onModelChanged: () => void;
}) {
  return (
    <section className="grid gap-4 rounded-lg border p-4">
      <div>
        <p className="text-sm font-medium">{descriptor.name} 配置</p>
        <p className="text-xs text-muted-foreground">
          {routeLabel} · {descriptor.protocol}
          {descriptor.capabilities.native_hotwords
            ? ` · 原生热词最多 ${descriptor.capabilities.max_hotwords ?? "不限"} 个`
            : descriptor.capabilities.supports_prompt
              ? " · 热词作为软提示"
              : ""}
        </p>
      </div>
      {descriptor.capabilities.local_model_management &&
        capability === "asr" && (
          <LocalAsrSettings onModelChanged={onModelChanged} />
        )}
      {descriptor.fields.map((field) => (
        <ProviderField
          key={field.key}
          capability={capability}
          field={field}
          providerId={descriptor.id}
          settings={settings}
          onChange={onChange}
        />
      ))}
    </section>
  );
}

function ProviderField({
  capability,
  field,
  providerId,
  settings,
  onChange,
}: {
  capability: Capability;
  field: ProviderFieldDescriptor;
  providerId: string;
  settings: ProviderSettings;
  onChange: (settings: ProviderSettings) => void;
}) {
  const id = `${capability}-${providerId}-${field.key}`;
  const options = settings.options ?? {};
  const update = (value: string | boolean | string[]) => {
    if (
      field.key === "api_key" ||
      field.key === "base_url" ||
      field.key === "model"
    ) {
      onChange({ ...settings, [field.key]: String(value) });
      return;
    }
    const option: ProviderOptionValue =
      typeof value === "boolean"
        ? { type: "boolean", value }
        : Array.isArray(value)
          ? { type: "string_list", value }
          : { type: "text", value };
    onChange({ ...settings, options: { ...options, [field.key]: option } });
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
        <p className="text-xs text-muted-foreground">{field.help}</p>
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
