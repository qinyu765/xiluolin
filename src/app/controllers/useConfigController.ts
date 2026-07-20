import { useEffect, useState } from "react";

import { commands } from "@/generated/tauri-bindings";
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
  const [revision, setRevision] = useState(0);

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
    setRevision((value) => value + 1);
    return saved;
  };

  const handleSaveAsrConfig = async (
    event: React.FormEvent<HTMLFormElement>,
  ) => {
    event.preventDefault();
    if (!appConfig) return;

    const nextConfig: AppConfig = {
      ...appConfig,
      asr_api_key: appConfig.asr_api_key.trim(),
      asr_base_url: appConfig.asr_base_url.trim(),
      asr_model: appConfig.asr_model.trim(),
      openai_api_key: appConfig.openai_api_key.trim(),
      openai_base_url: appConfig.openai_base_url.trim(),
      openai_asr_model: appConfig.openai_asr_model.trim(),
    };
    const selectedBaseUrl =
      nextConfig.asr_provider === "local"
        ? "local"
        : nextConfig.asr_provider === "openai"
          ? nextConfig.openai_base_url
          : nextConfig.asr_base_url;
    const selectedModel =
      nextConfig.asr_provider === "local"
        ? nextConfig.local_asr_model
        : nextConfig.asr_provider === "openai"
          ? nextConfig.openai_asr_model
          : nextConfig.asr_model;

    if (!selectedBaseUrl || !selectedModel) {
      setAsrStatus("当前 ASR Provider 的 Base URL 和模型名不能为空。");
      return;
    }

    setIsAsrSaving(true);
    setAsrStatus("正在保存 ASR 配置...");
    try {
      const saved = await saveConfig(nextConfig);
      const apiKey =
        saved.asr_provider === "local"
          ? "local"
          : saved.asr_provider === "openai"
            ? saved.openai_api_key
            : saved.asr_api_key;
      const label =
        saved.asr_provider === "local"
          ? "本地 Whisper"
          : saved.asr_provider === "openai"
            ? "OpenAI"
            : "智谱";
      setAsrStatus(
        apiKey
          ? `${label} ASR 配置已保存。`
          : "ASR 配置已保存，真实转写前仍需填写 API Key。",
      );
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

    const nextConfig: AppConfig = {
      ...appConfig,
      zhipu_api_key: appConfig.zhipu_api_key.trim(),
      zhipu_base_url: appConfig.zhipu_base_url.trim(),
      zhipu_model: appConfig.zhipu_model.trim(),
      openai_api_key: appConfig.openai_api_key.trim(),
      openai_base_url: appConfig.openai_base_url.trim(),
      openai_model: appConfig.openai_model.trim(),
    };
    const usesZhipu = nextConfig.text_provider === "zhipu";
    const label = usesZhipu ? "智谱" : "OpenAI 兼容";
    const apiKey = usesZhipu
      ? nextConfig.zhipu_api_key
      : nextConfig.openai_api_key;
    const baseUrl = usesZhipu
      ? nextConfig.zhipu_base_url
      : nextConfig.openai_base_url;
    const model = usesZhipu ? nextConfig.zhipu_model : nextConfig.openai_model;

    if (!baseUrl || !model) {
      setTextProcessingStatus("当前文本处理服务的 Base URL 和模型名不能为空。");
      return;
    }

    setIsTextProcessingSaving(true);
    setTextProcessingStatus(`正在保存${label}文本处理配置...`);
    try {
      await saveConfig(nextConfig);
      setTextProcessingStatus(
        apiKey
          ? `${label}文本处理配置已保存。`
          : `${label}文本处理配置已保存，真实整理前仍需填写 API Key。`,
      );
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
    revision,
    setAppConfig,
    saveConfig,
    handleSaveAsrConfig,
    handleSaveTextProcessingConfig,
  };
}
