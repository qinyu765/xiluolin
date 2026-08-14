import { useConfigController } from "@/features/settings/useConfigController";
import { useHistoryController } from "./useHistoryController";
import { usePersonaController } from "@/features/persona/usePersonaController";
import { HomePage } from "@/pages/HomePage";

export function HomeDashboard() {
  const config = useConfigController();
  const history = useHistoryController();
  const personas = usePersonaController(config.setAppConfig);

  return (
    <HomePage
      selectedPersona={personas.selected}
      historyStats={history.stats}
      historyRecords={history.records}
      appConfig={config.appConfig}
      onCopyHistoryText={history.copyText}
      onDeleteHistoryRecord={history.deleteRecord}
      onPlayHistoryRecording={history.playRecording}
      onReprocessHistoryAudio={history.reprocessAudio}
      onRefineHistoryText={history.refineText}
    />
  );
}
