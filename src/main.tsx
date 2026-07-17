import React, { useEffect, useMemo, useState } from "react";
import ReactDOM from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { toast, Toaster } from "sonner";
import { BookmarkIcon, HomeIcon, SettingsIcon, UserIcon } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { HomePage } from "@/pages/HomePage";
import { PersonaPage } from "@/pages/PersonaPage";
import { HotwordPage } from "@/pages/HotwordPage";
import { SettingsPage } from "@/pages/SettingsPage";
import { HotwordDialog } from "@/components/dialogs/HotwordDialog";
import { PersonaDialog } from "@/components/dialogs/PersonaDialog";
import type {
  Page,
  Persona,
  Hotword,
  HotwordDraft,
  PersonaDraft,
  AppConfig,
  AudioDevice,
  InputReadiness,
  VoiceInputResult,
  RecordingStartResult,
  RecordingResult,
  DeliveryResult,
  HistoryRecord,
  HistoryStatistics,
} from "@/types";
import { emptyHotwordDraft, emptyPersonaDraft } from "@/types";
import "./styles.css";

function App() {
  const [currentPage, setCurrentPage] = useState<Page>("home");
  const [personas, setPersonas] = useState<Persona[]>([]);
  const [selectedPersonaId, setSelectedPersonaId] = useState("");
  const [appConfig, setAppConfig] = useState<AppConfig | null>(null);
  const [hotwords, setHotwords] = useState<Hotword[]>([]);
  const [historyRecords, setHistoryRecords] = useState<HistoryRecord[]>([]);
  const [historyStats, setHistoryStats] = useState<HistoryStatistics | null>(
    null,
  );
  const [hotwordContext, setHotwordContext] = useState("");
  const [hotwordDraft, setHotwordDraft] =
    useState<HotwordDraft>(emptyHotwordDraft);
  const [editingHotwordId, setEditingHotwordId] = useState<string | null>(null);
  const [isHotwordDialogOpen, setIsHotwordDialogOpen] = useState(false);
  const [personaDraft, setPersonaDraft] =
    useState<PersonaDraft>(emptyPersonaDraft);
  const [editingPersonaId, setEditingPersonaId] = useState<string | null>(null);
  const [isPersonaDialogOpen, setIsPersonaDialogOpen] = useState(false);
  const [status, setStatus] = useState("正在读取本地人格配置...");
  const [asrStatus, setAsrStatus] = useState("正在读取智谱 ASR 配置...");
  const [openaiStatus, setOpenaiStatus] = useState("正在读取 OpenAI 配置...");
  const [hotwordStatus, setHotwordStatus] = useState("正在读取热词词典...");
  const [historyStatus, setHistoryStatus] = useState("正在读取历史记录...");
  const [audioDevices, setAudioDevices] = useState<AudioDevice[]>([]);
  const [voiceStatus, setVoiceStatus] = useState("请选择一段 wav 或 mp3 短音频。");
  const [selectedAudioName, setSelectedAudioName] = useState("");
  const [voiceResult, setVoiceResult] = useState<VoiceInputResult | null>(null);
  const [isSaving, setIsSaving] = useState(false);
  const [isAsrSaving, setIsAsrSaving] = useState(false);
  const [isOpenaiSaving, setIsOpenaiSaving] = useState(false);
  const [isHotwordSaving, setIsHotwordSaving] = useState(false);
  const [isPersonaSaving, setIsPersonaSaving] = useState(false);
  const [isVoiceProcessing, setIsVoiceProcessing] = useState(false);
  const [isRecording, setIsRecording] = useState(false);
  const [recordingStartTime, setRecordingStartTime] = useState<number | null>(null);
  const [recordingDuration, setRecordingDuration] = useState(0);
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);

  const selectedPersona = useMemo(
    () => personas.find((persona) => persona.id === selectedPersonaId),
    [personas, selectedPersonaId],
  );

  const enabledHotwordCount = hotwords.filter(
    (hotword) => hotword.enabled,
  ).length;

  // 录音时长计时器
  useEffect(() => {
    if (!isRecording || recordingStartTime === null) {
      return;
    }

    const interval = setInterval(() => {
      setRecordingDuration(Date.now() - recordingStartTime);
    }, 100);

    return () => clearInterval(interval);
  }, [isRecording, recordingStartTime]);

  useEffect(() => {
    let unlistenCompleted: (() => void) | null = null;
    let unlistenError: (() => void) | null = null;
    let isProcessing = false; // 防止重复处理

    async function initialize() {
      // 加载初始数据
      try {
        const loadedConfig = await invoke<AppConfig>("initialize_local_data");
        const loadedPersonas = await invoke<Persona[]>("list_personas");
        const loadedHotwords = await invoke<Hotword[]>("list_hotwords");
        const loadedContext = await invoke<string>("enabled_hotword_context");
        const loadedHistoryRecords = await invoke<HistoryRecord[]>(
          "list_history_records",
          { limit: 10 },
        );
        const loadedHistoryStats =
          await invoke<HistoryStatistics>("history_statistics");

        let loadedAudioDevices: AudioDevice[] = [];
        try {
          loadedAudioDevices = await invoke<AudioDevice[]>("list_audio_devices");
        } catch (error) {
          console.error("Failed to load audio devices:", error);
        }

        const defaultPersona =
          loadedPersonas.find((persona) => persona.is_default) ??
          loadedPersonas[0];

        setAppConfig(loadedConfig);
        setPersonas(loadedPersonas);
        setSelectedPersonaId(defaultPersona?.id ?? "");
        setHotwords(loadedHotwords);
        setHotwordContext(loadedContext);
        setHistoryRecords(loadedHistoryRecords);
        setHistoryStats(loadedHistoryStats);
        setAudioDevices(loadedAudioDevices);
        setStatus("已加载内置人格，可选择默认整理风格。");
        setAsrStatus("智谱 ASR 配置已加载。");
        setOpenaiStatus("OpenAI 配置已加载。");
        setHotwordStatus("热词词典已加载。");
        setHistoryStatus("历史记录和统计已加载。");
      } catch (error) {
        setStatus(`读取本地数据失败：${String(error)}`);
        setAsrStatus("智谱 ASR 配置读取失败。");
        setOpenaiStatus("OpenAI 配置读取失败。");
        setHotwordStatus("热词词典读取失败。");
        setHistoryStatus("历史记录读取失败。");
      }

      // 监听快捷键录音完成事件
      console.log("正在注册 recording-completed 事件监听器...");
      unlistenCompleted = await listen<RecordingResult>(
        "recording-completed",
        async (event) => {
          // 防止重复处理（React StrictMode 会导致事件监听器注册两次）
          if (isProcessing) {
            console.log("⚠️ 已有处理流程在进行中，跳过此次事件");
            return;
          }
          isProcessing = true;

          const flowStartTime = performance.now();
          console.log("✅ 收到 recording-completed 事件");
          console.log("  - 录音时长:", event.payload.duration_ms, "ms");

          setIsRecording(false);
          setRecordingStartTime(null);
          setIsVoiceProcessing(true);
          setVoiceStatus("录音完成，正在执行 ASR 识别...");

          try {
            // 步骤1: 调用 process_recording_file (ASR + 文本润色)
            const step1Start = performance.now();
            console.log("[⏱️ 前端] 步骤1: 开始调用 process_recording_file...");
            const result = await invoke<VoiceInputResult>("process_recording_file", {
              sessionId: event.payload.session_id,
              filePath: event.payload.file_path,
              durationMs: event.payload.duration_ms,
            });
            const step1Duration = performance.now() - step1Start;
            console.log(`[⏱️ 前端] 步骤1: process_recording_file 完成 - 耗时 ${step1Duration.toFixed(2)}ms`);

            setVoiceResult(result);

            // 步骤2 和 步骤3 并行执行：重新加载历史记录 + 自动输出文本
            const step2Start = performance.now();
            let step2Duration = 0;
            console.log("[⏱️ 前端] 步骤2+3: 并行执行历史记录加载和文本输出...");
            setVoiceStatus("语音处理完成，正在自动输出...");

            try {
              const [
                [loadedHistoryRecords, loadedHistoryStats],
                outputResult
              ] = await Promise.all([
                // 并行任务1: 重新加载历史记录
                Promise.all([
                  invoke<HistoryRecord[]>("list_history_records", { limit: 10 }),
                  invoke<HistoryStatistics>("history_statistics"),
                ]),
                // 并行任务2: 自动输出文本到光标位置
                invoke<DeliveryResult>("deliver_text", {
                  sessionId: event.payload.session_id,
                  text: result.final_text,
                })
              ]);

              step2Duration = performance.now() - step2Start;
              console.log(`[⏱️ 前端] 步骤2+3: 并行任务完成 - 总耗时 ${step2Duration.toFixed(2)}ms`);

              // 更新历史记录
              setHistoryRecords(loadedHistoryRecords);
              setHistoryStats(loadedHistoryStats);
              setHistoryStatus(
                result.history_record
                  ? "历史记录和统计已更新。"
                  : "当前配置关闭了自动保存，本次未写入历史。"
              );

              // 处理输出结果
              console.log("  - 输出方法:", outputResult.method);
              console.log("  - 输出结果:", outputResult.success ? "成功" : "失败");

              setVoiceStatus(outputResult.message);

              if (outputResult.success) {
                if (outputResult.method === "clipboard") {
                  toast.success("已自动输入到光标位置");
                }
              } else {
                toast.warning("自动粘贴失败，已复制到剪贴板，请手动粘贴 (Ctrl+V)");
              }
            } catch (outputError) {
              const errorMessage = String(outputError);
              console.error("❌ 输出文本失败:", errorMessage);
              setVoiceStatus(`输出文本失败：${errorMessage}`);
              toast.error(`输出文本失败：${errorMessage}`);
            }

            if (result.used_text_fallback) {
              console.log("⚠️ 使用了文本降级方案");
              toast.warning("文本整理失败，已保留原始识别文本");
            }

            const totalDuration = performance.now() - flowStartTime;
            console.log(`[⏱️ 前端] ========================================`);
            console.log(`[⏱️ 前端] 完整流程总耗时: ${totalDuration.toFixed(2)}ms`);
            console.log(`[⏱️ 前端]   - 步骤1 (ASR+润色): ${step1Duration.toFixed(2)}ms (${(step1Duration/totalDuration*100).toFixed(1)}%)`);
            console.log(`[⏱️ 前端]   - 步骤2+3 (并行): ${step2Duration.toFixed(2)}ms (${(step2Duration/totalDuration*100).toFixed(1)}%)`);
            console.log(`[⏱️ 前端] ========================================`);
          } catch (error) {
            const errorMessage = String(error);
            await invoke("abort_capture_session", {
              sessionId: event.payload.session_id,
            }).catch(() => undefined);
            console.error("❌ 录音处理失败:", errorMessage);
            setVoiceStatus(`录音处理失败：${errorMessage}`);
            toast.error(`录音处理失败：${errorMessage}`);
          } finally {
            setIsVoiceProcessing(false);
            isProcessing = false; // 重置处理标志
            console.log("录音处理流程结束");
          }
        }
      );
      console.log("✅ recording-completed 事件监听器注册成功");

      // 监听快捷键录音错误事件
      console.log("正在注册 recording-error 事件监听器...");
      unlistenError = await listen<string>("recording-error", (event) => {
        console.log("❌ 收到 recording-error 事件:", event.payload);

        setIsRecording(false);
        setRecordingStartTime(null);
        setIsVoiceProcessing(false);

        const errorMessage = event.payload;
        setVoiceStatus(`录音失败：${errorMessage}`);

        if (errorMessage.includes("麦克风权限")) {
          toast.error("麦克风权限缺失，请在系统设置中开启麦克风权限");
        } else if (errorMessage.includes("未找到可用的音频输入设备")) {
          toast.error("未找到麦克风设备，请检查麦克风连接");
        } else {
          toast.error(`录音失败：${errorMessage}`);
        }
      });
      console.log("✅ recording-error 事件监听器注册成功");
    }

    initialize();
    console.log("🚀 应用初始化完成，事件监听器已就绪");

    return () => {
      console.log("🧹 清理事件监听器...");
      if (unlistenCompleted) {
        unlistenCompleted();
        console.log("✅ recording-completed 监听器已清理");
      }
      if (unlistenError) {
        unlistenError();
        console.log("✅ recording-error 监听器已清理");
      }
    };
  }, []);

  async function handleDefaultPersonaChange(personaId: string) {
    setSelectedPersonaId(personaId);
    setIsSaving(true);
    setStatus("正在保存默认人格...");

    try {
      const updatedPersonas = await invoke<Persona[]>("set_default_persona", {
        personaId,
      });
      const updatedConfig = await invoke<AppConfig>("read_app_config");
      setAppConfig(updatedConfig);
      setPersonas(updatedPersonas);
      setStatus("默认人格已保存。");
    } catch (error) {
      const fallbackPersona = personas.find((persona) => persona.is_default);
      setSelectedPersonaId(fallbackPersona?.id ?? "");
      setStatus(`保存默认人格失败：${String(error)}`);
    } finally {
      setIsSaving(false);
    }
  }

  async function handleSaveAsrConfig(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!appConfig) {
      return;
    }

    const nextConfig = {
      ...appConfig,
      asr_api_key: appConfig.asr_api_key.trim(),
      asr_base_url: appConfig.asr_base_url.trim(),
      asr_model: appConfig.asr_model.trim(),
    };

    if (!nextConfig.asr_base_url || !nextConfig.asr_model) {
      setAsrStatus("Base URL 和模型名不能为空。");
      return;
    }

    setIsAsrSaving(true);
    setAsrStatus("正在保存智谱 ASR 配置...");

    try {
      const savedConfig = await invoke<AppConfig>("update_app_config", {
        config: nextConfig,
      });
      setAppConfig(savedConfig);
      window.dispatchEvent(new Event("xiluolin-config-saved"));
      setAsrStatus(
        savedConfig.asr_api_key
          ? "智谱 ASR 配置已保存。"
          : "智谱 ASR 配置已保存，真实转写前仍需填写 API Key。",
      );
    } catch (error) {
      setAsrStatus(`保存智谱 ASR 配置失败：${String(error)}`);
    } finally {
      setIsAsrSaving(false);
    }
  }

  async function handleSaveOpenaiConfig(
    event: React.FormEvent<HTMLFormElement>,
  ) {
    event.preventDefault();
    if (!appConfig) {
      return;
    }

    const nextConfig = {
      ...appConfig,
      openai_api_key: appConfig.openai_api_key.trim(),
      openai_base_url: appConfig.openai_base_url.trim(),
      openai_model: appConfig.openai_model.trim(),
    };

    if (!nextConfig.openai_base_url || !nextConfig.openai_model) {
      setOpenaiStatus("Base URL 和模型名不能为空。");
      return;
    }

    setIsOpenaiSaving(true);
    setOpenaiStatus("正在保存 OpenAI 配置...");

    try {
      const savedConfig = await invoke<AppConfig>("update_app_config", {
        config: nextConfig,
      });
      setAppConfig(savedConfig);
      window.dispatchEvent(new Event("xiluolin-config-saved"));
      setOpenaiStatus(
        savedConfig.openai_api_key
          ? "OpenAI 配置已保存。"
          : "OpenAI 配置已保存，真实整理前仍需填写 API Key。",
      );
    } catch (error) {
      setOpenaiStatus(`保存 OpenAI 配置失败：${String(error)}`);
    } finally {
      setIsOpenaiSaving(false);
    }
  }

  async function reloadHotwords(nextStatus: string) {
    const [loadedHotwords, loadedContext] = await Promise.all([
      invoke<Hotword[]>("list_hotwords"),
      invoke<string>("enabled_hotword_context"),
    ]);
    setHotwords(loadedHotwords);
    setHotwordContext(loadedContext);
    setHotwordStatus(nextStatus);
  }

  async function reloadHistoryData(nextStatus: string) {
    const [loadedHistoryRecords, loadedHistoryStats] = await Promise.all([
      invoke<HistoryRecord[]>("list_history_records", { limit: 10 }),
      invoke<HistoryStatistics>("history_statistics"),
    ]);
    setHistoryRecords(loadedHistoryRecords);
    setHistoryStats(loadedHistoryStats);
    setHistoryStatus(nextStatus);
  }

  async function handleDeleteHistoryRecord(id: string) {
    setHistoryStatus("正在删除历史记录...");

    try {
      await invoke("delete_history_record", { id });
      await reloadHistoryData("历史记录已删除。");
      toast.success("历史记录已删除");
    } catch (error) {
      const errorMessage = String(error);
      setHistoryStatus(`删除历史记录失败: ${errorMessage}`);
      toast.error(`删除失败: ${errorMessage}`);
    }
  }

  function openCreateHotwordDialog() {
    setEditingHotwordId(null);
    setHotwordDraft(emptyHotwordDraft);
    setIsHotwordDialogOpen(true);
  }

  function openEditHotwordDialog(hotword: Hotword) {
    setEditingHotwordId(hotword.id);
    setHotwordDraft({
      text: hotword.text,
      category: hotword.category,
      enabled: hotword.enabled,
    });
    setIsHotwordDialogOpen(true);
  }

  async function handleSaveHotword(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const draft = {
      ...hotwordDraft,
      text: hotwordDraft.text.trim(),
      category: hotwordDraft.category.trim(),
    };

    if (!draft.text) {
      setHotwordStatus("热词不能为空。");
      return;
    }

    setIsHotwordSaving(true);
    setHotwordStatus("正在保存热词...");

    try {
      if (editingHotwordId) {
        await invoke<Hotword>("update_hotword", {
          id: editingHotwordId,
          draft,
        });
      } else {
        await invoke<Hotword>("create_hotword", { draft });
      }
      await reloadHotwords("热词已保存，并会进入文本整理上下文。");
      setIsHotwordDialogOpen(false);
    } catch (error) {
      setHotwordStatus(`保存热词失败：${String(error)}`);
    } finally {
      setIsHotwordSaving(false);
    }
  }

  async function handleHotwordEnabledChange(hotword: Hotword, enabled: boolean) {
    setHotwordStatus("正在更新热词状态...");

    try {
      await invoke<Hotword>("update_hotword", {
        id: hotword.id,
        draft: {
          text: hotword.text,
          category: hotword.category,
          enabled,
        },
      });
      await reloadHotwords(enabled ? "热词已启用。" : "热词已停用。");
    } catch (error) {
      setHotwordStatus(`更新热词状态失败：${String(error)}`);
    }
  }

  async function handleDeleteHotword(id: string) {
    setHotwordStatus("正在删除热词...");

    try {
      const updatedHotwords = await invoke<Hotword[]>("delete_hotword", { id });
      const updatedContext = await invoke<string>("enabled_hotword_context");
      setHotwords(updatedHotwords);
      setHotwordContext(updatedContext);
      setHotwordStatus("热词已删除。");
    } catch (error) {
      setHotwordStatus(`删除热词失败：${String(error)}`);
    }
  }

  function openCreatePersonaDialog() {
    setEditingPersonaId(null);
    setPersonaDraft(emptyPersonaDraft);
    setIsPersonaDialogOpen(true);
  }

  function openEditPersonaDialog(persona: Persona) {
    setEditingPersonaId(persona.id);
    setPersonaDraft({
      name: persona.name,
      description: persona.description,
      icon: persona.icon,
    });
    setIsPersonaDialogOpen(true);
  }

  async function handleSavePersona(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const draft = {
      ...personaDraft,
      name: personaDraft.name.trim(),
      description: personaDraft.description.trim(),
      icon: personaDraft.icon.trim(),
    };

    if (!draft.name || !draft.description) {
      setStatus("人格名称和描述不能为空。");
      return;
    }

    setIsPersonaSaving(true);
    setStatus("正在保存人格...");

    try {
      if (editingPersonaId) {
        await invoke<Persona>("update_persona", {
          id: editingPersonaId,
          draft,
        });
      } else {
        await invoke<Persona>("create_persona", { draft });
      }
      const updatedPersonas = await invoke<Persona[]>("list_personas");
      setPersonas(updatedPersonas);
      setStatus("人格已保存。");
      setIsPersonaDialogOpen(false);
    } catch (error) {
      setStatus(`保存人格失败：${String(error)}`);
    } finally {
      setIsPersonaSaving(false);
    }
  }

  async function handleDeletePersona(id: string) {
    setStatus("正在删除人格...");

    try {
      const updatedPersonas = await invoke<Persona[]>("delete_persona", { id });
      setPersonas(updatedPersonas);
      setStatus("人格已删除。");
    } catch (error) {
      setStatus(`删除人格失败：${String(error)}`);
    }
  }

  async function handleSetDefaultPersona(personaId: string) {
    setStatus("正在设置默认人格...");

    try {
      const updatedPersonas = await invoke<Persona[]>("set_default_persona", {
        personaId,
      });
      const updatedConfig = await invoke<AppConfig>("read_app_config");
      setAppConfig(updatedConfig);
      setPersonas(updatedPersonas);
      setSelectedPersonaId(personaId);
      setStatus("默认人格已设置。");
    } catch (error) {
      setStatus(`设置默认人格失败：${String(error)}`);
    }
  }

  async function handleProcessAudio(event: React.ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    event.target.value = "";
    if (!file) {
      return;
    }

    const extension = file.name.split(".").pop()?.toLowerCase() ?? "";
    if (extension !== "wav" && extension !== "mp3") {
      toast.error("仅支持 wav 或 mp3 短音频");
      setVoiceStatus("仅支持 wav 或 mp3 短音频。");
      return;
    }

    try {
      const readiness = await invoke<InputReadiness>("read_input_readiness");
      if (!readiness.models_ready) {
        toast.error("请先完成语音识别和文本 Provider 配置");
        setVoiceStatus("模型配置不完整，请前往设置页查看就绪检查。");
        return;
      }
    } catch (error) {
      toast.error(`无法检查模型配置：${String(error)}`);
      return;
    }

    setIsVoiceProcessing(true);
    setSelectedAudioName(file.name);
    setVoiceResult(null);
    setVoiceStatus("正在上传短音频并执行 ASR 识别...");

    try {
      const audioBuffer = await file.arrayBuffer();
      const audioBytes = Array.from(new Uint8Array(audioBuffer));
      const result = await invoke<VoiceInputResult>("process_uploaded_audio", {
        request: {
          audio_bytes: audioBytes,
          audio_extension: extension,
          duration_ms: 0,
        },
      });
      setVoiceResult(result);
      await reloadHistoryData(
        result.history_record
          ? "历史记录和统计已更新。"
          : "当前配置关闭了自动保存，本次未写入历史。",
      );
      setVoiceStatus(
        result.used_text_fallback
          ? "ASR 已完成，OpenAI 整理失败，已保留原文作为结果。"
          : "语音主流程已完成，结果可复制使用。",
      );
      if (result.used_text_fallback) {
        toast.warning("文本整理失败，已保留原始识别文本");
      } else {
        toast.success("语音处理完成");
      }
    } catch (error) {
      const errorMessage = String(error);
      setVoiceStatus(`语音主流程失败：${errorMessage}`);
      toast.error(`语音处理失败：${errorMessage}`);
    } finally {
      setIsVoiceProcessing(false);
    }
  }

  async function handleCopyFinalText() {
    if (!voiceResult?.final_text) {
      return;
    }

    try {
      await navigator.clipboard.writeText(voiceResult.final_text);
      setVoiceStatus("整理结果已复制到剪贴板。");
      toast.success("已复制到剪贴板");
    } catch (error) {
      const errorMessage = String(error);
      setVoiceStatus(`复制失败：${errorMessage}`);
      toast.error(`复制失败：${errorMessage}`);
    }
  }

  async function handleCopyHistoryText(text: string) {
    try {
      await navigator.clipboard.writeText(text);
      setHistoryStatus("历史记录已复制到剪贴板。");
      toast.success("已复制到剪贴板");
    } catch (error) {
      const errorMessage = String(error);
      setHistoryStatus(`复制失败：${errorMessage}`);
      toast.error(`复制失败：${errorMessage}`);
    }
  }

  async function handleStartRecording() {
    try {
      const readiness = await invoke<InputReadiness>("read_input_readiness");
      if (!readiness.can_process) {
        toast.error("语音输入环境未就绪，请前往设置页查看缺失项");
        setVoiceStatus("麦克风或模型配置未就绪。");
        return;
      }
    } catch (error) {
      toast.error(`无法检查语音输入环境：${String(error)}`);
      return;
    }

    setIsRecording(true);
    setRecordingStartTime(Date.now());
    setRecordingDuration(0);
    setVoiceResult(null);
    setVoiceStatus("正在录音中...");

    try {
      const started = await invoke<RecordingStartResult>("start_recording");
      setActiveSessionId(started.session_id);
    } catch (error) {
      const errorMessage = String(error);
      setIsRecording(false);
      setRecordingStartTime(null);
      setActiveSessionId(null);
      setVoiceStatus(`开始录音失败：${errorMessage}`);

      // 根据错误类型显示不同的提示
      if (errorMessage.includes("麦克风权限")) {
        toast.error("麦克风权限缺失，请在系统设置中开启麦克风权限");
      } else if (errorMessage.includes("未找到可用的音频输入设备")) {
        toast.error("未找到麦克风设备，请检查麦克风连接");
      } else {
        toast.error(`录音失败：${errorMessage}`);
      }
    }
  }

  async function handleStopRecording() {
    if (!isRecording) {
      return;
    }

    setIsRecording(false);
    setRecordingStartTime(null);
    setIsVoiceProcessing(true);
    setVoiceStatus("正在停止录音并处理...");

    try {
      const recordingResult = await invoke<RecordingResult>("stop_recording");
      if (activeSessionId && activeSessionId !== recordingResult.session_id) {
        throw new Error("录音会话与当前界面状态不一致");
      }
      setVoiceStatus("录音完成，正在执行 ASR 识别...");

      // 使用新的命令处理录音文件
      const result = await invoke<VoiceInputResult>("process_recording_file", {
        sessionId: recordingResult.session_id,
        filePath: recordingResult.file_path,
        durationMs: recordingResult.duration_ms,
      });

      setVoiceResult(result);
      const delivery = await invoke<DeliveryResult>("deliver_text", {
        sessionId: recordingResult.session_id,
        text: result.final_text,
      });
      setActiveSessionId(null);
      await reloadHistoryData(
        result.history_record
          ? "历史记录和统计已更新。"
          : "当前配置关闭了自动保存，本次未写入历史。",
      );
      setVoiceStatus(
        result.used_text_fallback
          ? "ASR 已完成，文本整理失败，已复制原始识别文本。"
          : delivery.message,
      );
      if (result.used_text_fallback) {
        toast.warning("文本整理失败，已保留原始识别文本");
      } else {
        toast.success("语音处理完成");
      }
    } catch (error) {
      const errorMessage = String(error);
      if (activeSessionId) {
        await invoke("abort_capture_session", {
          sessionId: activeSessionId,
        }).catch(() => undefined);
      }
      setActiveSessionId(null);
      setVoiceStatus(`录音处理失败：${errorMessage}`);
      toast.error(`录音处理失败：${errorMessage}`);
    } finally {
      setIsVoiceProcessing(false);
    }
  }

  async function handleOutputText() {
    if (!voiceResult?.final_text) {
      return;
    }

    setVoiceStatus("正在输出文本...");

    try {
      const result = await invoke<DeliveryResult>("deliver_text", {
        sessionId: null,
        text: voiceResult.final_text,
      });
      setVoiceStatus(result.message);

      if (result.success) {
        toast.success("结果已复制到剪贴板");
      } else {
        toast.warning("自动粘贴失败，已复制到剪贴板，请手动粘贴 (Ctrl+V)");
      }
    } catch (error) {
      const errorMessage = String(error);
      setVoiceStatus(`输出文本失败：${errorMessage}`);
      toast.error(`输出失败：${errorMessage}`);
    }
  }

  return (
    <main className="flex min-h-screen">
      <Toaster position="top-center" richColors />

      {/* 左侧导航栏 - 固定定位 */}
      <Tabs
        value={currentPage}
        onValueChange={(value) => setCurrentPage(value as Page)}
        orientation="vertical"
        className="fixed left-0 top-0 z-10 flex h-screen w-48 flex-col border-r bg-muted/30"
      >
        {/* 标题区域 */}
        <div className="border-b px-6 py-4">
          <h1 className="text-2xl font-semibold tracking-normal [font-family:Georgia,'Times_New_Roman',serif]">
            XiLuoLin
          </h1>
        </div>

        {/* 导航菜单 */}
        <TabsList className="flex h-auto w-full flex-col items-stretch gap-1 rounded-none bg-transparent p-2">
          <TabsTrigger
            value="home"
            className="justify-start gap-2 rounded-md data-[state=active]:bg-background"
          >
            <HomeIcon className="size-4" aria-hidden="true" />
            首页
          </TabsTrigger>
          <TabsTrigger
            value="persona"
            className="justify-start gap-2 rounded-md data-[state=active]:bg-background"
          >
            <UserIcon className="size-4" aria-hidden="true" />
            人格
          </TabsTrigger>
          <TabsTrigger
            value="hotword"
            className="justify-start gap-2 rounded-md data-[state=active]:bg-background"
          >
            <BookmarkIcon className="size-4" aria-hidden="true" />
            热词
          </TabsTrigger>
          <TabsTrigger
            value="settings"
            className="justify-start gap-2 rounded-md data-[state=active]:bg-background"
          >
            <SettingsIcon className="size-4" aria-hidden="true" />
            设置
          </TabsTrigger>
        </TabsList>
      </Tabs>

      {/* 右侧内容区 - 添加左边距 */}
      <div className="ml-48 flex-1 overflow-y-auto overflow-x-hidden">
          <div className="mx-auto max-w-4xl px-6 py-8">
            {currentPage === "home" && (
              <HomePage
                personas={personas}
                selectedPersonaId={selectedPersonaId}
                selectedPersona={selectedPersona}
                isRecording={isRecording}
                isVoiceProcessing={isVoiceProcessing}
                recordingDuration={recordingDuration}
                voiceStatus={voiceStatus}
                selectedAudioName={selectedAudioName}
                voiceResult={voiceResult}
                historyStats={historyStats}
                historyRecords={historyRecords}
                historyStatus={historyStatus}
                appConfig={appConfig}
                onPersonaChange={setSelectedPersonaId}
                onStartRecording={handleStartRecording}
                onStopRecording={handleStopRecording}
                onProcessAudio={handleProcessAudio}
                onCopyFinalText={handleCopyFinalText}
                onOutputText={handleOutputText}
                onCopyHistoryText={handleCopyHistoryText}
                onDeleteHistoryRecord={handleDeleteHistoryRecord}
              />
            )}

            {currentPage === "persona" && (
              <PersonaPage
                personas={personas}
                status={status}
                onCreatePersona={openCreatePersonaDialog}
                onEditPersona={openEditPersonaDialog}
                onDeletePersona={handleDeletePersona}
                onSetDefaultPersona={handleSetDefaultPersona}
              />
            )}

            {currentPage === "hotword" && (
              <HotwordPage
                hotwords={hotwords}
                hotwordContext={hotwordContext}
                hotwordStatus={hotwordStatus}
                enabledHotwordCount={enabledHotwordCount}
                onCreateHotword={openCreateHotwordDialog}
                onEditHotword={openEditHotwordDialog}
                onDeleteHotword={handleDeleteHotword}
                onHotwordEnabledChange={handleHotwordEnabledChange}
              />
            )}

            {currentPage === "settings" && (
              <SettingsPage
                appConfig={appConfig}
                audioDevices={audioDevices}
                asrStatus={asrStatus}
                openaiStatus={openaiStatus}
                isAsrSaving={isAsrSaving}
                isOpenaiSaving={isOpenaiSaving}
                onSaveAsrConfig={handleSaveAsrConfig}
                onSaveOpenaiConfig={handleSaveOpenaiConfig}
                onConfigChange={setAppConfig}
                onConfigSaved={setAppConfig}
              />
            )}
          </div>
        </div>

      <HotwordDialog
        open={isHotwordDialogOpen}
        isEditing={editingHotwordId !== null}
        isSaving={isHotwordSaving}
        draft={hotwordDraft}
        onOpenChange={setIsHotwordDialogOpen}
        onDraftChange={setHotwordDraft}
        onSave={handleSaveHotword}
      />

      <PersonaDialog
        open={isPersonaDialogOpen}
        isEditing={editingPersonaId !== null}
        isSaving={isPersonaSaving}
        draft={personaDraft}
        onOpenChange={setIsPersonaDialogOpen}
        onDraftChange={setPersonaDraft}
        onSave={handleSavePersona}
      />
    </main>
  );
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <App />,
);
