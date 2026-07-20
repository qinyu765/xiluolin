import React from "react";
import { CopyIcon, FileAudioIcon, Loader2Icon, Mic2Icon, SaveIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";

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

type VoiceInputResult = {
  raw_text: string;
  final_text: string;
  used_text_fallback: boolean;
  history_record: any | null;
};

type QuickStartCardProps = {
  personas: Persona[];
  selectedPersonaId: string;
  selectedPersona: Persona | undefined;
  isRecording: boolean;
  isVoiceProcessing: boolean;
  recordingDuration: number;
  voiceStatus: string;
  selectedAudioName: string;
  voiceResult: VoiceInputResult | null;
  onPersonaChange: (personaId: string) => void;
  onStartRecording: () => void;
  onStopRecording: () => void;
  onProcessAudio: (event: React.ChangeEvent<HTMLInputElement>) => void;
  onCopyFinalText: () => void;
  onOutputText: () => void;
  formatDuration: (ms: number) => string;
};

export function QuickStartCard({
  personas,
  selectedPersonaId,
  selectedPersona,
  isRecording,
  isVoiceProcessing,
  recordingDuration,
  voiceStatus,
  selectedAudioName,
  voiceResult,
  onPersonaChange,
  onStartRecording,
  onStopRecording,
  onProcessAudio,
  onCopyFinalText,
  onOutputText,
  formatDuration,
}: QuickStartCardProps) {
  return (
    <Card>
      <CardHeader>
        <div>
          <CardTitle className="text-2xl">快速开始</CardTitle>
          <CardDescription className="mt-2">
            点击录音按钮开始说话,再次点击停止录音并自动处理。支持上传音频文件测试。
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
            onValueChange={onPersonaChange}
            disabled={isRecording || isVoiceProcessing || personas.length === 0}
          >
            <SelectTrigger id="main-persona-select" className="h-10">
              <SelectValue placeholder="选择人格" />
            </SelectTrigger>
            <SelectContent>
              {personas.map((persona) => (
                <SelectItem key={persona.id} value={persona.id}>
                  {persona.name}{persona.id === "general" ? "（推荐）" : ""}
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
              onClick={isRecording ? onStopRecording : onStartRecording}
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
            onChange={onProcessAudio}
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
                    onClick={onCopyFinalText}
                  >
                    <CopyIcon className="size-4" aria-hidden="true" />
                    复制
                  </Button>
                  <Button
                    type="button"
                    size="sm"
                    onClick={onOutputText}
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
  );
}
