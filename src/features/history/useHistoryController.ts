import { useCallback, useEffect, useState } from "react";
import { toast } from "sonner";

import { commands, events } from "@/generated/tauri-bindings";
import type { HistoryRecord, HistoryStatistics } from "@/types";
import { toErrorMessage } from "@/utils/error";

export function useHistoryController() {
  const [records, setRecords] = useState<HistoryRecord[]>([]);
  const [stats, setStats] = useState<HistoryStatistics | null>(null);

  const reload = useCallback(async () => {
    const [nextRecords, nextStats] = await Promise.all([
      commands.listHistoryRecords(10),
      commands.historyStatistics(),
    ]);
    setRecords(nextRecords);
    setStats(nextStats);
  }, []);

  useEffect(() => {
    let active = true;
    void Promise.all([
      commands.listHistoryRecords(10),
      commands.historyStatistics(),
    ])
      .then(([nextRecords, nextStats]) => {
        if (!active) return;
        setRecords(nextRecords);
        setStats(nextStats);
      })
      .catch((error) => {
        if (active) toast.error(`历史记录读取失败：${toErrorMessage(error)}`);
      });
    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    let dispose: (() => void) | undefined;
    let disposed = false;
    void events.historyChanged
      .listen(() => {
        void reload();
      })
      .then((nextDispose) => {
        if (disposed) nextDispose();
        else dispose = nextDispose;
      });
    return () => {
      disposed = true;
      dispose?.();
    };
  }, [reload]);

  const copyText = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      toast.success("已复制到剪贴板");
    } catch (error) {
      const message = toErrorMessage(error);
      toast.error(`复制失败：${message}`);
    }
  };

  const playRecording = async (id: string) => {
    try {
      const bytes = await commands.readRetainedRecording(id);
      const url = URL.createObjectURL(
        new Blob([new Uint8Array(bytes)], { type: "audio/wav" }),
      );
      const audio = new Audio(url);
      audio.addEventListener("ended", () => URL.revokeObjectURL(url), {
        once: true,
      });
      try {
        await audio.play();
      } catch (error) {
        URL.revokeObjectURL(url);
        throw error;
      }
      toast.success("正在播放保留录音");
    } catch (error) {
      toast.error(`播放录音失败：${toErrorMessage(error)}`);
    }
  };

  const reprocessAudio = async (id: string) => {
    try {
      await commands.reprocessHistoryAudio(id);
      await reload();
      toast.success("重新转写完成");
    } catch (error) {
      const message = toErrorMessage(error);
      toast.error(`重新转写失败：${message}`);
    }
  };

  const refineText = async (id: string) => {
    try {
      await commands.refineHistoryText(id);
      await reload();
      toast.success("重新整理完成");
    } catch (error) {
      const message = toErrorMessage(error);
      toast.error(`重新整理失败：${message}`);
    }
  };

  const deleteRecord = async (id: string) => {
    try {
      await commands.deleteHistoryRecord(id);
      await reload();
      toast.success("历史记录已删除");
    } catch (error) {
      const message = toErrorMessage(error);
      toast.error(`删除失败：${message}`);
    }
  };

  return {
    records,
    stats,
    reload,
    copyText,
    playRecording,
    reprocessAudio,
    refineText,
    deleteRecord,
  };
}
