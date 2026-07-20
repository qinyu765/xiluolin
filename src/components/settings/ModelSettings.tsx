import { Loader2Icon, SaveIcon } from "lucide-react";

import { LocalAsrSettings } from "@/components/settings/LocalAsrSettings";
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

export function ModelSettings(props: ModelSettingsProps) {
  const {
    appConfig,
    asrStatus,
    textProcessingStatus,
    isAsrSaving,
    isTextProcessingSaving,
    onSaveAsrConfig,
    onSaveTextProcessingConfig,
    updateConfig,
    onModelChanged,
  } = props;
  return (
    <>
      <Card>
        <CardHeader>
          <CardTitle>语音识别服务</CardTitle>
          <CardDescription>
            配置 ASR 服务，用于把短音频转换为原始识别文本
          </CardDescription>
        </CardHeader>

        <CardContent>
          <form className="grid gap-4" onSubmit={onSaveAsrConfig}>
            <div className="grid gap-2">
              <Label htmlFor="asr-provider">服务商</Label>
              <Select
                value={appConfig?.asr_provider ?? "zhipu"}
                onValueChange={(value) => updateConfig({ asr_provider: value })}
              >
                <SelectTrigger id="asr-provider" className="h-10">
                  <SelectValue placeholder="选择服务商" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="zhipu">智谱 AI</SelectItem>
                  <SelectItem value="openai">OpenAI 兼容</SelectItem>
                  <SelectItem value="local">本地（离线）</SelectItem>
                </SelectContent>
              </Select>
              <p className="text-xs text-muted-foreground">
                模型名可在下方自行配置；应用不会根据服务商选择覆盖模型名
              </p>
            </div>

            {appConfig?.asr_provider === "local" ? (
              appConfig ? (
                <LocalAsrSettings
                  config={appConfig}
                  onChange={(config) => updateConfig(config)}
                  onModelChanged={onModelChanged}
                />
              ) : null
            ) : appConfig?.asr_provider === "openai" ? (
              <>
                <div className="grid gap-2">
                  <Label htmlFor="openai-asr-api-key">
                    OpenAI API Key <span className="text-destructive">*</span>
                  </Label>
                  <Input
                    id="openai-asr-api-key"
                    type="password"
                    value={appConfig?.openai_api_key ?? ""}
                    onChange={(event) =>
                      updateConfig({ openai_api_key: event.target.value })
                    }
                    placeholder="本地保存，不写入仓库"
                    autoComplete="off"
                    required
                  />
                </div>

                <div className="grid gap-4 sm:grid-cols-[1fr_180px]">
                  <div className="grid gap-2">
                    <Label htmlFor="openai-asr-base-url">Base URL</Label>
                    <Input
                      id="openai-asr-base-url"
                      value={appConfig?.openai_base_url ?? ""}
                      onChange={(event) =>
                        updateConfig({
                          openai_base_url: event.target.value,
                        })
                      }
                    />
                  </div>
                  <div className="grid gap-2">
                    <Label htmlFor="openai-asr-model">模型</Label>
                    <Input
                      id="openai-asr-model"
                      value={appConfig?.openai_asr_model ?? ""}
                      onChange={(event) =>
                        updateConfig({
                          openai_asr_model: event.target.value,
                        })
                      }
                    />
                  </div>
                </div>
              </>
            ) : (
              <>
                <div className="grid gap-2">
                  <Label htmlFor="asr-api-key">
                    智谱 API Key <span className="text-destructive">*</span>
                  </Label>
                  <Input
                    id="asr-api-key"
                    type="password"
                    value={appConfig?.asr_api_key ?? ""}
                    onChange={(event) =>
                      updateConfig({ asr_api_key: event.target.value })
                    }
                    placeholder="本地保存，不写入仓库"
                    autoComplete="off"
                    required
                  />
                </div>

                <div className="grid gap-4 sm:grid-cols-[1fr_180px]">
                  <div className="grid gap-2">
                    <Label htmlFor="asr-base-url">Base URL</Label>
                    <Input
                      id="asr-base-url"
                      value={appConfig?.asr_base_url ?? ""}
                      onChange={(event) =>
                        updateConfig({ asr_base_url: event.target.value })
                      }
                    />
                  </div>
                  <div className="grid gap-2">
                    <Label htmlFor="asr-model">模型</Label>
                    <Input
                      id="asr-model"
                      value={appConfig?.asr_model ?? ""}
                      onChange={(event) =>
                        updateConfig({ asr_model: event.target.value })
                      }
                    />
                  </div>
                </div>
              </>
            )}

            <div className="flex flex-col gap-3 border-t pt-4 sm:flex-row sm:items-center sm:justify-between">
              <p className="text-sm leading-6 text-muted-foreground">
                {asrStatus}
              </p>
              <Button
                type="submit"
                size="sm"
                disabled={!appConfig || isAsrSaving}
              >
                {isAsrSaving ? (
                  <Loader2Icon
                    className="size-4 animate-spin"
                    aria-hidden="true"
                  />
                ) : (
                  <SaveIcon className="size-4" aria-hidden="true" />
                )}
                保存 ASR 配置
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>文本整理服务</CardTitle>
          <CardDescription>
            配置文本处理 API，用于把原始识别文本整理成可直接使用的结果
          </CardDescription>
        </CardHeader>

        <CardContent>
          <form className="grid gap-4" onSubmit={onSaveTextProcessingConfig}>
            <div className="grid gap-2">
              <Label htmlFor="text-provider">服务商</Label>
              <Select
                value={appConfig?.text_provider ?? "zhipu"}
                onValueChange={(value) =>
                  updateConfig({ text_provider: value })
                }
              >
                <SelectTrigger id="text-provider" className="h-10">
                  <SelectValue placeholder="选择服务商" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="zhipu">智谱 AI</SelectItem>
                  <SelectItem value="openai">OpenAI 兼容</SelectItem>
                </SelectContent>
              </Select>
              <p className="text-xs text-muted-foreground">
                模型名可在下方自行配置；应用不会根据服务商选择覆盖模型名
              </p>
            </div>

            {appConfig?.text_provider === "zhipu" ? (
              <>
                <div className="grid gap-2">
                  <Label htmlFor="zhipu-api-key">
                    智谱 API Key <span className="text-destructive">*</span>
                  </Label>
                  <Input
                    id="zhipu-api-key"
                    type="password"
                    value={appConfig?.zhipu_api_key ?? ""}
                    onChange={(event) =>
                      updateConfig({ zhipu_api_key: event.target.value })
                    }
                    placeholder="本地保存，不写入仓库"
                    autoComplete="off"
                    required
                  />
                </div>

                <div className="grid gap-4 sm:grid-cols-[1fr_180px]">
                  <div className="grid gap-2">
                    <Label htmlFor="zhipu-base-url">Base URL</Label>
                    <Input
                      id="zhipu-base-url"
                      value={appConfig?.zhipu_base_url ?? ""}
                      onChange={(event) =>
                        updateConfig({ zhipu_base_url: event.target.value })
                      }
                    />
                  </div>
                  <div className="grid gap-2">
                    <Label htmlFor="zhipu-model">模型</Label>
                    <Input
                      id="zhipu-model"
                      value={appConfig?.zhipu_model ?? ""}
                      onChange={(event) =>
                        updateConfig({ zhipu_model: event.target.value })
                      }
                    />
                  </div>
                </div>
              </>
            ) : (
              <>
                <div className="grid gap-2">
                  <Label htmlFor="openai-api-key">
                    OpenAI API Key <span className="text-destructive">*</span>
                  </Label>
                  <Input
                    id="openai-api-key"
                    type="password"
                    value={appConfig?.openai_api_key ?? ""}
                    onChange={(event) =>
                      updateConfig({ openai_api_key: event.target.value })
                    }
                    placeholder="本地保存，不写入仓库"
                    autoComplete="off"
                    required
                  />
                </div>

                <div className="grid gap-4 sm:grid-cols-[1fr_180px]">
                  <div className="grid gap-2">
                    <Label htmlFor="openai-base-url">Base URL</Label>
                    <Input
                      id="openai-base-url"
                      value={appConfig?.openai_base_url ?? ""}
                      onChange={(event) =>
                        updateConfig({
                          openai_base_url: event.target.value,
                        })
                      }
                    />
                  </div>
                  <div className="grid gap-2">
                    <Label htmlFor="openai-model">模型</Label>
                    <Input
                      id="openai-model"
                      value={appConfig?.openai_model ?? ""}
                      onChange={(event) =>
                        updateConfig({ openai_model: event.target.value })
                      }
                    />
                  </div>
                </div>
              </>
            )}

            <div className="flex flex-col gap-3 border-t pt-4 sm:flex-row sm:items-center sm:justify-between">
              <p className="text-sm leading-6 text-muted-foreground">
                {textProcessingStatus}
              </p>
              <Button
                type="submit"
                size="sm"
                disabled={!appConfig || isTextProcessingSaving}
              >
                {isTextProcessingSaving ? (
                  <Loader2Icon
                    className="size-4 animate-spin"
                    aria-hidden="true"
                  />
                ) : (
                  <SaveIcon className="size-4" aria-hidden="true" />
                )}
                保存文本处理配置
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>
    </>
  );
}
