import { useCallback, useEffect, useState } from "react";
import { toast } from "sonner";

import { commands } from "@/generated/tauri-bindings";
import type { HistoryRecord, HistoryStatistics } from "@/types";
import { toErrorMessage } from "@/utils/error";

export function useHistoryController() {
  const [records, setRecords] = useState<HistoryRecord[]>([]);
  const [stats, setStats] = useState<HistoryStatistics | null>(null);
  const [status, setStatus] = useState("正在读取历史记录...");
  const [revision, setRevision] = useState(0);

  const reload = useCallback(async (nextStatus: string) => {
    const [nextRecords, nextStats] = await Promise.all([
      commands.listHistoryRecords(10),
      commands.historyStatistics(),
    ]);
    setRecords(nextRecords);
    setStats(nextStats);
    setStatus(nextStatus);
    setRevision((value) => value + 1);
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
        setStatus("历史记录和统计已加载。");
      })
      .catch((error) => {
        if (active) setStatus(`历史记录读取失败：${toErrorMessage(error)}`);
      });
    return () => {
      active = false;
    };
  }, []);

  const copyText = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setStatus("历史记录已复制到剪贴板。");
      toast.success("已复制到剪贴板");
    } catch (error) {
      const message = toErrorMessage(error);
      setStatus(`复制失败：${message}`);
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
    setStatus("正在使用当前模型重新转写...");
    try {
      await commands.reprocessHistoryAudio(id);
      await reload("录音已使用当前模型重新转写。后续复制会使用新结果。");
      toast.success("重新转写完成");
    } catch (error) {
      const message = toErrorMessage(error);
      setStatus(`重新转写失败：${message}`);
      toast.error(`重新转写失败：${message}`);
    }
  };

  const refineText = async (id: string) => {
    setStatus("正在使用当前人格重新整理...");
    try {
      await commands.refineHistoryText(id);
      await reload("历史文本已使用当前人格重新整理。原始识别文本保持不变。");
      toast.success("重新整理完成");
    } catch (error) {
      const message = toErrorMessage(error);
      setStatus(`重新整理失败：${message}`);
      toast.error(`重新整理失败：${message}`);
    }
  };

  const deleteRecord = async (id: string) => {
    setStatus("正在删除历史记录...");
    try {
      await commands.deleteHistoryRecord(id);
      await reload("历史记录已删除。");
      toast.success("历史记录已删除");
    } catch (error) {
      const message = toErrorMessage(error);
      setStatus(`删除历史记录失败：${message}`);
      toast.error(`删除失败：${message}`);
    }
  };

  return {
    records,
    stats,
    status,
    revision,
    reload,
    copyText,
    playRecording,
    reprocessAudio,
    refineText,
    deleteRecord,
  };
}
