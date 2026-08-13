import { useConfigController } from "./useConfigController";
import { SettingsPage } from "@/pages/SettingsPage";

export function SettingsScreen() {
  const config = useConfigController();

  return (
    <SettingsPage
      appConfig={config.appConfig}
      audioDevices={config.audioDevices}
      asrStatus={config.asrStatus}
      textProcessingStatus={config.textProcessingStatus}
      isAsrSaving={config.isAsrSaving}
      isTextProcessingSaving={config.isTextProcessingSaving}
      onSaveAsrConfig={config.handleSaveAsrConfig}
      onSaveTextProcessingConfig={config.handleSaveTextProcessingConfig}
      onConfigChange={config.setAppConfig}
      onSaveConfig={config.saveConfig}
    />
  );
}
