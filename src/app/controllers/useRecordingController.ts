import { useCallback, useEffect, useReducer, useRef } from "react";
import { toast } from "sonner";

import { commands, events } from "@/generated/tauri-bindings";
import type { RecordingResult, VoiceInputResult } from "@/types";
import { toErrorMessage } from "@/utils/error";

export type RecordingPhase =
  "idle" | "starting" | "recording" | "processing" | "ready" | "error";

type RecordingState = {
  phase: RecordingPhase;
  status: string;
  startedAt: number | null;
  duration: number;
  activeSessionId: string | null;
  selectedAudioName: string;
  result: VoiceInputResult | null;
};

type RecordingAction =
  | { type: "patch"; patch: Partial<RecordingState> }
  | { type: "tick"; duration: number };

const initialState: RecordingState = {
  phase: "idle",
  status: "请选择一段 wav 或 mp3 短音频。",
  startedAt: null,
  duration: 0,
  activeSessionId: null,
  selectedAudioName: "",
  result: null,
};

function reducer(
  state: RecordingState,
  action: RecordingAction,
): RecordingState {
  return action.type === "tick"
    ? { ...state, duration: action.duration }
    : { ...state, ...action.patch };
}

type HistoryReloader = (status: string) => Promise<void>;

function showRecordingStartError(message: string) {
  if (message.includes("麦克风权限")) {
    toast.error("麦克风权限缺失，请在系统设置中开启麦克风权限");
  } else if (message.includes("未找到可用的音频输入设备")) {
    toast.error("未找到麦克风设备，请检查麦克风连接");
  } else {
    toast.error(`录音失败：${message}`);
  }
}

export function useRecordingController(reloadHistory: HistoryReloader) {
  const [state, dispatch] = useReducer(reducer, initialState);
  const inFlightSessions = useRef(new Set<string>());

  useEffect(() => {
    if (state.phase !== "recording" || state.startedAt === null) return;
    const timer = window.setInterval(() => {
      dispatch({ type: "tick", duration: Date.now() - state.startedAt! });
    }, 100);
    return () => window.clearInterval(timer);
  }, [state.phase, state.startedAt]);

  const processCompletedRecording = useCallback(
    async (recording: RecordingResult, expectedSessionId?: string | null) => {
      if (inFlightSessions.current.has(recording.session_id)) return;
      if (expectedSessionId && expectedSessionId !== recording.session_id) {
        await commands
          .abortCaptureSession(recording.session_id)
          .catch(() => undefined);
        const message = "录音会话与当前界面状态不一致";
        dispatch({
          type: "patch",
          patch: { phase: "error", status: message, activeSessionId: null },
        });
        toast.error(message);
        return;
      }

      inFlightSessions.current.add(recording.session_id);
      dispatch({
        type: "patch",
        patch: {
          phase: "processing",
          startedAt: null,
          activeSessionId: recording.session_id,
          status: "录音完成，正在执行 ASR 识别...",
        },
      });

      try {
        const result = await commands.processRecordingFile(
          recording.session_id,
          recording.file_path,
          recording.duration_ms,
        );
        dispatch({ type: "patch", patch: { result } });

        const delivery = await commands.deliverText(
          recording.session_id,
          null,
          result.final_text,
        );
        await reloadHistory(
          result.history_record
            ? "历史记录和统计已更新。"
            : "当前配置关闭了自动保存，本次未写入历史。",
        );

        const status = result.used_text_fallback
          ? "ASR 已完成，文本整理失败，已保留原始识别文本。"
          : delivery.message;
        dispatch({
          type: "patch",
          patch: {
            phase: "ready",
            status,
            activeSessionId: null,
            result,
          },
        });

        if (result.used_text_fallback) {
          toast.warning("文本整理失败，已保留原始识别文本");
        } else if (
          !delivery.success &&
          delivery.message.includes("辅助功能权限")
        ) {
          toast.warning(delivery.message, {
            action: {
              label: "打开设置",
              onClick: () =>
                void commands.openMacosPrivacySettings("accessibility"),
            },
          });
        } else if (delivery.success) {
          toast.success(delivery.message || "语音处理完成");
        } else {
          toast.warning(delivery.message);
        }
      } catch (error) {
        const message = toErrorMessage(error);
        await commands
          .abortCaptureSession(recording.session_id)
          .catch(() => undefined);
        dispatch({
          type: "patch",
          patch: {
            phase: "error",
            status: `录音处理失败：${message}`,
            activeSessionId: null,
          },
        });
        toast.error(`录音处理失败：${message}`);
      } finally {
        inFlightSessions.current.delete(recording.session_id);
      }
    },
    [reloadHistory],
  );

  useEffect(() => {
    let disposed = false;
    let disposeListeners: Array<() => void> = [];

    void Promise.all([
      events.recordingCompleted.listen((event) => {
        void processCompletedRecording(event.payload);
      }),
      events.recordingError.listen((event) => {
        const message = event.payload;
        dispatch({
          type: "patch",
          patch: {
            phase: "error",
            startedAt: null,
            activeSessionId: null,
            status: `录音失败：${message}`,
          },
        });
        showRecordingStartError(message);
      }),
    ]).then((listeners) => {
      if (disposed) listeners.forEach((dispose) => dispose());
      else disposeListeners = listeners;
    });

    return () => {
      disposed = true;
      disposeListeners.forEach((dispose) => dispose());
    };
  }, [processCompletedRecording]);

  const startRecording = async () => {
    try {
      const readiness = await commands.readInputReadiness();
      if (!readiness.can_process) {
        toast.error("语音输入环境未就绪，请前往设置页查看缺失项");
        dispatch({
          type: "patch",
          patch: { phase: "error", status: "麦克风或模型配置未就绪。" },
        });
        return;
      }
    } catch (error) {
      toast.error(`无法检查语音输入环境：${toErrorMessage(error)}`);
      return;
    }

    dispatch({
      type: "patch",
      patch: {
        phase: "starting",
        startedAt: Date.now(),
        duration: 0,
        result: null,
        status: "正在启动录音...",
      },
    });
    try {
      const started = await commands.startRecording();
      dispatch({
        type: "patch",
        patch: {
          phase: "recording",
          activeSessionId: started.session_id,
          status: "正在录音中...",
        },
      });
    } catch (error) {
      const message = toErrorMessage(error);
      dispatch({
        type: "patch",
        patch: {
          phase: "error",
          startedAt: null,
          activeSessionId: null,
          status: `开始录音失败：${message}`,
        },
      });
      showRecordingStartError(message);
    }
  };

  const stopRecording = async () => {
    if (state.phase !== "recording") return;
    const expectedSessionId = state.activeSessionId;
    dispatch({
      type: "patch",
      patch: {
        phase: "processing",
        startedAt: null,
        status: "正在停止录音并处理...",
      },
    });
    try {
      const recording = await commands.stopRecording();
      await processCompletedRecording(recording, expectedSessionId);
    } catch (error) {
      const message = toErrorMessage(error);
      if (expectedSessionId) {
        await commands
          .abortCaptureSession(expectedSessionId)
          .catch(() => undefined);
      }
      dispatch({
        type: "patch",
        patch: {
          phase: "error",
          activeSessionId: null,
          status: `录音处理失败：${message}`,
        },
      });
      toast.error(`录音处理失败：${message}`);
    }
  };

  const processAudio = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    event.target.value = "";
    if (!file) return;

    const extension = file.name.split(".").pop()?.toLowerCase() ?? "";
    if (extension !== "wav" && extension !== "mp3") {
      toast.error("仅支持 wav 或 mp3 短音频");
      dispatch({
        type: "patch",
        patch: { status: "仅支持 wav 或 mp3 短音频。" },
      });
      return;
    }

    try {
      const readiness = await commands.readInputReadiness();
      if (!readiness.models_ready) {
        toast.error("请先完成语音识别和文本 Provider 配置");
        dispatch({
          type: "patch",
          patch: { status: "模型配置不完整，请前往设置页查看就绪检查。" },
        });
        return;
      }
    } catch (error) {
      toast.error(`无法检查模型配置：${toErrorMessage(error)}`);
      return;
    }

    dispatch({
      type: "patch",
      patch: {
        phase: "processing",
        selectedAudioName: file.name,
        result: null,
        status: "正在上传短音频并执行 ASR 识别...",
      },
    });
    try {
      const bytes = Array.from(new Uint8Array(await file.arrayBuffer()));
      const result = await commands.processUploadedAudio({
        audio_bytes: bytes,
        audio_extension: extension,
        duration_ms: 0,
      });
      await reloadHistory(
        result.history_record
          ? "历史记录和统计已更新。"
          : "当前配置关闭了自动保存，本次未写入历史。",
      );
      dispatch({
        type: "patch",
        patch: {
          phase: "ready",
          result,
          status: result.used_text_fallback
            ? "ASR 已完成，文本整理失败，已保留原文作为结果。"
            : "语音主流程已完成，结果可复制使用。",
        },
      });
      toast[result.used_text_fallback ? "warning" : "success"](
        result.used_text_fallback
          ? "文本整理失败，已保留原始识别文本"
          : "语音处理完成",
      );
    } catch (error) {
      const message = toErrorMessage(error);
      dispatch({
        type: "patch",
        patch: { phase: "error", status: `语音主流程失败：${message}` },
      });
      toast.error(`语音处理失败：${message}`);
    }
  };

  const copyFinalText = async () => {
    if (!state.result?.final_text) return;
    try {
      await navigator.clipboard.writeText(state.result.final_text);
      dispatch({
        type: "patch",
        patch: { status: "整理结果已复制到剪贴板。" },
      });
      toast.success("已复制到剪贴板");
    } catch (error) {
      const message = toErrorMessage(error);
      dispatch({ type: "patch", patch: { status: `复制失败：${message}` } });
      toast.error(`复制失败：${message}`);
    }
  };

  const outputText = async () => {
    if (!state.result?.final_text) return;
    dispatch({ type: "patch", patch: { status: "正在输出文本..." } });
    try {
      const delivery = await commands.deliverText(
        null,
        state.result.history_record?.id ?? null,
        state.result.final_text,
      );
      dispatch({ type: "patch", patch: { status: delivery.message } });
      toast[delivery.success ? "success" : "warning"](
        delivery.success
          ? "结果已复制到剪贴板"
          : "自动粘贴失败，已复制到剪贴板，请手动粘贴",
      );
    } catch (error) {
      const message = toErrorMessage(error);
      dispatch({
        type: "patch",
        patch: { status: `输出文本失败：${message}` },
      });
      toast.error(`输出失败：${message}`);
    }
  };

  return {
    phase: state.phase,
    isRecording: state.phase === "recording",
    isProcessing: state.phase === "starting" || state.phase === "processing",
    status: state.status,
    duration: state.duration,
    selectedAudioName: state.selectedAudioName,
    result: state.result,
    startRecording,
    stopRecording,
    processAudio,
    copyFinalText,
    outputText,
  };
}
