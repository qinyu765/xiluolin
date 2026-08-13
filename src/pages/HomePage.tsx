import { VoiceInputStatsCard } from "@/components/home/VoiceInputStatsCard";
import { HomeReadinessCard } from "@/features/capture/HomeReadinessCard";
import type {
  Persona,
  HistoryRecord,
  HistoryStatistics,
  AppConfig,
} from "@/types";
import { formatDuration, formatCreatedAt } from "@/utils/format";
import { groupHistoryByDate } from "@/utils/date";

type HomePageProps = {
  selectedPersona: Persona | undefined;
  historyStats: HistoryStatistics | null;
  historyRecords: HistoryRecord[];
  historyStatus: string;
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
      <header>
        <p className="text-xs font-semibold uppercase tracking-[0.16em] text-primary">
          Background dictation
        </p>
        <h1 className="mt-2 text-3xl font-semibold tracking-tight">
          语音输入工作台
        </h1>
        <p className="mt-2 text-sm text-muted-foreground">
          无需停留在窗口中，使用全局快捷键即可开始口述。
        </p>
      </header>

      <HomeReadinessCard appConfig={appConfig} persona={selectedPersona} />

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
