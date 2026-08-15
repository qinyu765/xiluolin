import { useCallback, useEffect, useRef, useState } from "react";

import {
  prepareSettingsConfig,
  type SettingsSaveMode,
  validateSettingsConfig,
} from "@/components/settings/settings-schema";
import { commands } from "@/generated/tauri-bindings";
import type { AppConfig, AudioDevice } from "@/types";

import {
  createConfigSaveQueue,
  type ConfigSaveState,
} from "@/app/controllers/config-save-queue";

export function useConfigController() {
  const [appConfig, setAppConfig] = useState<AppConfig | null>(null);
  const configRef = useRef<AppConfig | null>(null);
  const [audioDevices, setAudioDevices] = useState<AudioDevice[]>([]);
  const [saveState, setSaveState] = useState<ConfigSaveState>({
    status: "idle",
  });
  const [revision, setRevision] = useState(0);
  const [savedResult, setSavedResult] = useState<{
    config: AppConfig;
    isLatest: boolean;
  } | null>(null);
  const [saveQueue] = useState(() =>
    createConfigSaveQueue<AppConfig>({
      save: async (config) =>
        (await commands.updateAppConfig(config)) as AppConfig,
      prepare: prepareSettingsConfig,
      validate: validateSettingsConfig,
      onStateChange: setSaveState,
      onSaved: (config, isLatest) => setSavedResult({ config, isLatest }),
    }),
  );

  useEffect(() => {
    configRef.current = appConfig;
  }, [appConfig]);

  useEffect(() => {
    if (!savedResult) return;
    if (savedResult.isLatest) {
      configRef.current = savedResult.config;
      setAppConfig(savedResult.config);
    }
    setRevision((value) => value + 1);
  }, [savedResult]);

  useEffect(() => {
    let active = true;
    void Promise.all([
      commands.initializeLocalData(),
      commands.listAudioDevices().catch(() => []),
    ])
      .then(([config, devices]) => {
        if (!active) return;
        const loadedConfig = config as AppConfig;
        configRef.current = loadedConfig;
        setAppConfig(loadedConfig);
        setAudioDevices(devices);
      })
      .catch(() => undefined);
    return () => {
      active = false;
      // 离开设置页前先冲刷防抖队列，避免 600ms 内的最后一次修改丢失。
      void saveQueue.flush();
      saveQueue.dispose();
    };
  }, [saveQueue]);

  const updateConfig = useCallback(
    (patch: Partial<AppConfig>, saveMode: SettingsSaveMode) => {
      if (!configRef.current) return;
      const next = { ...configRef.current, ...patch };
      configRef.current = next;
      setAppConfig(next);
      saveQueue.update(next, saveMode);
    },
    [saveQueue],
  );

  const replaceAppConfig = useCallback((config: AppConfig) => {
    configRef.current = config;
    setAppConfig(config);
  }, []);

  const syncConfig = useCallback((patch: Partial<AppConfig>) => {
    if (!configRef.current) return;
    const next = { ...configRef.current, ...patch };
    configRef.current = next;
    setAppConfig(next);
  }, []);

  const flushConfigSave = useCallback(() => {
    void saveQueue.flush();
  }, [saveQueue]);

  const retryConfigSave = useCallback(() => {
    void saveQueue.retry();
  }, [saveQueue]);

  return {
    appConfig,
    audioDevices,
    saveState,
    revision,
    setAppConfig: replaceAppConfig,
    syncConfig,
    updateConfig,
    flushConfigSave,
    retryConfigSave,
  };
}
