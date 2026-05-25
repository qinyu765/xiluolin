import React from "react";
import { QuickStartCard } from "@/components/home/QuickStartCard";
import { VoiceInputStatsCard } from "@/components/home/VoiceInputStatsCard";
import type {
  Persona,
  VoiceInputResult,
  HistoryRecord,
  HistoryStatistics,
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
      <QuickStartCard
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
      />

      <VoiceInputStatsCard
        historyStats={historyStats}
        historyRecords={historyRecords}
        historyStatus={historyStatus}
        onCopyHistoryText={onCopyHistoryText}
        onDeleteHistoryRecord={onDeleteHistoryRecord}
        formatDuration={formatDuration}
        formatCreatedAt={formatCreatedAt}
        groupHistoryByDate={groupHistoryByDate}
      />
    </div>
  );
}
