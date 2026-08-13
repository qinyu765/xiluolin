import React from "react";
import { HomeGreetingCard } from "@/components/home/HomeGreetingCard";
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
  onPlayHistoryRecording: (id: string) => void;
  onReprocessHistoryAudio: (id: string) => void;
  onRefineHistoryText: (id: string) => void;
};

export function HomePage({
  selectedPersona,
  historyStats,
  historyRecords,
  historyStatus,
  appConfig,
  onCopyHistoryText,
  onDeleteHistoryRecord,
  onPlayHistoryRecording,
  onReprocessHistoryAudio,
  onRefineHistoryText,
}: HomePageProps) {
  return (
    <div className="space-y-6">
      <HomeGreetingCard
        personaName={selectedPersona?.name}
        personaDescription={selectedPersona?.description}
        longpressShortcut={appConfig?.longpress_shortcut}
        toggleShortcut={appConfig?.toggle_shortcut}
      />

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
        onCopyHistoryText={onCopyHistoryText}
        onDeleteHistoryRecord={onDeleteHistoryRecord}
        onPlayHistoryRecording={onPlayHistoryRecording}
        onReprocessHistoryAudio={onReprocessHistoryAudio}
        onRefineHistoryText={onRefineHistoryText}
        formatDuration={formatDuration}
        formatCreatedAt={formatCreatedAt}
        groupHistoryByDate={groupHistoryByDate}
      />
    </div>
  );
}
