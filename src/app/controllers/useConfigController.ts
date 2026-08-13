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
} from "./config-save-queue";

export function useConfigController() {
  const [appConfig, setAppConfig] = useState<AppConfig | null>(null);
  const configRef = useRef<AppConfig | null>(null);
  const [audioDevices, setAudioDevices] = useState<AudioDevice[]>([]);
  const [saveState, setSaveState] = useState<ConfigSaveState>({
    status: "idle",
  });
  const [revision, setRevision] = useState(0);
  const [saveQueue] = useState(() =>
    createConfigSaveQueue<AppConfig>({
      save: async (config) =>
        (await commands.updateAppConfig(config)) as AppConfig,
      prepare: prepareSettingsConfig,
      validate: validateSettingsConfig,
      onStateChange: setSaveState,
      onSaved: (saved, isLatest) => {
        if (isLatest) setAppConfig(saved);
        setRevision((value) => value + 1);
      },
    }),
  );

  useEffect(() => {
    configRef.current = appConfig;
  }, [appConfig]);

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
    updateConfig,
    flushConfigSave,
    retryConfigSave,
  };
}
