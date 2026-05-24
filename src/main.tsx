import React, { useEffect, useMemo, useState } from "react";
import ReactDOM from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import { Loader2Icon, Mic2Icon } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
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

function App() {
  const [personas, setPersonas] = useState<Persona[]>([]);
  const [selectedPersonaId, setSelectedPersonaId] = useState("");
  const [status, setStatus] = useState("正在读取本地人格配置...");
  const [isSaving, setIsSaving] = useState(false);

  const selectedPersona = useMemo(
    () => personas.find((persona) => persona.id === selectedPersonaId),
    [personas, selectedPersonaId],
  );

  useEffect(() => {
    async function loadPersonas() {
      try {
        await invoke("initialize_local_data");
        const loadedPersonas = await invoke<Persona[]>("list_personas");
        const defaultPersona =
          loadedPersonas.find((persona) => persona.is_default) ??
          loadedPersonas[0];

        setPersonas(loadedPersonas);
        setSelectedPersonaId(defaultPersona?.id ?? "");
        setStatus("已加载内置人格，可选择默认整理风格。");
      } catch (error) {
        setStatus(`读取人格失败：${String(error)}`);
      }
    }

    loadPersonas();
  }, []);

  async function handleDefaultPersonaChange(personaId: string) {
    setSelectedPersonaId(personaId);
    setIsSaving(true);
    setStatus("正在保存默认人格...");

    try {
      const updatedPersonas = await invoke<Persona[]>("set_default_persona", {
        personaId,
      });
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

  return (
    <main className="min-h-screen px-4 py-8 sm:px-6 lg:px-8">
      <div className="mx-auto grid min-h-[calc(100vh-4rem)] w-full max-w-3xl content-center gap-6">
        <section className="space-y-4">
          <div className="inline-flex items-center gap-2 rounded-md border bg-card px-3 py-1 text-sm font-medium text-muted-foreground shadow-sm">
            <Mic2Icon className="size-4 text-primary" aria-hidden="true" />
            AI 语音输入助手
          </div>
          <div className="space-y-3">
            <h1 className="text-5xl font-semibold tracking-normal text-balance sm:text-6xl">
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
      </div>
    </main>
  );
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
