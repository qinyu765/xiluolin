import { useHotwordController } from "./useHotwordController";
import { HotwordDialog } from "@/components/dialogs/HotwordDialog";
import { HotwordPage } from "@/pages/HotwordPage";

export function HotwordScreen() {
  const hotwords = useHotwordController();

  return (
    <>
      <HotwordPage
        hotwords={hotwords.hotwords}
        bulkText={hotwords.bulkText}
        bulkCount={hotwords.bulkCount}
        isLoading={hotwords.isLoading}
        isBulkDirty={hotwords.isBulkDirty}
        isBulkSaving={hotwords.isBulkSaving}
        onBulkTextChange={hotwords.setBulkText}
        onSaveBulk={() => void hotwords.saveBulk()}
        onClearBulk={hotwords.clearBulk}
        onCreateHotword={hotwords.openCreate}
        onEditHotword={hotwords.openEdit}
        onDeleteHotword={hotwords.deleteHotword}
        onHotwordEnabledChange={hotwords.setEnabled}
      />
      <HotwordDialog
        open={hotwords.isDialogOpen}
        isEditing={hotwords.editingId !== null}
        isSaving={hotwords.isSaving}
        draft={hotwords.draft}
        onOpenChange={hotwords.setDialogOpen}
        onDraftChange={hotwords.setDraft}
        onSave={hotwords.save}
      />
    </>
  );
}
