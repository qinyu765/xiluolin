import { useEffect, useMemo, useState } from "react";

import { commands } from "@/generated/tauri-bindings";
import type { AppConfig, Persona, PersonaDraft } from "@/types";
import { emptyPersonaDraft } from "@/types";
import { toErrorMessage } from "@/utils/error";

export function usePersonaController(
  onConfigLoaded: (config: AppConfig) => void,
) {
  const [personas, setPersonas] = useState<Persona[]>([]);
  const [selectedId, setSelectedId] = useState("");
  const [status, setStatus] = useState("正在读取本地人格配置...");
  const [draft, setDraft] = useState<PersonaDraft>(emptyPersonaDraft);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [isDialogOpen, setDialogOpen] = useState(false);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    let active = true;
    void commands
      .listPersonas()
      .then((nextPersonas) => {
        if (!active) return;
        const defaultPersona =
          nextPersonas.find((persona) => persona.is_default) ?? nextPersonas[0];
        setPersonas(nextPersonas);
        setSelectedId(defaultPersona?.id ?? "");
        setStatus("已加载内置人格，可选择默认整理风格。");
      })
      .catch((error) => {
        if (active) setStatus(`读取人格失败：${toErrorMessage(error)}`);
      });
    return () => {
      active = false;
    };
  }, []);

  const openCreate = () => {
    setEditingId(null);
    setDraft(emptyPersonaDraft);
    setDialogOpen(true);
  };

  const openEdit = (persona: Persona) => {
    setEditingId(persona.id);
    setDraft({
      name: persona.name,
      description: persona.description,
      icon: persona.icon,
    });
    setDialogOpen(true);
  };

  const save = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const nextDraft = {
      name: draft.name.trim(),
      description: draft.description.trim(),
      icon: draft.icon.trim(),
    };
    if (!nextDraft.name || !nextDraft.description) {
      setStatus("人格名称和描述不能为空。");
      return;
    }

    setIsSaving(true);
    setStatus("正在保存人格...");
    try {
      if (editingId) await commands.updatePersona(editingId, nextDraft);
      else await commands.createPersona(nextDraft);
      setPersonas(await commands.listPersonas());
      setStatus("人格已保存。");
      setDialogOpen(false);
    } catch (error) {
      setStatus(`保存人格失败：${toErrorMessage(error)}`);
    } finally {
      setIsSaving(false);
    }
  };

  const deletePersona = async (id: string) => {
    setStatus("正在删除人格...");
    try {
      setPersonas(await commands.deletePersona(id));
      setStatus("人格已删除。");
    } catch (error) {
      setStatus(`删除人格失败：${toErrorMessage(error)}`);
    }
  };

  const setDefault = async (personaId: string) => {
    setStatus("正在设置默认人格...");
    try {
      const [nextPersonas, config] = await Promise.all([
        commands.setDefaultPersona(personaId),
        commands.readAppConfig(),
      ]);
      setPersonas(nextPersonas);
      setSelectedId(personaId);
      onConfigLoaded(config as AppConfig);
      setStatus("默认人格已设置。");
    } catch (error) {
      setStatus(`设置默认人格失败：${toErrorMessage(error)}`);
    }
  };

  const selected = useMemo(
    () => personas.find((persona) => persona.id === selectedId),
    [personas, selectedId],
  );

  return {
    personas,
    selected,
    selectedId,
    status,
    draft,
    editingId,
    isDialogOpen,
    isSaving,
    setSelectedId,
    setDraft,
    setDialogOpen,
    openCreate,
    openEdit,
    save,
    deletePersona,
    setDefault,
  };
}
