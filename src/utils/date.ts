import type { HistoryRecord, GroupedHistory } from "@/types";

export function isSameDay(date1: Date, date2: Date): boolean {
  return (
    date1.getFullYear() === date2.getFullYear() &&
    date1.getMonth() === date2.getMonth() &&
    date1.getDate() === date2.getDate()
  );
}

export function formatDateKey(date: Date): string {
  const month = date.getMonth() + 1;
  const day = date.getDate();
  return `${month}月${day}日`;
}

export function groupHistoryByDate(records: HistoryRecord[]): GroupedHistory {
  const today = new Date();
  const yesterday = new Date(today);
  yesterday.setDate(yesterday.getDate() - 1);

  const todayRecords: HistoryRecord[] = [];
  const yesterdayRecords: HistoryRecord[] = [];
  const olderRecords: Map<string, HistoryRecord[]> = new Map();

  records.forEach((record) => {
    const recordDate = new Date(record.created_at);
    if (isSameDay(recordDate, today)) {
      todayRecords.push(record);
    } else if (isSameDay(recordDate, yesterday)) {
      yesterdayRecords.push(record);
    } else {
      const dateKey = formatDateKey(recordDate);
      if (!olderRecords.has(dateKey)) {
        olderRecords.set(dateKey, []);
      }
      olderRecords.get(dateKey)!.push(record);
    }
  });

  return { todayRecords, yesterdayRecords, olderRecords };
}
