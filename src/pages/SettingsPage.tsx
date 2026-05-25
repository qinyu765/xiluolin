import React from "react";
import { Loader2Icon, SaveIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardAction,
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
import type { Persona, AppConfig } from "@/types";

type SettingsPageProps = {
  personas: Persona[];
  selectedPersonaId: string;
  selectedPersona: Persona | undefined;
  appConfig: AppConfig | null;
  status: string;
  asrStatus: string;
  openaiStatus: string;
  isSaving: boolean;
  isAsrSaving: boolean;
  isOpenaiSaving: boolean;
  onDefaultPersonaChange: (personaId: string) => void;
  onSaveAsrConfig: (event: React.FormEvent<HTMLFormElement>) => void;
  onSaveOpenaiConfig: (event: React.FormEvent<HTMLFormElement>) => void;
  onConfigChange: (config: AppConfig) => void;
};

export function SettingsPage({
  personas,
  selectedPersonaId,
  selectedPersona,
  appConfig,
  status,
  asrStatus,
  openaiStatus,
  isSaving,
  isAsrSaving,
  isOpenaiSaving,
  onDefaultPersonaChange,
  onSaveAsrConfig,
  onSaveOpenaiConfig,
  onConfigChange,
}: SettingsPageProps) {
  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <div>
            <p className="mb-2 text-xs font-semibold tracking-normal text-primary uppercase">
              T004A UI 基础
            </p>
            <CardTitle className="text-2xl">默认整理风格</CardTitle>
            <CardDescription className="mt-2">
              选择语音内容整理时默认使用的人格。当前阶段只提供内置人格。
            </CardDescription>
          </div>
          <CardAction>
            <span className="inline-flex h-8 items-center rounded-md bg-secondary px-3 text-xs font-medium text-secondary-foreground">
              {isSaving ? "保存中" : "本地配置"}
            </span>
          </CardAction>
        </CardHeader>

        <CardContent className="space-y-5">
          <div className="space-y-2">
            <Label htmlFor="persona-select">默认人格</Label>
            <Select
              value={selectedPersonaId}
              onValueChange={onDefaultPersonaChange}
              disabled={isSaving || personas.length === 0}
            >
              <SelectTrigger id="persona-select" className="h-10">
                <SelectValue placeholder="选择默认人格" />
              </SelectTrigger>
              <SelectContent>
                {personas.map((persona) => (
                  <SelectItem key={persona.id} value={persona.id}>
                    {persona.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {selectedPersona ? (
            <section className="rounded-lg border bg-muted/40 p-4">
              <div className="mb-4 flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
                <div>
                  <h2 className="text-lg font-semibold">
                    {selectedPersona.name}
                  </h2>
                  <p className="mt-1 text-sm leading-6 text-muted-foreground">
                    {selectedPersona.description}
                  </p>
                </div>
                {selectedPersona.is_default ? (
                  <span className="inline-flex h-7 w-fit items-center rounded-md border bg-background px-2.5 text-xs font-medium">
                    默认
                  </span>
                ) : null}
              </div>

              <dl className="grid gap-3 text-sm sm:grid-cols-3">
                <div>
                  <dt className="font-medium text-foreground">适用场景</dt>
                  <dd className="mt-1 leading-6 text-muted-foreground">
                    {selectedPersona.scene}
                  </dd>
                </div>
                <div>
                  <dt className="font-medium text-foreground">输出语气</dt>
                  <dd className="mt-1 leading-6 text-muted-foreground">
                    {selectedPersona.tone}
                  </dd>
                </div>
                <div>
                  <dt className="font-medium text-foreground">输出结构</dt>
                  <dd className="mt-1 leading-6 text-muted-foreground">
                    {selectedPersona.output_structure}
                  </dd>
                </div>
              </dl>
            </section>
          ) : null}

          <div className="flex flex-col gap-3 border-t pt-4 sm:flex-row sm:items-center sm:justify-between">
            <p className="text-sm leading-6 text-muted-foreground">
              {status}
            </p>
            <Button type="button" variant="outline" size="sm" disabled>
              {isSaving ? (
                <Loader2Icon className="size-4 animate-spin" aria-hidden="true" />
              ) : null}
              语音主流程待接入
            </Button>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <div>
            <p className="mb-2 text-xs font-semibold tracking-normal text-primary uppercase">
              T007 OpenAI 整理
            </p>
            <CardTitle className="text-2xl">文本整理服务</CardTitle>
            <CardDescription className="mt-2">
              配置 OpenAI Responses API，用于把原始识别文本整理成可直接使用的结果。
            </CardDescription>
          </div>
          <CardAction>
            <span className="inline-flex h-8 items-center rounded-md bg-secondary px-3 text-xs font-medium text-secondary-foreground">
              {appConfig?.openai_api_key ? "已配置 Key" : "待配置 Key"}
            </span>
          </CardAction>
        </CardHeader>

        <CardContent>
          <form className="grid gap-4" onSubmit={onSaveOpenaiConfig}>
            <div className="grid gap-2">
              <Label htmlFor="openai-api-key">OpenAI API Key</Label>
              <Input
                id="openai-api-key"
                type="password"
                value={appConfig?.openai_api_key ?? ""}
                onChange={(event) =>
                  onConfigChange(
                    appConfig
                      ? { ...appConfig, openai_api_key: event.target.value }
                      : appConfig!,
                  )
                }
                placeholder="本地保存，不写入仓库"
                autoComplete="off"
              />
            </div>

            <div className="grid gap-4 sm:grid-cols-[1fr_180px]">
              <div className="grid gap-2">
                <Label htmlFor="openai-base-url">Base URL</Label>
                <Input
                  id="openai-base-url"
                  value={appConfig?.openai_base_url ?? ""}
                  onChange={(event) =>
                    onConfigChange(
                      appConfig
                        ? { ...appConfig, openai_base_url: event.target.value }
                        : appConfig!,
                    )
                  }
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="openai-model">模型</Label>
                <Input
                  id="openai-model"
                  value={appConfig?.openai_model ?? ""}
                  onChange={(event) =>
                    onConfigChange(
                      appConfig
                        ? { ...appConfig, openai_model: event.target.value }
                        : appConfig!,
                    )
                  }
                />
              </div>
            </div>

            <div className="flex flex-col gap-3 border-t pt-4 sm:flex-row sm:items-center sm:justify-between">
              <p className="text-sm leading-6 text-muted-foreground">
                {openaiStatus}
              </p>
              <Button
                type="submit"
                size="sm"
                disabled={!appConfig || isOpenaiSaving}
              >
                {isOpenaiSaving ? (
                  <Loader2Icon className="size-4 animate-spin" aria-hidden="true" />
                ) : (
                  <SaveIcon className="size-4" aria-hidden="true" />
                )}
                保存 OpenAI 配置
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <div>
            <p className="mb-2 text-xs font-semibold tracking-normal text-primary uppercase">
              T006 智谱 ASR
            </p>
            <CardTitle className="text-2xl">语音识别服务</CardTitle>
            <CardDescription className="mt-2">
              配置智谱 GLM-ASR-2512，用于把短音频转换为原始识别文本。
            </CardDescription>
          </div>
          <CardAction>
            <span className="inline-flex h-8 items-center rounded-md bg-secondary px-3 text-xs font-medium text-secondary-foreground">
              {appConfig?.asr_api_key ? "已配置 Key" : "待配置 Key"}
            </span>
          </CardAction>
        </CardHeader>

        <CardContent>
          <form className="grid gap-4" onSubmit={onSaveAsrConfig}>
            <div className="grid gap-2">
              <Label htmlFor="asr-api-key">智谱 API Key</Label>
              <Input
                id="asr-api-key"
                type="password"
                value={appConfig?.asr_api_key ?? ""}
                onChange={(event) =>
                  onConfigChange(
                    appConfig
                      ? { ...appConfig, asr_api_key: event.target.value }
                      : appConfig!,
                  )
                }
                placeholder="本地保存，不写入仓库"
                autoComplete="off"
              />
            </div>

            <div className="grid gap-4 sm:grid-cols-[1fr_180px]">
              <div className="grid gap-2">
                <Label htmlFor="asr-base-url">Base URL</Label>
                <Input
                  id="asr-base-url"
                  value={appConfig?.asr_base_url ?? ""}
                  onChange={(event) =>
                    onConfigChange(
                      appConfig
                        ? { ...appConfig, asr_base_url: event.target.value }
                        : appConfig!,
                    )
                  }
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="asr-model">模型</Label>
                <Input
                  id="asr-model"
                  value={appConfig?.asr_model ?? ""}
                  onChange={(event) =>
                    onConfigChange(
                      appConfig
                        ? { ...appConfig, asr_model: event.target.value }
                        : appConfig!,
                    )
                  }
                />
              </div>
            </div>

            <div className="flex flex-col gap-3 border-t pt-4 sm:flex-row sm:items-center sm:justify-between">
              <p className="text-sm leading-6 text-muted-foreground">
                {asrStatus}
              </p>
              <Button type="submit" size="sm" disabled={!appConfig || isAsrSaving}>
                {isAsrSaving ? (
                  <Loader2Icon className="size-4 animate-spin" aria-hidden="true" />
                ) : (
                  <SaveIcon className="size-4" aria-hidden="true" />
                )}
                保存 ASR 配置
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
