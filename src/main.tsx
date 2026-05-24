import React, { useEffect, useMemo, useState } from "react";
import ReactDOM from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
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

type AppConfig = {
  default_persona_id: string;
  asr_api_key: string;
  asr_base_url: string;
  asr_model: string;
  openai_api_key: string;
  openai_base_url: string;
  openai_model: string;
  recording_mode: string;
  shortcut: string;
  output_mode: string;
  auto_save_history: boolean;
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
  const [status, setStatus] = useState("正在读取本地人格配置...");
  const [asrStatus, setAsrStatus] = useState("正在读取智谱 ASR 配置...");
  const [openaiStatus, setOpenaiStatus] = useState("正在读取 OpenAI 配置...");
  const [hotwordStatus, setHotwordStatus] = useState("正在读取热词词典...");
  const [historyStatus, setHistoryStatus] = useState("正在读取历史记录...");
  const [voiceStatus, setVoiceStatus] = useState("请选择一段 wav 或 mp3 短音频。");
  const [selectedAudioName, setSelectedAudioName] = useState("");
  const [voiceResult, setVoiceResult] = useState<VoiceInputResult | null>(null);
  const [isSaving, setIsSaving] = useState(false);
  const [isAsrSaving, setIsAsrSaving] = useState(false);
  const [isOpenaiSaving, setIsOpenaiSaving] = useState(false);
  const [isHotwordSaving, setIsHotwordSaving] = useState(false);
  const [isVoiceProcessing, setIsVoiceProcessing] = useState(false);

  const selectedPersona = useMemo(
    () => personas.find((persona) => persona.id === selectedPersonaId),
    [personas, selectedPersonaId],
  );

  const enabledHotwordCount = hotwords.filter(
    (hotword) => hotword.enabled,
  ).length;

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

  async function handleProcessAudio(event: React.ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    event.target.value = "";
    if (!file) {
      return;
    }

    const extension = file.name.split(".").pop()?.toLowerCase() ?? "";
    if (extension !== "wav" && extension !== "mp3") {
      setVoiceStatus("仅支持 wav 或 mp3 短音频。");
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
    } catch (error) {
      setVoiceStatus(`语音主流程失败：${String(error)}`);
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
    } catch (error) {
      setVoiceStatus(`复制失败：${String(error)}`);
    }
  }

  return (
    <main className="min-h-screen px-4 py-8 sm:px-6 lg:px-8">
      <div className="mx-auto grid min-h-[calc(100vh-4rem)] w-full max-w-4xl content-center gap-6">
        <section className="space-y-4">
          <div className="inline-flex items-center gap-2 rounded-md border bg-card px-3 py-1 text-sm font-medium text-muted-foreground shadow-sm">
            <Mic2Icon className="size-4 text-primary" aria-hidden="true" />
            AI 语音输入助手
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
                T008 主流程
              </p>
              <CardTitle className="text-2xl">短音频输入</CardTitle>
              <CardDescription className="mt-2">
                上传 wav 或 mp3 短音频，串联 ASR 识别、人格化整理、结果展示和复制。
              </CardDescription>
            </div>
            <CardAction>
              <span className="inline-flex h-8 items-center rounded-md bg-secondary px-3 text-xs font-medium text-secondary-foreground">
                {isVoiceProcessing ? "处理中" : "上传音频"}
              </span>
            </CardAction>
          </CardHeader>

          <CardContent className="space-y-5">
            <div className="grid gap-3 rounded-lg border border-dashed bg-muted/20 p-5">
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
                  disabled={isVoiceProcessing}
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
                disabled={isVoiceProcessing}
              />
              <p className="text-sm leading-6 text-muted-foreground">
                {voiceStatus}
              </p>
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
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      onClick={handleCopyFinalText}
                    >
                      <CopyIcon className="size-4" aria-hidden="true" />
                      复制结果
                    </Button>
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
                        <div className="min-w-0">
                          <p className="text-sm font-semibold">
                            {record.persona_name}
                          </p>
                          <p className="mt-1 text-xs text-muted-foreground">
                            {formatCreatedAt(record.created_at)} ·{" "}
                            {formatDuration(record.duration_ms)} ·{" "}
                            {record.output_chars} 字
                          </p>
                        </div>
                        <span className="inline-flex h-7 w-fit items-center rounded-md border bg-muted/30 px-2.5 text-xs text-muted-foreground">
                          {record.output_mode === "paste" ? "自动粘贴" : "复制"}
                        </span>
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
    </main>
  );
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
