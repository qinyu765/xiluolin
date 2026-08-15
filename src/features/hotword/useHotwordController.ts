import { useCallback, useEffect, useState } from "react";
import { toast } from "sonner";

import { commands } from "@/generated/tauri-bindings";
import { emptyHotwordDraft } from "@/types";
import type { Hotword, HotwordDraft } from "@/types";
import { toErrorMessage } from "@/utils/error";
import { normalizeHotwordLines } from "@/lib/hotword-text";

export function useHotwordController() {
  const [hotwords, setHotwords] = useState<Hotword[]>([]);
  const [bulkText, setBulkText] = useState("");
  const [isLoading, setIsLoading] = useState(true);
  const [draft, setDraft] = useState<HotwordDraft>(emptyHotwordDraft);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [isDialogOpen, setDialogOpen] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [isBulkSaving, setIsBulkSaving] = useState(false);

  const reload = useCallback(async () => {
    const nextHotwords = await commands.listHotwords();
    setHotwords(nextHotwords);
  }, []);

  useEffect(() => {
    void reload()
      .catch((error) => toast.error(`读取热词失败：${toErrorMessage(error)}`))
      .finally(() => setIsLoading(false));
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
      toast.error("热词不能为空");
      return;
    }

    setIsSaving(true);
    try {
      if (editingId) await commands.updateHotword(editingId, nextDraft);
      else await commands.createHotword(nextDraft);
      await reload();
      setDialogOpen(false);
      toast.success(editingId ? "热词已更新" : "热词已添加");
    } catch (error) {
      toast.error(`保存热词失败：${toErrorMessage(error)}`);
    } finally {
      setIsSaving(false);
    }
  };

  const saveBulk = async () => {
    const texts = normalizeHotwordLines(bulkText);
    if (texts.length === 0) return;

    setIsBulkSaving(true);
    try {
      await commands.addHotwords(texts);
      await reload();
      setBulkText("");
      toast.success("热词已添加");
    } catch (error) {
      toast.error(`添加热词失败：${toErrorMessage(error)}`);
    } finally {
      setIsBulkSaving(false);
    }
  };

  const setEnabled = async (hotword: Hotword, enabled: boolean) => {
    try {
      await commands.updateHotword(hotword.id, {
        text: hotword.text,
        category: hotword.category,
        enabled,
      });
      await reload();
      toast.success(enabled ? "热词已启用" : "热词已停用");
    } catch (error) {
      toast.error(`更新热词失败：${toErrorMessage(error)}`);
    }
  };

  const deleteHotword = async (id: string) => {
    try {
      await commands.deleteHotword(id);
      await reload();
      toast.success("热词已删除");
    } catch (error) {
      toast.error(`删除热词失败：${toErrorMessage(error)}`);
    }
  };

  const bulkCount = normalizeHotwordLines(bulkText).length;

  return {
    hotwords,
    bulkText,
    bulkCount,
    isLoading,
    isBulkDirty: bulkCount > 0,
    draft,
    editingId,
    isDialogOpen,
    isSaving,
    isBulkSaving,
    setBulkText,
    setDraft,
    setDialogOpen,
    openCreate,
    openEdit,
    save,
    saveBulk,
    clearBulk: () => setBulkText(""),
    setEnabled,
    deleteHotword,
  };
}
