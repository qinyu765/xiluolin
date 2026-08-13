import { useEffect, useState } from "react";

import {
  commands,
  type ProviderRoutingConfig,
  type ProviderSettings,
} from "@/generated/tauri-bindings";
import type { AppConfig, AudioDevice } from "@/types";
import { toErrorMessage } from "@/utils/error";

export function useConfigController() {
  const [appConfig, setAppConfig] = useState<AppConfig | null>(null);
  const [audioDevices, setAudioDevices] = useState<AudioDevice[]>([]);
  const [asrStatus, setAsrStatus] = useState("正在读取 ASR 配置...");
  const [textProcessingStatus, setTextProcessingStatus] =
    useState("正在读取文本处理配置...");
  const [isAsrSaving, setIsAsrSaving] = useState(false);
  const [isTextProcessingSaving, setIsTextProcessingSaving] = useState(false);

  useEffect(() => {
    let active = true;
    void Promise.all([
      commands.initializeLocalData(),
      commands.listAudioDevices().catch(() => []),
    ])
      .then(([config, devices]) => {
        if (!active) return;
        setAppConfig(config as AppConfig);
        setAudioDevices(devices);
        setAsrStatus("ASR 配置已加载。");
        setTextProcessingStatus("文本处理配置已加载。");
      })
      .catch((error) => {
        if (!active) return;
        setAsrStatus(`ASR 配置读取失败：${toErrorMessage(error)}`);
        setTextProcessingStatus("文本处理配置读取失败。");
      });
    return () => {
      active = false;
    };
  }, []);

  const saveConfig = async (config: AppConfig) => {
    const saved = (await commands.updateAppConfig(config)) as AppConfig;
    setAppConfig(saved);
    window.dispatchEvent(new Event("xiluolin:readiness-changed"));
    return saved;
  };

  const handleSaveAsrConfig = async (
    event: React.FormEvent<HTMLFormElement>,
  ) => {
    event.preventDefault();
    if (!appConfig) return;

    const nextConfig = normalizeProviderConfig(appConfig);
    const invalidProvider = invalidRouteProvider(nextConfig.asr, true);
    if (invalidProvider) {
      setAsrStatus(
        `${invalidProvider} 的 API Key、Base URL 或模型配置不完整。`,
      );
      return;
    }

    setIsAsrSaving(true);
    setAsrStatus("正在保存 ASR 配置...");
    try {
      await saveConfig(nextConfig);
      setAsrStatus("ASR Provider 调用链已保存。");
    } catch (error) {
      setAsrStatus(`保存 ASR 配置失败：${toErrorMessage(error)}`);
    } finally {
      setIsAsrSaving(false);
    }
  };

  const handleSaveTextProcessingConfig = async (
    event: React.FormEvent<HTMLFormElement>,
  ) => {
    event.preventDefault();
    if (!appConfig) return;

    const nextConfig = normalizeProviderConfig(appConfig);
    const invalidProvider = invalidRouteProvider(nextConfig.text, false);
    if (invalidProvider) {
      setTextProcessingStatus(
        `${invalidProvider} 的 API Key、Base URL 或模型配置不完整。`,
      );
      return;
    }

    setIsTextProcessingSaving(true);
    setTextProcessingStatus("正在保存文本 Provider 调用链...");
    try {
      await saveConfig(nextConfig);
      setTextProcessingStatus("文本 Provider 调用链已保存。");
    } catch (error) {
      setTextProcessingStatus(`保存文本处理配置失败：${toErrorMessage(error)}`);
    } finally {
      setIsTextProcessingSaving(false);
    }
  };

  return {
    appConfig,
    audioDevices,
    asrStatus,
    textProcessingStatus,
    isAsrSaving,
    isTextProcessingSaving,
    setAppConfig,
    saveConfig,
    handleSaveAsrConfig,
    handleSaveTextProcessingConfig,
  };
}

function normalizeProviderConfig(config: AppConfig): AppConfig {
  const normalizeRoute = (
    route: ProviderRoutingConfig,
  ): ProviderRoutingConfig => ({
    primary: route.primary?.trim() ?? "",
    fallbacks: route.fallbacks ?? [],
    settings: Object.fromEntries(
      Object.entries(route.settings ?? {}).map(([provider, settings]) => [
        provider,
        {
          ...settings,
          api_key: settings.api_key?.trim() ?? "",
          base_url: settings.base_url?.trim() ?? "",
          model: settings.model?.trim() ?? "",
          options: settings.options ?? {},
        },
      ]),
    ),
  });
  return {
    ...config,
    asr: normalizeRoute(config.asr),
    text: normalizeRoute(config.text),
  };
}

function invalidRouteProvider(
  route: ProviderRoutingConfig,
  allowLocal: boolean,
) {
  for (const provider of [route.primary, ...(route.fallbacks ?? [])]) {
    if (!provider) return "Provider";
    if (allowLocal && provider === "local") continue;
    const settings: ProviderSettings | undefined = route.settings?.[provider];
    if (
      !settings?.api_key?.trim() ||
      !settings.base_url?.trim() ||
      !settings.model?.trim()
    ) {
      return provider;
    }
  }
  return "";
}
