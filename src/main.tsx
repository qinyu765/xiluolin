import React, { useEffect, useMemo, useState } from "react";
import ReactDOM from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import { toast, Toaster } from "sonner";
import {
  BarChart3Icon,
  Clock3Icon,
  CopyIcon,
  FileAudioIcon,
  HistoryIcon,
  Loader2Icon,
  Mic2Icon,
  PencilIcon,
  PlusIcon,
  SaveIcon,
  SettingsIcon,
  Trash2Icon,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import "./styles.css";

type Persona = {
  id: string;
  name: string;
  description: string;
  scene: string;
  tone: string;
  output_structure: string;
  prompt: string;
  is_builtin: boolean;
  is_default: boolean;
};

type Hotword = {
  id: string;
  source_text: string;
  target_text: string;
  category: string;
  enabled: boolean;
};

type HotwordDraft = {
  source_text: string;
  target_text: string;
  category: string;
  enabled: boolean;
};

type PersonaDraft = {
  name: string;
  description: string;
  scene: string;
  tone: string;
  output_structure: string;
  prompt: string;
};

type AppConfig = {
  default_persona_id: string;
  asr_api_key: string;
  asr_base_url: string;
  asr_model: string;
  openai_api_key: string;
  openai_base_url: string;
  openai_model: string;
  recording_mode: string;
  longpress_shortcut: string;
  toggle_shortcut: string;
  output_mode: string;
  auto_save_history: boolean;
  mute_system_audio: boolean;
  selected_microphone: string;
};

type AudioDevice = {
  name: string;
  is_default: boolean;
};

type VoiceInputResult = {
  raw_text: string;
  final_text: string;
  used_text_fallback: boolean;
  history_record: HistoryRecord | null;
};

type HistoryRecord = {
  id: string;
  raw_text: string;
  final_text: string;
  persona_id: string;
  persona_name: string;
  duration_ms: number;
  output_chars: number;
  output_mode: string;
  created_at: string;
};

type HistoryStatistics = {
  total_count: number;
  total_duration_ms: number;
  total_output_chars: number;
  estimated_saved_ms: number;
  top_persona_name: string | null;
  top_persona_count: number;
};

const emptyHotwordDraft: HotwordDraft = {
  source_text: "",
  target_text: "",
  category: "",
  enabled: true,
};

const emptyPersonaDraft: PersonaDraft = {
  name: "",
  description: "",
  scene: "",
  tone: "",
  output_structure: "",
  prompt: "",
};

function formatDuration(milliseconds: number) {
  const totalSeconds = Math.max(0, Math.round(milliseconds / 1000));
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;

  if (minutes === 0) {
    return `${seconds} 秒`;
  }

  return `${minutes} 分 ${seconds} 秒`;
}

function formatCreatedAt(createdAt: string) {
  return createdAt.replace("T", " ").replace(/\.\d+Z?$/, "");
}

function App() {
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
  const [isSettingsDialogOpen, setIsSettingsDialogOpen] = useState(false);
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
    async function loadData() {
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
    }

    loadData();
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

  function openCreateHotwordDialog() {
    setEditingHotwordId(null);
    setHotwordDraft(emptyHotwordDraft);
    setIsHotwordDialogOpen(true);
  }

  function openEditHotwordDialog(hotword: Hotword) {
    setEditingHotwordId(hotword.id);
    setHotwordDraft({
      source_text: hotword.source_text,
      target_text: hotword.target_text,
      category: hotword.category,
      enabled: hotword.enabled,
    });
    setIsHotwordDialogOpen(true);
  }

  async function handleSaveHotword(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const draft = {
      ...hotwordDraft,
      source_text: hotwordDraft.source_text.trim(),
      target_text: hotwordDraft.target_text.trim(),
      category: hotwordDraft.category.trim(),
    };

    if (!draft.source_text || !draft.target_text) {
      setHotwordStatus("原始说法和修正写法不能为空。");
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
          source_text: hotword.source_text,
          target_text: hotword.target_text,
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
      scene: persona.scene,
      tone: persona.tone,
      output_structure: persona.output_structure,
      prompt: persona.prompt,
    });
    setIsPersonaDialogOpen(true);
  }

  async function handleSavePersona(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const draft = {
      ...personaDraft,
      name: personaDraft.name.trim(),
      description: personaDraft.description.trim(),
      scene: personaDraft.scene.trim(),
      tone: personaDraft.tone.trim(),
      output_structure: personaDraft.output_structure.trim(),
      prompt: personaDraft.prompt.trim(),
    };

    if (!draft.name || !draft.description || !draft.prompt) {
      setStatus("人格名称、描述和提示词不能为空。");
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

    // 检查 API Key 配置
    if (!appConfig?.asr_api_key || !appConfig?.openai_api_key) {
      toast.error("请先在设置页配置 API Key");
      setVoiceStatus("未配置 API Key，请前往设置页配置。");
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
    // 检查 API Key 配置
    if (!appConfig?.asr_api_key || !appConfig?.openai_api_key) {
      toast.error("请先在设置页配置 API Key");
      setVoiceStatus("未配置 API Key，请前往设置页配置。");
      return;
    }

    setIsRecording(true);
    setRecordingStartTime(Date.now());
    setRecordingDuration(0);
    setVoiceResult(null);
    setVoiceStatus("正在录音中...");

    try {
      await invoke<string>("start_recording");
    } catch (error) {
      const errorMessage = String(error);
      setIsRecording(false);
      setRecordingStartTime(null);
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
      const recordingResult = await invoke<{ file_path: string; duration_ms: number }>("stop_recording");
      setVoiceStatus("录音完成，正在执行 ASR 识别...");

      // 使用新的命令处理录音文件
      const result = await invoke<VoiceInputResult>("process_recording_file", {
        filePath: recordingResult.file_path,
        durationMs: recordingResult.duration_ms,
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
      const result = await invoke<{ method: string; success: boolean; message: string }>("output_text", {
        text: voiceResult.final_text,
      });
      setVoiceStatus(result.message);

      if (result.success) {
        if (result.method === "keyboard") {
          toast.success("已自动输入到光标位置");
        } else if (result.method === "clipboard") {
          toast.success("已通过剪贴板输入");
        }
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
    <main className="min-h-screen px-4 py-8 sm:px-6 lg:px-8">
      <Toaster position="top-center" richColors />
      <div className="mx-auto grid min-h-[calc(100vh-4rem)] w-full max-w-4xl content-center gap-6">
        <section className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="inline-flex items-center gap-2 rounded-md border bg-card px-3 py-1 text-sm font-medium text-muted-foreground shadow-sm">
              <Mic2Icon className="size-4 text-primary" aria-hidden="true" />
              AI 语音输入助手
            </div>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={() => setIsSettingsDialogOpen(true)}
            >
              <SettingsIcon className="size-4" aria-hidden="true" />
              设置
            </Button>
          </div>
          <div className="space-y-3">
            <h1 className="text-5xl font-semibold tracking-normal text-balance [font-family:Georgia,'Times_New_Roman',serif] sm:text-6xl">
              XiLuoLin
            </h1>
            <p className="max-w-2xl text-lg leading-8 text-muted-foreground">
              面向办公、写作和编程场景，把短语音整理成可直接使用的文本。
            </p>
          </div>
        </section>

        <Card>
          <CardHeader>
            <div>
              <p className="mb-2 text-xs font-semibold tracking-normal text-primary uppercase">
                T015 主界面
              </p>
              <CardTitle className="text-2xl">语音输入</CardTitle>
              <CardDescription className="mt-2">
                点击录音按钮开始说话，再次点击停止录音并自动处理。支持上传音频文件测试。
              </CardDescription>
            </div>
            <CardAction>
              <span className="inline-flex h-8 items-center rounded-md bg-secondary px-3 text-xs font-medium text-secondary-foreground">
                {isRecording ? "录音中" : isVoiceProcessing ? "处理中" : "就绪"}
              </span>
            </CardAction>
          </CardHeader>

          <CardContent className="space-y-5">
            {/* 人格选择 */}
            <div className="space-y-2">
              <Label htmlFor="main-persona-select">当前人格</Label>
              <Select
                value={selectedPersonaId}
                onValueChange={setSelectedPersonaId}
                disabled={isRecording || isVoiceProcessing || personas.length === 0}
              >
                <SelectTrigger id="main-persona-select" className="h-10">
                  <SelectValue placeholder="选择人格" />
                </SelectTrigger>
                <SelectContent>
                  {personas.map((persona) => (
                    <SelectItem key={persona.id} value={persona.id}>
                      {persona.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
              {selectedPersona ? (
                <p className="text-xs text-muted-foreground">
                  {selectedPersona.description}
                </p>
              ) : null}
            </div>

            {/* 录音控制区 */}
            <div className="grid gap-3 rounded-lg border border-dashed bg-muted/20 p-5">
              <div className="flex flex-col items-center gap-4">
                <Button
                  type="button"
                  size="lg"
                  variant={isRecording ? "destructive" : "default"}
                  className="h-16 w-16 rounded-full"
                  onClick={isRecording ? handleStopRecording : handleStartRecording}
                  disabled={isVoiceProcessing}
                >
                  {isRecording ? (
                    <div className="size-6 rounded-sm bg-white" />
                  ) : (
                    <Mic2Icon className="size-6" aria-hidden="true" />
                  )}
                </Button>
                <div className="text-center">
                  <p className="text-sm font-medium">
                    {isRecording
                      ? `录音中 ${formatDuration(recordingDuration)}`
                      : isVoiceProcessing
                        ? "处理中..."
                        : "点击开始录音"}
                  </p>
                  <p className="mt-1 text-xs text-muted-foreground">
                    {voiceStatus}
                  </p>
                </div>
              </div>

              {/* 或上传音频文件 */}
              <div className="flex items-center gap-3 border-t pt-3">
                <div className="h-px flex-1 bg-border" />
                <span className="text-xs text-muted-foreground">或上传音频文件</span>
                <div className="h-px flex-1 bg-border" />
              </div>

              <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                <div className="min-w-0">
                  <p className="text-sm font-medium">选择短音频文件</p>
                  <p className="mt-1 truncate text-sm text-muted-foreground">
                    {selectedAudioName || "尚未选择文件"}
                  </p>
                </div>
                <Button
                  type="button"
                  size="sm"
                  disabled={isRecording || isVoiceProcessing}
                  asChild
                >
                  <Label htmlFor="voice-audio-file" className="cursor-pointer">
                    {isVoiceProcessing ? (
                      <Loader2Icon className="size-4 animate-spin" aria-hidden="true" />
                    ) : (
                      <FileAudioIcon className="size-4" aria-hidden="true" />
                    )}
                    选择音频
                  </Label>
                </Button>
              </div>
              <Input
                id="voice-audio-file"
                type="file"
                accept=".wav,.mp3,audio/wav,audio/mpeg"
                className="hidden"
                onChange={handleProcessAudio}
                disabled={isRecording || isVoiceProcessing}
              />
            </div>

            {voiceResult ? (
              <div className="grid gap-4">
                <section className="grid gap-2">
                  <Label htmlFor="voice-raw-text">原始识别文本</Label>
                  <Textarea
                    id="voice-raw-text"
                    value={voiceResult.raw_text}
                    readOnly
                    className="min-h-24 resize-none bg-background text-sm"
                  />
                </section>

                <section className="grid gap-2">
                  <div className="flex items-center justify-between gap-3">
                    <Label htmlFor="voice-final-text">整理结果</Label>
                    <div className="flex gap-2">
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={handleCopyFinalText}
                      >
                        <CopyIcon className="size-4" aria-hidden="true" />
                        复制
                      </Button>
                      <Button
                        type="button"
                        size="sm"
                        onClick={handleOutputText}
                      >
                        <SaveIcon className="size-4" aria-hidden="true" />
                        输出
                      </Button>
                    </div>
                  </div>
                  <Textarea
                    id="voice-final-text"
                    value={voiceResult.final_text}
                    readOnly
                    className="min-h-36 resize-none bg-background text-sm"
                  />
                </section>
              </div>
            ) : null}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <div>
              <p className="mb-2 text-xs font-semibold tracking-normal text-primary uppercase">
                T009 历史与统计
              </p>
              <CardTitle className="text-2xl">语音输入成效</CardTitle>
              <CardDescription className="mt-2">
                基于本地历史记录展示协作次数、口述时间、生成字数、预计节省时间和常用人格。
              </CardDescription>
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
                <BarChart3Icon className="mb-3 size-4 text-primary" aria-hidden="true" />
                <p className="text-xs text-muted-foreground">语音协作次数</p>
                <p className="mt-1 text-2xl font-semibold">
                  {historyStats?.total_count ?? 0}
                </p>
              </section>
              <section className="rounded-lg border bg-muted/30 p-4">
                <Clock3Icon className="mb-3 size-4 text-primary" aria-hidden="true" />
                <p className="text-xs text-muted-foreground">累计口述时间</p>
                <p className="mt-1 text-2xl font-semibold">
                  {formatDuration(historyStats?.total_duration_ms ?? 0)}
                </p>
              </section>
              <section className="rounded-lg border bg-muted/30 p-4">
                <PencilIcon className="mb-3 size-4 text-primary" aria-hidden="true" />
                <p className="text-xs text-muted-foreground">口述生成字数</p>
                <p className="mt-1 text-2xl font-semibold">
                  {historyStats?.total_output_chars ?? 0}
                </p>
              </section>
              <section className="rounded-lg border bg-muted/30 p-4">
                <HistoryIcon className="mb-3 size-4 text-primary" aria-hidden="true" />
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
                <div className="grid gap-3">
                  {historyRecords.map((record) => (
                    <section
                      key={record.id}
                      className="grid gap-3 rounded-lg border bg-background p-4"
                    >
                      <div className="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
                        <div className="min-w-0 flex-1">
                          <p className="text-sm font-semibold">
                            {record.persona_name}
                          </p>
                          <p className="mt-1 text-xs text-muted-foreground">
                            {formatCreatedAt(record.created_at)} ·{" "}
                            {formatDuration(record.duration_ms)} ·{" "}
                            {record.output_chars} 字
                          </p>
                        </div>
                        <div className="flex items-center gap-2">
                          <span className="inline-flex h-7 w-fit items-center rounded-md border bg-muted/30 px-2.5 text-xs text-muted-foreground">
                            {record.output_mode === "paste" ? "自动粘贴" : "复制"}
                          </span>
                          <Button
                            type="button"
                            variant="ghost"
                            size="sm"
                            className="h-7 px-2"
                            onClick={() => handleCopyHistoryText(record.final_text)}
                          >
                            <CopyIcon className="size-3.5" aria-hidden="true" />
                          </Button>
                        </div>
                      </div>
                      <p className="line-clamp-3 text-sm leading-6 text-muted-foreground">
                        {record.final_text}
                      </p>
                    </section>
                  ))}
                </div>
              ) : (
                <section className="rounded-lg border border-dashed bg-muted/20 p-5 text-sm leading-6 text-muted-foreground">
                  暂无历史记录。完成一次短音频输入后，这里会展示最近结果和统计数据。
                </section>
              )}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <div>
              <p className="mb-2 text-xs font-semibold tracking-normal text-primary uppercase">
                T017 人格页
              </p>
              <CardTitle className="text-2xl">人格管理</CardTitle>
              <CardDescription className="mt-2">
                管理内置人格和自定义人格，设置默认人格。内置人格不可编辑或删除。
              </CardDescription>
            </div>
            <CardAction>
              <Button type="button" size="sm" onClick={openCreatePersonaDialog}>
                <PlusIcon className="size-4" aria-hidden="true" />
                新建人格
              </Button>
            </CardAction>
          </CardHeader>

          <CardContent className="space-y-5">
            <div className="space-y-3">
              <h3 className="text-sm font-semibold">内置人格</h3>
              <div className="grid gap-3">
                {personas.filter((p) => p.is_builtin).length > 0 ? (
                  personas
                    .filter((p) => p.is_builtin)
                    .map((persona) => (
                      <section
                        key={persona.id}
                        className="grid gap-3 rounded-lg border bg-muted/30 p-4"
                      >
                        <div className="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
                          <div className="min-w-0 flex-1">
                            <div className="flex items-center gap-2">
                              <p className="text-sm font-semibold">
                                {persona.name}
                              </p>
                              {persona.is_default ? (
                                <span className="inline-flex h-6 items-center rounded-md border bg-background px-2 text-xs font-medium">
                                  默认
                                </span>
                              ) : null}
                            </div>
                            <p className="mt-1 text-sm leading-6 text-muted-foreground">
                              {persona.description}
                            </p>
                          </div>
                          {!persona.is_default ? (
                            <Button
                              type="button"
                              variant="outline"
                              size="sm"
                              onClick={() => handleSetDefaultPersona(persona.id)}
                            >
                              设为默认
                            </Button>
                          ) : null}
                        </div>
                        <dl className="grid gap-3 border-t pt-3 text-sm sm:grid-cols-3">
                          <div>
                            <dt className="font-medium text-foreground">适用场景</dt>
                            <dd className="mt-1 leading-6 text-muted-foreground">
                              {persona.scene}
                            </dd>
                          </div>
                          <div>
                            <dt className="font-medium text-foreground">输出语气</dt>
                            <dd className="mt-1 leading-6 text-muted-foreground">
                              {persona.tone}
                            </dd>
                          </div>
                          <div>
                            <dt className="font-medium text-foreground">输出结构</dt>
                            <dd className="mt-1 leading-6 text-muted-foreground">
                              {persona.output_structure}
                            </dd>
                          </div>
                        </dl>
                      </section>
                    ))
                ) : (
                  <section className="rounded-lg border border-dashed bg-muted/20 p-5 text-sm leading-6 text-muted-foreground">
                    暂无内置人格。
                  </section>
                )}
              </div>
            </div>

            <div className="space-y-3 border-t pt-4">
              <h3 className="text-sm font-semibold">自定义人格</h3>
              <div className="grid gap-3">
                {personas.filter((p) => !p.is_builtin).length > 0 ? (
                  personas
                    .filter((p) => !p.is_builtin)
                    .map((persona) => (
                      <section
                        key={persona.id}
                        className="grid gap-3 rounded-lg border bg-background p-4"
                      >
                        <div className="flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
                          <div className="min-w-0 flex-1">
                            <div className="flex items-center gap-2">
                              <p className="text-sm font-semibold">
                                {persona.name}
                              </p>
                              {persona.is_default ? (
                                <span className="inline-flex h-6 items-center rounded-md border bg-background px-2 text-xs font-medium">
                                  默认
                                </span>
                              ) : null}
                            </div>
                            <p className="mt-1 text-sm leading-6 text-muted-foreground">
                              {persona.description}
                            </p>
                          </div>
                          <div className="flex items-center gap-2">
                            {!persona.is_default ? (
                              <Button
                                type="button"
                                variant="outline"
                                size="sm"
                                onClick={() => handleSetDefaultPersona(persona.id)}
                              >
                                设为默认
                              </Button>
                            ) : null}
                            <Button
                              type="button"
                              variant="outline"
                              size="icon"
                              onClick={() => openEditPersonaDialog(persona)}
                              aria-label={`编辑 ${persona.name}`}
                            >
                              <PencilIcon className="size-4" aria-hidden="true" />
                            </Button>
                            <Button
                              type="button"
                              variant="outline"
                              size="icon"
                              onClick={() => handleDeletePersona(persona.id)}
                              aria-label={`删除 ${persona.name}`}
                            >
                              <Trash2Icon className="size-4" aria-hidden="true" />
                            </Button>
                          </div>
                        </div>
                        <dl className="grid gap-3 border-t pt-3 text-sm sm:grid-cols-3">
                          <div>
                            <dt className="font-medium text-foreground">适用场景</dt>
                            <dd className="mt-1 leading-6 text-muted-foreground">
                              {persona.scene}
                            </dd>
                          </div>
                          <div>
                            <dt className="font-medium text-foreground">输出语气</dt>
                            <dd className="mt-1 leading-6 text-muted-foreground">
                              {persona.tone}
                            </dd>
                          </div>
                          <div>
                            <dt className="font-medium text-foreground">输出结构</dt>
                            <dd className="mt-1 leading-6 text-muted-foreground">
                              {persona.output_structure}
                            </dd>
                          </div>
                        </dl>
                      </section>
                    ))
                ) : (
                  <section className="rounded-lg border border-dashed bg-muted/20 p-5 text-sm leading-6 text-muted-foreground">
                    暂无自定义人格。可以新建人格来定义自己的文本整理风格。
                  </section>
                )}
              </div>
            </div>

            <div className="flex flex-col gap-2 border-t pt-4 sm:flex-row sm:items-center sm:justify-between">
              <p className="text-sm leading-6 text-muted-foreground">
                {status}
              </p>
              <span className="inline-flex h-8 w-fit items-center rounded-md bg-secondary px-3 text-xs font-medium text-secondary-foreground">
                共 {personas.length} 个人格
              </span>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <div>
              <p className="mb-2 text-xs font-semibold tracking-normal text-primary uppercase">
                T004A UI 基础
              </p>
              <CardTitle className="text-2xl">默认整理风格</CardTitle>
              <CardDescription className="mt-2">
                选择语音内容整理时默认使用的人格。当前阶段只提供内置人格。
              </CardDescription>
            </div>
            <CardAction>
              <span className="inline-flex h-8 items-center rounded-md bg-secondary px-3 text-xs font-medium text-secondary-foreground">
                {isSaving ? "保存中" : "本地配置"}
              </span>
            </CardAction>
          </CardHeader>

          <CardContent className="space-y-5">
            <div className="space-y-2">
              <Label htmlFor="persona-select">默认人格</Label>
              <Select
                value={selectedPersonaId}
                onValueChange={handleDefaultPersonaChange}
                disabled={isSaving || personas.length === 0}
              >
                <SelectTrigger id="persona-select" className="h-10">
                  <SelectValue placeholder="选择默认人格" />
                </SelectTrigger>
                <SelectContent>
                  {personas.map((persona) => (
                    <SelectItem key={persona.id} value={persona.id}>
                      {persona.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            {selectedPersona ? (
              <section className="rounded-lg border bg-muted/40 p-4">
                <div className="mb-4 flex flex-col gap-2 sm:flex-row sm:items-start sm:justify-between">
                  <div>
                    <h2 className="text-lg font-semibold">
                      {selectedPersona.name}
                    </h2>
                    <p className="mt-1 text-sm leading-6 text-muted-foreground">
                      {selectedPersona.description}
                    </p>
                  </div>
                  {selectedPersona.is_default ? (
                    <span className="inline-flex h-7 w-fit items-center rounded-md border bg-background px-2.5 text-xs font-medium">
                      默认
                    </span>
                  ) : null}
                </div>

                <dl className="grid gap-3 text-sm sm:grid-cols-3">
                  <div>
                    <dt className="font-medium text-foreground">适用场景</dt>
                    <dd className="mt-1 leading-6 text-muted-foreground">
                      {selectedPersona.scene}
                    </dd>
                  </div>
                  <div>
                    <dt className="font-medium text-foreground">输出语气</dt>
                    <dd className="mt-1 leading-6 text-muted-foreground">
                      {selectedPersona.tone}
                    </dd>
                  </div>
                  <div>
                    <dt className="font-medium text-foreground">输出结构</dt>
                    <dd className="mt-1 leading-6 text-muted-foreground">
                      {selectedPersona.output_structure}
                    </dd>
                  </div>
                </dl>
              </section>
            ) : null}

            <div className="flex flex-col gap-3 border-t pt-4 sm:flex-row sm:items-center sm:justify-between">
              <p className="text-sm leading-6 text-muted-foreground">
                {status}
              </p>
              <Button type="button" variant="outline" size="sm" disabled>
                {isSaving ? (
                  <Loader2Icon className="size-4 animate-spin" aria-hidden="true" />
                ) : null}
                语音主流程待接入
              </Button>
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <div>
              <p className="mb-2 text-xs font-semibold tracking-normal text-primary uppercase">
                T007 OpenAI 整理
              </p>
              <CardTitle className="text-2xl">文本整理服务</CardTitle>
              <CardDescription className="mt-2">
                配置 OpenAI Responses API，用于把原始识别文本整理成可直接使用的结果。
              </CardDescription>
            </div>
            <CardAction>
              <span className="inline-flex h-8 items-center rounded-md bg-secondary px-3 text-xs font-medium text-secondary-foreground">
                {appConfig?.openai_api_key ? "已配置 Key" : "待配置 Key"}
              </span>
            </CardAction>
          </CardHeader>

          <CardContent>
            <form className="grid gap-4" onSubmit={handleSaveOpenaiConfig}>
              <div className="grid gap-2">
                <Label htmlFor="openai-api-key">OpenAI API Key</Label>
                <Input
                  id="openai-api-key"
                  type="password"
                  value={appConfig?.openai_api_key ?? ""}
                  onChange={(event) =>
                    setAppConfig((config) =>
                      config
                        ? { ...config, openai_api_key: event.target.value }
                        : config,
                    )
                  }
                  placeholder="本地保存，不写入仓库"
                  autoComplete="off"
                />
              </div>

              <div className="grid gap-4 sm:grid-cols-[1fr_180px]">
                <div className="grid gap-2">
                  <Label htmlFor="openai-base-url">Base URL</Label>
                  <Input
                    id="openai-base-url"
                    value={appConfig?.openai_base_url ?? ""}
                    onChange={(event) =>
                      setAppConfig((config) =>
                        config
                          ? { ...config, openai_base_url: event.target.value }
                          : config,
                      )
                    }
                  />
                </div>
                <div className="grid gap-2">
                  <Label htmlFor="openai-model">模型</Label>
                  <Input
                    id="openai-model"
                    value={appConfig?.openai_model ?? ""}
                    onChange={(event) =>
                      setAppConfig((config) =>
                        config
                          ? { ...config, openai_model: event.target.value }
                          : config,
                      )
                    }
                  />
                </div>
              </div>

              <div className="flex flex-col gap-3 border-t pt-4 sm:flex-row sm:items-center sm:justify-between">
                <p className="text-sm leading-6 text-muted-foreground">
                  {openaiStatus}
                </p>
                <Button
                  type="submit"
                  size="sm"
                  disabled={!appConfig || isOpenaiSaving}
                >
                  {isOpenaiSaving ? (
                    <Loader2Icon className="size-4 animate-spin" aria-hidden="true" />
                  ) : (
                    <SaveIcon className="size-4" aria-hidden="true" />
                  )}
                  保存 OpenAI 配置
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <div>
              <p className="mb-2 text-xs font-semibold tracking-normal text-primary uppercase">
                T006 智谱 ASR
              </p>
              <CardTitle className="text-2xl">语音识别服务</CardTitle>
              <CardDescription className="mt-2">
                配置智谱 GLM-ASR-2512，用于把短音频转换为原始识别文本。
              </CardDescription>
            </div>
            <CardAction>
              <span className="inline-flex h-8 items-center rounded-md bg-secondary px-3 text-xs font-medium text-secondary-foreground">
                {appConfig?.asr_api_key ? "已配置 Key" : "待配置 Key"}
              </span>
            </CardAction>
          </CardHeader>

          <CardContent>
            <form className="grid gap-4" onSubmit={handleSaveAsrConfig}>
              <div className="grid gap-2">
                <Label htmlFor="asr-api-key">智谱 API Key</Label>
                <Input
                  id="asr-api-key"
                  type="password"
                  value={appConfig?.asr_api_key ?? ""}
                  onChange={(event) =>
                    setAppConfig((config) =>
                      config
                        ? { ...config, asr_api_key: event.target.value }
                        : config,
                    )
                  }
                  placeholder="本地保存，不写入仓库"
                  autoComplete="off"
                />
              </div>

              <div className="grid gap-4 sm:grid-cols-[1fr_180px]">
                <div className="grid gap-2">
                  <Label htmlFor="asr-base-url">Base URL</Label>
                  <Input
                    id="asr-base-url"
                    value={appConfig?.asr_base_url ?? ""}
                    onChange={(event) =>
                      setAppConfig((config) =>
                        config
                          ? { ...config, asr_base_url: event.target.value }
                          : config,
                      )
                    }
                  />
                </div>
                <div className="grid gap-2">
                  <Label htmlFor="asr-model">模型</Label>
                  <Input
                    id="asr-model"
                    value={appConfig?.asr_model ?? ""}
                    onChange={(event) =>
                      setAppConfig((config) =>
                        config
                          ? { ...config, asr_model: event.target.value }
                          : config,
                      )
                    }
                  />
                </div>
              </div>

              <div className="flex flex-col gap-3 border-t pt-4 sm:flex-row sm:items-center sm:justify-between">
                <p className="text-sm leading-6 text-muted-foreground">
                  {asrStatus}
                </p>
                <Button type="submit" size="sm" disabled={!appConfig || isAsrSaving}>
                  {isAsrSaving ? (
                    <Loader2Icon className="size-4 animate-spin" aria-hidden="true" />
                  ) : (
                    <SaveIcon className="size-4" aria-hidden="true" />
                  )}
                  保存 ASR 配置
                </Button>
              </div>
            </form>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <div>
              <p className="mb-2 text-xs font-semibold tracking-normal text-primary uppercase">
                T005 热词词典
              </p>
              <CardTitle className="text-2xl">热词修正</CardTitle>
              <CardDescription className="mt-2">
                维护专有名词、项目名和技术词，启用后的热词会作为文本整理上下文。
              </CardDescription>
            </div>
            <CardAction>
              <Button type="button" size="sm" onClick={openCreateHotwordDialog}>
                <PlusIcon className="size-4" aria-hidden="true" />
                新增热词
              </Button>
            </CardAction>
          </CardHeader>

          <CardContent className="space-y-5">
            <div className="grid gap-3">
              {hotwords.length > 0 ? (
                hotwords.map((hotword) => (
                  <section
                    key={hotword.id}
                    className="grid gap-3 rounded-lg border bg-muted/30 p-4 sm:grid-cols-[1fr_auto] sm:items-center"
                  >
                    <div className="min-w-0 space-y-2">
                      <div className="flex flex-wrap items-center gap-2">
                        <p className="truncate text-sm font-semibold">
                          {hotword.source_text}
                          <span className="mx-2 text-muted-foreground">→</span>
                          {hotword.target_text}
                        </p>
                        {hotword.category ? (
                          <span className="inline-flex h-6 items-center rounded-md border bg-background px-2 text-xs text-muted-foreground">
                            {hotword.category}
                          </span>
                        ) : null}
                      </div>
                      <p className="text-xs text-muted-foreground">
                        {hotword.enabled ? "已启用" : "已停用"}
                      </p>
                    </div>

                    <div className="flex items-center gap-2">
                      <Switch
                        checked={hotword.enabled}
                        onCheckedChange={(enabled) =>
                          handleHotwordEnabledChange(hotword, enabled)
                        }
                        aria-label={`切换 ${hotword.target_text} 热词状态`}
                      />
                      <Button
                        type="button"
                        variant="outline"
                        size="icon"
                        onClick={() => openEditHotwordDialog(hotword)}
                        aria-label={`编辑 ${hotword.target_text}`}
                      >
                        <PencilIcon className="size-4" aria-hidden="true" />
                      </Button>
                      <Button
                        type="button"
                        variant="outline"
                        size="icon"
                        onClick={() => handleDeleteHotword(hotword.id)}
                        aria-label={`删除 ${hotword.target_text}`}
                      >
                        <Trash2Icon className="size-4" aria-hidden="true" />
                      </Button>
                    </div>
                  </section>
                ))
              ) : (
                <section className="rounded-lg border border-dashed bg-muted/20 p-5 text-sm leading-6 text-muted-foreground">
                  暂无热词。可以先添加项目名、技术词或常见误识别词。
                </section>
              )}
            </div>

            <div className="grid gap-3 border-t pt-4">
              <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
                <p className="text-sm leading-6 text-muted-foreground">
                  {hotwordStatus}
                </p>
                <span className="inline-flex h-8 w-fit items-center rounded-md bg-secondary px-3 text-xs font-medium text-secondary-foreground">
                  已启用 {enabledHotwordCount} 个
                </span>
              </div>
              <Textarea
                value={hotwordContext || "暂无启用热词上下文。"}
                readOnly
                className="min-h-24 resize-none bg-background text-sm"
                aria-label="启用热词上下文"
              />
            </div>
          </CardContent>
        </Card>
      </div>

      <Dialog
        open={isSettingsDialogOpen}
        onOpenChange={setIsSettingsDialogOpen}
      >
        <DialogContent className="max-h-[90vh] overflow-y-auto sm:max-w-2xl">
          <form className="grid gap-4" onSubmit={(e) => {
            e.preventDefault();
            if (!appConfig) return;

            const nextConfig = {
              ...appConfig,
              longpress_shortcut: appConfig.longpress_shortcut.trim(),
              toggle_shortcut: appConfig.toggle_shortcut.trim(),
            };

            if (!nextConfig.longpress_shortcut) {
              toast.error("长按模式快捷键不能为空");
              return;
            }

            if (!nextConfig.toggle_shortcut) {
              toast.error("切换模式快捷键不能为空");
              return;
            }

            invoke<AppConfig>("update_app_config", { config: nextConfig })
              .then((savedConfig) => {
                setAppConfig(savedConfig);
                toast.success("应用设置已保存");
                setIsSettingsDialogOpen(false);
              })
              .catch((error) => {
                toast.error(`保存应用设置失败：${String(error)}`);
              });
          }}>
            <DialogHeader>
              <DialogTitle>应用设置</DialogTitle>
              <DialogDescription>
                配置快捷键、录音模式、输出方式和历史记录保存选项。
              </DialogDescription>
            </DialogHeader>

            <div className="grid gap-4">
              <div className="grid gap-2">
                <Label htmlFor="longpress-shortcut">长按模式快捷键</Label>
                <Input
                  id="longpress-shortcut"
                  value={appConfig?.longpress_shortcut ?? ""}
                  onChange={(event) =>
                    setAppConfig((config) =>
                      config
                        ? { ...config, longpress_shortcut: event.target.value }
                        : config,
                    )
                  }
                  placeholder="例如：RightControl"
                />
                <p className="text-xs text-muted-foreground">
                  按住快捷键录音，松开停止。默认：右Ctrl
                </p>
              </div>

              <div className="grid gap-2">
                <Label htmlFor="toggle-shortcut">切换模式快捷键</Label>
                <Input
                  id="toggle-shortcut"
                  value={appConfig?.toggle_shortcut ?? ""}
                  onChange={(event) =>
                    setAppConfig((config) =>
                      config
                        ? { ...config, toggle_shortcut: event.target.value }
                        : config,
                    )
                  }
                  placeholder="例如：Alt+Space"
                />
                <p className="text-xs text-muted-foreground">
                  按一次开始录音，再按一次停止。默认：左Alt+Space
                </p>
              </div>

              <div className="grid gap-2">
                <Label htmlFor="app-recording-mode">录音模式</Label>
                <Select
                  value={appConfig?.recording_mode ?? "toggle"}
                  onValueChange={(value) =>
                    setAppConfig((config) =>
                      config
                        ? { ...config, recording_mode: value }
                        : config,
                    )
                  }
                >
                  <SelectTrigger id="app-recording-mode" className="h-10">
                    <SelectValue placeholder="选择录音模式" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="long_press">长按录音</SelectItem>
                    <SelectItem value="toggle">切换式录音</SelectItem>
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  长按：按住快捷键录音，松开停止。切换：按一次开始，再按一次停止。
                </p>
              </div>

              <div className="grid gap-2">
                <Label htmlFor="app-microphone">麦克风设备</Label>
                <Select
                  value={appConfig?.selected_microphone || "__default__"}
                  onValueChange={(value) =>
                    setAppConfig((config) =>
                      config
                        ? { ...config, selected_microphone: value === "__default__" ? "" : value }
                        : config,
                    )
                  }
                >
                  <SelectTrigger id="app-microphone" className="h-10">
                    <SelectValue placeholder="使用默认麦克风" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="__default__">使用默认麦克风</SelectItem>
                    {audioDevices.map((device) => (
                      <SelectItem key={device.name} value={device.name}>
                        {device.name} {device.is_default ? "(默认)" : ""}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  选择用于录音的麦克风设备。留空则使用系统默认麦克风。
                </p>
              </div>

              <div className="flex items-center justify-between rounded-lg border p-3">
                <div className="space-y-0.5">
                  <Label htmlFor="app-mute-audio">录音时静音其他应用</Label>
                  <p className="text-xs text-muted-foreground">
                    开启后，语音输入时会暂停系统音频播放，输入完成后自动恢复
                  </p>
                </div>
                <Switch
                  id="app-mute-audio"
                  checked={appConfig?.mute_system_audio ?? false}
                  onCheckedChange={(checked) =>
                    setAppConfig((config) =>
                      config
                        ? { ...config, mute_system_audio: checked }
                        : config,
                    )
                  }
                />
              </div>

              <div className="grid gap-2">
                <Label htmlFor="app-output-mode">输出方式</Label>
                <Select
                  value={appConfig?.output_mode ?? "copy"}
                  onValueChange={(value) =>
                    setAppConfig((config) =>
                      config
                        ? { ...config, output_mode: value }
                        : config,
                    )
                  }
                >
                  <SelectTrigger id="app-output-mode" className="h-10">
                    <SelectValue placeholder="选择输出方式" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="copy">复制到剪贴板</SelectItem>
                    <SelectItem value="paste">自动粘贴</SelectItem>
                  </SelectContent>
                </Select>
                <p className="text-xs text-muted-foreground">
                  复制：结果复制到剪贴板。自动粘贴：尝试模拟粘贴到当前输入位置。
                </p>
              </div>

              <div className="flex items-center justify-between rounded-lg border p-3">
                <div className="space-y-0.5">
                  <Label htmlFor="app-auto-save">自动保存历史</Label>
                  <p className="text-xs text-muted-foreground">
                    每次语音输入完成后自动保存到历史记录
                  </p>
                </div>
                <Switch
                  id="app-auto-save"
                  checked={appConfig?.auto_save_history ?? true}
                  onCheckedChange={(checked) =>
                    setAppConfig((config) =>
                      config
                        ? { ...config, auto_save_history: checked }
                        : config,
                    )
                  }
                />
              </div>
            </div>

            <DialogFooter>
              <Button type="submit" size="sm" disabled={!appConfig}>
                <SaveIcon className="size-4" aria-hidden="true" />
                保存设置
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <Dialog
        open={isHotwordDialogOpen}
        onOpenChange={setIsHotwordDialogOpen}
      >
        <DialogContent>
          <form onSubmit={handleSaveHotword} className="grid gap-4">
            <DialogHeader>
              <DialogTitle>
                {editingHotwordId ? "编辑热词" : "新增热词"}
              </DialogTitle>
              <DialogDescription>
                原始说法用于匹配口述识别结果，修正写法会进入 AI 整理上下文。
              </DialogDescription>
            </DialogHeader>

            <div className="grid gap-4">
              <div className="grid gap-2">
                <Label htmlFor="hotword-source">原始说法</Label>
                <Input
                  id="hotword-source"
                  value={hotwordDraft.source_text}
                  onChange={(event) =>
                    setHotwordDraft((draft) => ({
                      ...draft,
                      source_text: event.target.value,
                    }))
                  }
                  placeholder="next 点 js"
                />
              </div>

              <div className="grid gap-2">
                <Label htmlFor="hotword-target">修正写法</Label>
                <Input
                  id="hotword-target"
                  value={hotwordDraft.target_text}
                  onChange={(event) =>
                    setHotwordDraft((draft) => ({
                      ...draft,
                      target_text: event.target.value,
                    }))
                  }
                  placeholder="Next.js"
                />
              </div>

              <div className="grid gap-2">
                <Label htmlFor="hotword-category">分类</Label>
                <Input
                  id="hotword-category"
                  value={hotwordDraft.category}
                  onChange={(event) =>
                    setHotwordDraft((draft) => ({
                      ...draft,
                      category: event.target.value,
                    }))
                  }
                  placeholder="技术词"
                />
              </div>

              <div className="flex items-center justify-between rounded-lg border p-3">
                <Label htmlFor="hotword-enabled">启用热词</Label>
                <Switch
                  id="hotword-enabled"
                  checked={hotwordDraft.enabled}
                  onCheckedChange={(enabled) =>
                    setHotwordDraft((draft) => ({ ...draft, enabled }))
                  }
                />
              </div>
            </div>

            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={() => setIsHotwordDialogOpen(false)}
              >
                取消
              </Button>
              <Button type="submit" disabled={isHotwordSaving}>
                {isHotwordSaving ? (
                  <Loader2Icon className="size-4 animate-spin" aria-hidden="true" />
                ) : null}
                保存
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>

      <Dialog
        open={isPersonaDialogOpen}
        onOpenChange={setIsPersonaDialogOpen}
      >
        <DialogContent className="max-h-[90vh] overflow-y-auto">
          <form onSubmit={handleSavePersona} className="grid gap-4">
            <DialogHeader>
              <DialogTitle>
                {editingPersonaId ? "编辑人格" : "新建人格"}
              </DialogTitle>
              <DialogDescription>
                定义人格的名称、描述、适用场景和整理提示词。
              </DialogDescription>
            </DialogHeader>

            <div className="grid gap-4">
              <div className="grid gap-2">
                <Label htmlFor="persona-name">人格名称</Label>
                <Input
                  id="persona-name"
                  value={personaDraft.name}
                  onChange={(event) =>
                    setPersonaDraft((draft) => ({
                      ...draft,
                      name: event.target.value,
                    }))
                  }
                  placeholder="例如：技术文档助手"
                />
              </div>

              <div className="grid gap-2">
                <Label htmlFor="persona-description">人格描述</Label>
                <Textarea
                  id="persona-description"
                  value={personaDraft.description}
                  onChange={(event) =>
                    setPersonaDraft((draft) => ({
                      ...draft,
                      description: event.target.value,
                    }))
                  }
                  placeholder="简要说明这个人格的用途"
                  className="min-h-20 resize-none"
                />
              </div>

              <div className="grid gap-2">
                <Label htmlFor="persona-scene">适用场景</Label>
                <Input
                  id="persona-scene"
                  value={personaDraft.scene}
                  onChange={(event) =>
                    setPersonaDraft((draft) => ({
                      ...draft,
                      scene: event.target.value,
                    }))
                  }
                  placeholder="例如：技术文档、API 说明"
                />
              </div>

              <div className="grid gap-2">
                <Label htmlFor="persona-tone">输出语气</Label>
                <Input
                  id="persona-tone"
                  value={personaDraft.tone}
                  onChange={(event) =>
                    setPersonaDraft((draft) => ({
                      ...draft,
                      tone: event.target.value,
                    }))
                  }
                  placeholder="例如：专业、准确、简洁"
                />
              </div>

              <div className="grid gap-2">
                <Label htmlFor="persona-structure">输出结构</Label>
                <Input
                  id="persona-structure"
                  value={personaDraft.output_structure}
                  onChange={(event) =>
                    setPersonaDraft((draft) => ({
                      ...draft,
                      output_structure: event.target.value,
                    }))
                  }
                  placeholder="例如：标题、要点、代码示例"
                />
              </div>

              <div className="grid gap-2">
                <Label htmlFor="persona-prompt">整理提示词</Label>
                <Textarea
                  id="persona-prompt"
                  value={personaDraft.prompt}
                  onChange={(event) =>
                    setPersonaDraft((draft) => ({
                      ...draft,
                      prompt: event.target.value,
                    }))
                  }
                  placeholder="输入用于指导 AI 整理文本的系统提示词"
                  className="min-h-32 resize-none"
                />
              </div>
            </div>

            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={() => setIsPersonaDialogOpen(false)}
              >
                取消
              </Button>
              <Button type="submit" disabled={isPersonaSaving}>
                {isPersonaSaving ? (
                  <Loader2Icon className="size-4 animate-spin" aria-hidden="true" />
                ) : null}
                保存
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </main>
  );
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
