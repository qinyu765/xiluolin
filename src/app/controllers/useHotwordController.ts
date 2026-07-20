import { useCallback, useEffect, useMemo, useState } from "react";

import { commands } from "@/generated/tauri-bindings";
import type { Hotword, HotwordDraft } from "@/types";
import { emptyHotwordDraft } from "@/types";
import { toErrorMessage } from "@/utils/error";

export function useHotwordController() {
  const [hotwords, setHotwords] = useState<Hotword[]>([]);
  const [context, setContext] = useState("");
  const [status, setStatus] = useState("正在读取热词词典...");
  const [draft, setDraft] = useState<HotwordDraft>(emptyHotwordDraft);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [isDialogOpen, setDialogOpen] = useState(false);
  const [isSaving, setIsSaving] = useState(false);

  const reload = useCallback(async (nextStatus: string) => {
    const [nextHotwords, nextContext] = await Promise.all([
      commands.listHotwords(),
      commands.enabledHotwordContext(),
    ]);
    setHotwords(nextHotwords);
    setContext(nextContext);
    setStatus(nextStatus);
  }, []);

  useEffect(() => {
    void reload("热词词典已加载。").catch((error) =>
      setStatus(`热词词典读取失败：${toErrorMessage(error)}`),
    );
  }, [reload]);

  const openCreate = () => {
    setEditingId(null);
    setDraft(emptyHotwordDraft);
    setDialogOpen(true);
  };

  const openEdit = (hotword: Hotword) => {
    setEditingId(hotword.id);
    setDraft({
      text: hotword.text,
      category: hotword.category,
      enabled: hotword.enabled,
    });
    setDialogOpen(true);
  };

  const save = async (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const nextDraft = {
      ...draft,
      text: draft.text.trim(),
      category: draft.category.trim(),
    };
    if (!nextDraft.text) {
      setStatus("热词不能为空。");
      return;
    }

    setIsSaving(true);
    setStatus("正在保存热词...");
    try {
      if (editingId) await commands.updateHotword(editingId, nextDraft);
      else await commands.createHotword(nextDraft);
      await reload("热词已保存，并会进入文本整理上下文。");
      setDialogOpen(false);
    } catch (error) {
      setStatus(`保存热词失败：${toErrorMessage(error)}`);
    } finally {
      setIsSaving(false);
    }
  };

  const setEnabled = async (hotword: Hotword, enabled: boolean) => {
    setStatus("正在更新热词状态...");
    try {
      await commands.updateHotword(hotword.id, {
        text: hotword.text,
        category: hotword.category,
        enabled,
      });
      await reload(enabled ? "热词已启用。" : "热词已停用。");
    } catch (error) {
      setStatus(`更新热词状态失败：${toErrorMessage(error)}`);
    }
  };

  const deleteHotword = async (id: string) => {
    setStatus("正在删除热词...");
    try {
      const [nextHotwords, nextContext] = await Promise.all([
        commands.deleteHotword(id),
        commands.enabledHotwordContext(),
      ]);
      setHotwords(nextHotwords);
      setContext(nextContext);
      setStatus("热词已删除。");
    } catch (error) {
      setStatus(`删除热词失败：${toErrorMessage(error)}`);
    }
  };

  const enabledCount = useMemo(
    () => hotwords.filter((hotword) => hotword.enabled).length,
    [hotwords],
  );

  return {
    hotwords,
    context,
    status,
    draft,
    editingId,
    isDialogOpen,
    isSaving,
    enabledCount,
    setDraft,
    setDialogOpen,
    openCreate,
    openEdit,
    save,
    setEnabled,
    deleteHotword,
  };
}
