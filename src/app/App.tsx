import { useState } from "react";
import { Toaster } from "sonner";

import { HotwordDialog } from "@/components/dialogs/HotwordDialog";
import { PersonaDialog } from "@/components/dialogs/PersonaDialog";
import { HomePage } from "@/pages/HomePage";
import { HotwordPage } from "@/pages/HotwordPage";
import { PersonaPage } from "@/pages/PersonaPage";
import { SettingsPage } from "@/pages/SettingsPage";
import type { Page } from "@/types";

import { AppSidebar } from "./AppSidebar";
import { useConfigController } from "./controllers/useConfigController";
import { useHistoryController } from "./controllers/useHistoryController";
import { useHotwordController } from "./controllers/useHotwordController";
import { usePersonaController } from "./controllers/usePersonaController";
import { useRecordingController } from "./controllers/useRecordingController";

export function App() {
  const [page, setPage] = useState<Page>("home");
  const config = useConfigController();
  const history = useHistoryController();
  const personas = usePersonaController(config.setAppConfig);
  const hotwords = useHotwordController();
  const recording = useRecordingController(history.reload);

  return (
    <main className="flex min-h-screen">
      <Toaster position="top-center" richColors />
      <AppSidebar page={page} onPageChange={setPage} />

      <div className="ml-48 flex-1 overflow-y-auto overflow-x-hidden">
        <div className="mx-auto max-w-4xl px-6 py-8">
          {page === "home" && (
            <HomePage
              personas={personas.personas}
              selectedPersonaId={personas.selectedId}
              selectedPersona={personas.selected}
              isRecording={recording.isRecording}
              isVoiceProcessing={recording.isProcessing}
              recordingDuration={recording.duration}
              voiceStatus={recording.status}
              selectedAudioName={recording.selectedAudioName}
              voiceResult={recording.result}
              historyStats={history.stats}
              historyRecords={history.records}
              historyStatus={history.status}
              appConfig={config.appConfig}
              onPersonaChange={personas.setSelectedId}
              onStartRecording={recording.startRecording}
              onStopRecording={recording.stopRecording}
              onProcessAudio={recording.processAudio}
              onCopyFinalText={recording.copyFinalText}
              onOutputText={recording.outputText}
              onCopyHistoryText={history.copyText}
              onDeleteHistoryRecord={history.deleteRecord}
              onPlayHistoryRecording={history.playRecording}
              onReprocessHistoryAudio={history.reprocessAudio}
              onRefineHistoryText={history.refineText}
            />
          )}

          {page === "persona" && (
            <PersonaPage
              personas={personas.personas}
              status={personas.status}
              onCreatePersona={personas.openCreate}
              onEditPersona={personas.openEdit}
              onDeletePersona={personas.deletePersona}
              onSetDefaultPersona={personas.setDefault}
            />
          )}

          {page === "hotword" && (
            <HotwordPage
              hotwords={hotwords.hotwords}
              hotwordContext={hotwords.context}
              hotwordStatus={hotwords.status}
              enabledHotwordCount={hotwords.enabledCount}
              onCreateHotword={hotwords.openCreate}
              onEditHotword={hotwords.openEdit}
              onDeleteHotword={hotwords.deleteHotword}
              onHotwordEnabledChange={hotwords.setEnabled}
            />
          )}

          {page === "settings" && (
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
              configRevision={config.revision}
              historyRevision={history.revision}
            />
          )}
        </div>
      </div>

      <HotwordDialog
        open={hotwords.isDialogOpen}
        isEditing={hotwords.editingId !== null}
        isSaving={hotwords.isSaving}
        draft={hotwords.draft}
        onOpenChange={hotwords.setDialogOpen}
        onDraftChange={hotwords.setDraft}
        onSave={hotwords.save}
      />

      <PersonaDialog
        open={personas.isDialogOpen}
        isEditing={personas.editingId !== null}
        isSaving={personas.isSaving}
        draft={personas.draft}
        onOpenChange={personas.setDialogOpen}
        onDraftChange={personas.setDraft}
        onSave={personas.save}
      />
    </main>
  );
}
