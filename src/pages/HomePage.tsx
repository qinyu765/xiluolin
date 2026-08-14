import { HomeGreetingCard } from "@/components/home/HomeGreetingCard";
import { VoiceInputStatsCard } from "@/components/home/VoiceInputStatsCard";
import type {
  AppConfig,
  HistoryRecord,
  HistoryStatistics,
  Persona,
} from "@/types";
import { formatCreatedAt, formatDuration } from "@/utils/format";
import { groupHistoryByDate } from "@/utils/date";

type HomePageProps = {
  selectedPersona: Persona | undefined;
  historyStats: HistoryStatistics | null;
  historyRecords: HistoryRecord[];
  appConfig: AppConfig | null;
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

      <VoiceInputStatsCard
        historyStats={historyStats}
        historyRecords={historyRecords}
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
