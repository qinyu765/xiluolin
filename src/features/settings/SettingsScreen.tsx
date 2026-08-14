import { useConfigController } from "./useConfigController";
import { SettingsPage } from "@/pages/SettingsPage";

export function SettingsScreen() {
  const config = useConfigController();

  return (
    <SettingsPage
      appConfig={config.appConfig}
      audioDevices={config.audioDevices}
      saveState={config.saveState}
      onConfigChange={config.updateConfig}
      onConfigSync={config.syncConfig}
      onConfigBlur={config.flushConfigSave}
      onRetryConfigSave={config.retryConfigSave}
      configRevision={config.revision}
      historyRevision={0}
    />
  );
}
