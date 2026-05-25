import React from "react";
import { QuickStartCard } from "@/components/home/QuickStartCard";
import { VoiceInputStatsCard } from "@/components/home/VoiceInputStatsCard";
import type {
  Persona,
  VoiceInputResult,
  HistoryRecord,
  HistoryStatistics,
  AppConfig,
} from "@/types";
import { formatDuration, formatCreatedAt } from "@/utils/format";
import { groupHistoryByDate } from "@/utils/date";

type HomePageProps = {
  personas: Persona[];
  selectedPersonaId: string;
  selectedPersona: Persona | undefined;
  isRecording: boolean;
  isVoiceProcessing: boolean;
  recordingDuration: number;
  voiceStatus: string;
  selectedAudioName: string;
  voiceResult: VoiceInputResult | null;
  historyStats: HistoryStatistics | null;
  historyRecords: HistoryRecord[];
  historyStatus: string;
  appConfig: AppConfig | null;
  onPersonaChange: (personaId: string) => void;
  onStartRecording: () => void;
  onStopRecording: () => void;
  onProcessAudio: (event: React.ChangeEvent<HTMLInputElement>) => void;
  onCopyFinalText: () => void;
  onOutputText: () => void;
  onCopyHistoryText: (text: string) => void;
  onDeleteHistoryRecord: (id: string) => void;
};

export function HomePage({
  personas,
  selectedPersonaId,
  selectedPersona,
  isRecording,
  isVoiceProcessing,
  recordingDuration,
  voiceStatus,
  selectedAudioName,
  voiceResult,
  historyStats,
  historyRecords,
  historyStatus,
  appConfig,
  onPersonaChange,
  onStartRecording,
  onStopRecording,
  onProcessAudio,
  onCopyFinalText,
  onOutputText,
  onCopyHistoryText,
  onDeleteHistoryRecord,
}: HomePageProps) {
  return (
    <div className="space-y-6">
      {/* 问候语 */}
      <div className="rounded-lg border bg-card p-6">
        <h2 className="text-2xl font-medium">
          Hi, 当前人格是{selectedPersona?.name || "未选择"}
        </h2>
        {selectedPersona?.description && (
          <p className="mt-2 text-sm text-muted-foreground">
            {selectedPersona.description}
          </p>
        )}
      </div>

      {/* 快速开始 - 暂时隐藏，不符合当前产品定位，保留以备后用 */}
      {/* <QuickStartCard
        personas={personas}
        selectedPersonaId={selectedPersonaId}
        selectedPersona={selectedPersona}
        isRecording={isRecording}
        isVoiceProcessing={isVoiceProcessing}
        recordingDuration={recordingDuration}
        voiceStatus={voiceStatus}
        selectedAudioName={selectedAudioName}
        voiceResult={voiceResult}
        onPersonaChange={onPersonaChange}
        onStartRecording={onStartRecording}
        onStopRecording={onStopRecording}
        onProcessAudio={onProcessAudio}
        onCopyFinalText={onCopyFinalText}
        onOutputText={onOutputText}
        formatDuration={formatDuration}
      /> */}

      <VoiceInputStatsCard
        historyStats={historyStats}
        historyRecords={historyRecords}
        historyStatus={historyStatus}
        appConfig={appConfig}
        onCopyHistoryText={onCopyHistoryText}
        onDeleteHistoryRecord={onDeleteHistoryRecord}
        formatDuration={formatDuration}
        formatCreatedAt={formatCreatedAt}
        groupHistoryByDate={groupHistoryByDate}
      />
    </div>
  );
}
