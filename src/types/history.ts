import type {
  HistoryRecord,
  HistoryStatistics,
} from "@/generated/tauri-bindings";

export type { HistoryRecord, HistoryStatistics };

export type GroupedHistory = {
  todayRecords: HistoryRecord[];
  yesterdayRecords: HistoryRecord[];
  olderRecords: Map<string, HistoryRecord[]>;
};
