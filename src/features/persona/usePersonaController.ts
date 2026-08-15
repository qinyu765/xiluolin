import { useEffect, useMemo, useState, type FormEvent } from "react";
import { toast } from "sonner";

import { commands } from "@/generated/tauri-bindings";
import type { AppConfig, Persona, PersonaDraft } from "@/types";
import { emptyPersonaDraft } from "@/types";
import { toErrorMessage } from "@/utils/error";

export function usePersonaController(
  onConfigLoaded: (config: AppConfig) => void,
) {
  const [personas, setPersonas] = useState<Persona[]>([]);
  const [selectedId, setSelectedId] = useState("");
  const [draft, setDraft] = useState<PersonaDraft>(emptyPersonaDraft);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [isDialogOpen, setDialogOpen] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<Persona | null>(null);
  const [isDeleting, setIsDeleting] = useState(false);

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
      })
      .catch((error) => {
        if (active) toast.error(`读取人格失败：${toErrorMessage(error)}`);
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
      processing_mode: persona.processing_mode,
    });
    setDialogOpen(true);
  };

  const save = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const nextDraft = {
      name: draft.name.trim(),
      description: draft.description.trim(),
      icon: draft.icon.trim(),
      processing_mode: draft.processing_mode,
    };
    if (!nextDraft.name || !nextDraft.description) {
      toast.error("人格名称和描述不能为空");
      return;
    }

    setIsSaving(true);
    try {
      if (editingId) await commands.updatePersona(editingId, nextDraft);
      else await commands.createPersona(nextDraft);
      const nextPersonas = await commands.listPersonas();
      setPersonas(nextPersonas);
      setDialogOpen(false);
      toast.success("人格已保存");
    } catch (error) {
      toast.error(`保存人格失败：${toErrorMessage(error)}`);
    } finally {
      setIsSaving(false);
    }
  };

  const setDefault = async (personaId: string) => {
    try {
      const update = await commands.setDefaultPersona(personaId);
      setPersonas(update.personas);
      setSelectedId(personaId);
      onConfigLoaded(update.config as AppConfig);
      toast.success("默认人格已切换");
    } catch (error) {
      toast.error(`切换默认人格失败：${toErrorMessage(error)}`);
    }
  };

  const requestDelete = (persona: Persona) => {
    if (persona.id === "general" || !persona.is_default) return;
    setDeleteTarget(persona);
  };

  const confirmDelete = async () => {
    if (!deleteTarget || isDeleting) return;

    setIsDeleting(true);
    try {
      const update = await commands.deletePersona(deleteTarget.id);
      setPersonas(update.personas);
      setSelectedId(update.config.default_persona_id);
      onConfigLoaded(update.config as AppConfig);
      setDeleteTarget(null);
      toast.success("人格已删除，已切换为通用人格");
    } catch (error) {
      toast.error(`删除人格失败：${toErrorMessage(error)}`);
    } finally {
      setIsDeleting(false);
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
    draft,
    editingId,
    isDialogOpen,
    isSaving,
    deleteTarget,
    isDeleting,
    setDraft,
    setDialogOpen,
    setDeleteTarget,
    openCreate,
    openEdit,
    save,
    setDefault,
    requestDelete,
    confirmDelete,
  };
}
