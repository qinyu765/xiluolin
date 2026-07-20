import React from "react";
import {
  BarChart3Icon,
  Clock3Icon,
  CopyIcon,
  HistoryIcon,
  Mic2Icon,
  PencilIcon,
  PlayIcon,
  RefreshCwIcon,
  Trash2Icon,
  WandSparklesIcon,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardAction,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import type { AppConfig, HistoryRecord, HistoryStatistics } from "@/types";
import { formatShortcutDisplay } from "@/utils/shortcut";

type VoiceInputStatsCardProps = {
  historyStats: HistoryStatistics | null;
  historyRecords: HistoryRecord[];
  historyStatus: string;
  appConfig: AppConfig | null;
  onCopyHistoryText: (text: string) => void;
  onDeleteHistoryRecord: (id: string) => void;
  onPlayHistoryRecording: (id: string) => void;
  onReprocessHistoryAudio: (id: string) => void;
  onRefineHistoryText: (id: string) => void;
  formatDuration: (ms: number) => string;
  formatCreatedAt: (createdAt: string) => string;
  groupHistoryByDate: (records: HistoryRecord[]) => {
    todayRecords: HistoryRecord[];
    yesterdayRecords: HistoryRecord[];
    olderRecords: Map<string, HistoryRecord[]>;
  };
};

export function VoiceInputStatsCard({
  historyStats,
  historyRecords,
  historyStatus,
  appConfig,
  onCopyHistoryText,
  onDeleteHistoryRecord,
  onPlayHistoryRecording,
  onReprocessHistoryAudio,
  onRefineHistoryText,
  formatDuration,
  formatCreatedAt,
  groupHistoryByDate,
}: VoiceInputStatsCardProps) {
  const getShortcutHint = () => {
    const longpress = appConfig?.longpress_shortcut;
    const toggle = appConfig?.toggle_shortcut;

    if (longpress) {
      const display = formatShortcutDisplay(longpress);
      return `按住 ${display} 开始语音输入`;
    }

    if (toggle) {
      const display = formatShortcutDisplay(toggle);
      return `按 ${display} 开始/停止语音输入`;
    }

    return "请前往设置页配置快捷键";
  };

  return (
    <Card>
      <CardHeader>
        <div>
          <CardTitle className="text-xl">{getShortcutHint()}</CardTitle>
        </div>
        <CardAction>
          <span className="inline-flex h-8 items-center rounded-md bg-secondary px-3 text-xs font-medium text-secondary-foreground">
            {historyStats?.total_count ?? 0} 次记录
          </span>
        </CardAction>
      </CardHeader>

      <CardContent className="space-y-5">
        <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-5">
          <section className="rounded-lg border bg-muted/30 p-4">
            <BarChart3Icon
              className="mb-3 size-4 text-primary"
              aria-hidden="true"
            />
            <p className="text-xs text-muted-foreground">语音协作次数</p>
            <p className="mt-1 text-2xl font-semibold">
              {historyStats?.total_count ?? 0}
            </p>
          </section>
          <section className="rounded-lg border bg-muted/30 p-4">
            <Clock3Icon
              className="mb-3 size-4 text-primary"
              aria-hidden="true"
            />
            <p className="text-xs text-muted-foreground">累计口述时间</p>
            <p className="mt-1 text-2xl font-semibold">
              {formatDuration(historyStats?.total_duration_ms ?? 0)}
            </p>
          </section>
          <section className="rounded-lg border bg-muted/30 p-4">
            <PencilIcon
              className="mb-3 size-4 text-primary"
              aria-hidden="true"
            />
            <p className="text-xs text-muted-foreground">口述生成字数</p>
            <p className="mt-1 text-2xl font-semibold">
              {historyStats?.total_output_chars ?? 0}
            </p>
          </section>
          <section className="rounded-lg border bg-muted/30 p-4">
            <HistoryIcon
              className="mb-3 size-4 text-primary"
              aria-hidden="true"
            />
            <p className="text-xs text-muted-foreground">预计节省时间</p>
            <p className="mt-1 text-2xl font-semibold">
              {formatDuration(historyStats?.estimated_saved_ms ?? 0)}
            </p>
          </section>
          <section className="rounded-lg border bg-muted/30 p-4 sm:col-span-2 lg:col-span-1">
            <Mic2Icon className="mb-3 size-4 text-primary" aria-hidden="true" />
            <p className="text-xs text-muted-foreground">常用人格</p>
            <p className="mt-1 truncate text-lg font-semibold">
              {historyStats?.top_persona_name ?? "暂无"}
            </p>
            {historyStats?.top_persona_name ? (
              <p className="mt-1 text-xs text-muted-foreground">
                使用 {historyStats.top_persona_count} 次
              </p>
            ) : null}
          </section>
        </div>

        <div className="grid gap-3 border-t pt-4">
          <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
            <p className="text-sm leading-6 text-muted-foreground">
              {historyStatus}
            </p>
            <span className="inline-flex h-8 w-fit items-center rounded-md bg-secondary px-3 text-xs font-medium text-secondary-foreground">
              最近 {historyRecords.length} 条
            </span>
          </div>

          {historyRecords.length > 0 ? (
            (() => {
              const { todayRecords, yesterdayRecords, olderRecords } =
                groupHistoryByDate(historyRecords);
              return (
                <div className="grid gap-4">
                  {todayRecords.length > 0 && (
                    <div className="grid gap-3">
                      <h3 className="text-sm font-semibold text-foreground">
                        今天
                      </h3>
                      {todayRecords.map((record) => (
                        <HistoryRecordItem
                          key={record.id}
                          record={record}
                          onCopy={onCopyHistoryText}
                          onDelete={onDeleteHistoryRecord}
                          onPlay={onPlayHistoryRecording}
                          onReprocess={onReprocessHistoryAudio}
                          onRefine={onRefineHistoryText}
                          formatCreatedAt={formatCreatedAt}
                          formatDuration={formatDuration}
                        />
                      ))}
                    </div>
                  )}

                  {yesterdayRecords.length > 0 && (
                    <div className="grid gap-3">
                      <h3 className="text-sm font-semibold text-foreground">
                        昨天
                      </h3>
                      {yesterdayRecords.map((record) => (
                        <HistoryRecordItem
                          key={record.id}
                          record={record}
                          onCopy={onCopyHistoryText}
                          onDelete={onDeleteHistoryRecord}
                          onPlay={onPlayHistoryRecording}
                          onReprocess={onReprocessHistoryAudio}
                          onRefine={onRefineHistoryText}
                          formatCreatedAt={formatCreatedAt}
                          formatDuration={formatDuration}
                        />
                      ))}
                    </div>
                  )}

                  {Array.from(olderRecords.entries()).map(
                    ([dateKey, records]) => (
                      <div key={dateKey} className="grid gap-3">
                        <h3 className="text-sm font-semibold text-foreground">
                          {dateKey}
                        </h3>
                        {records.map((record) => (
                          <HistoryRecordItem
                            key={record.id}
                            record={record}
                            onCopy={onCopyHistoryText}
                            onDelete={onDeleteHistoryRecord}
                            onPlay={onPlayHistoryRecording}
                            onReprocess={onReprocessHistoryAudio}
                            onRefine={onRefineHistoryText}
                            formatCreatedAt={formatCreatedAt}
                            formatDuration={formatDuration}
                          />
                        ))}
                      </div>
                    ),
                  )}
                </div>
              );
            })()
          ) : (
            <section className="rounded-lg border border-dashed bg-muted/20 p-5 text-sm leading-6 text-muted-foreground">
              暂无历史记录。完成一次短音频输入后,这里会展示最近结果和统计数据。
            </section>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

type HistoryRecordItemProps = {
  record: HistoryRecord;
  onCopy: (text: string) => void;
  onDelete: (id: string) => void;
  onPlay: (id: string) => void;
  onReprocess: (id: string) => void;
  onRefine: (id: string) => void;
  formatCreatedAt: (createdAt: string) => string;
  formatDuration: (ms: number) => string;
};

function HistoryRecordItem({
  record,
  onCopy,
  onDelete,
  onPlay,
  onReprocess,
  onRefine,
  formatCreatedAt,
  formatDuration,
}: HistoryRecordItemProps) {
  return (
    <section className="grid gap-3 rounded-lg border bg-background p-4">
      <div className="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
        <div className="min-w-0 flex-1">
          <p className="text-sm font-semibold">{record.persona_name}</p>
          <p className="mt-1 text-xs text-muted-foreground">
            {formatCreatedAt(record.created_at)} ·{" "}
            {formatDuration(record.duration_ms)} · {record.output_chars} 字 ·{" "}
            {record.source === "upload" ? "上传" : "录音"}
          </p>
          <p className="mt-1 text-xs text-muted-foreground">
            {record.asr_provider || "未知 ASR"}/{record.asr_model || "未知模型"}
            {record.audio_path ? " · 已保留录音" : ""}
            {record.used_asr_fallback ? " · 使用云端 ASR 降级" : ""}
            {record.used_fallback ? " · 使用文本降级" : ""}
          </p>
        </div>
        <div className="flex items-center gap-1">
          {record.audio_path && (
            <>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="h-7 px-2"
                onClick={() => onPlay(record.id)}
                title="试听保留录音"
                aria-label="试听保留录音"
              >
                <PlayIcon className="size-3.5" aria-hidden="true" />
              </Button>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="h-7 px-2"
                onClick={() => onReprocess(record.id)}
                title="使用当前模型重新转写"
                aria-label="使用当前模型重新转写"
              >
                <RefreshCwIcon className="size-3.5" aria-hidden="true" />
              </Button>
            </>
          )}
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className="h-7 px-2"
            onClick={() => onRefine(record.id)}
            title="使用当前人格重新整理"
            aria-label="使用当前人格重新整理"
          >
            <WandSparklesIcon className="size-3.5" aria-hidden="true" />
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className="h-7 px-2"
            onClick={() => onCopy(record.final_text)}
          >
            <CopyIcon className="size-3.5" aria-hidden="true" />
          </Button>
          <Button
            type="button"
            variant="ghost"
            size="sm"
            className="h-7 px-2 text-muted-foreground hover:text-destructive"
            onClick={() => onDelete(record.id)}
          >
            <Trash2Icon className="size-3.5" aria-hidden="true" />
          </Button>
        </div>
      </div>
      <p className="line-clamp-3 text-sm leading-6 text-muted-foreground">
        {record.final_text}
      </p>
    </section>
  );
}
