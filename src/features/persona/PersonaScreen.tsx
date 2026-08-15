import { useConfigController } from "@/features/settings/useConfigController";
import { PersonaDeleteDialog } from "@/components/dialogs/PersonaDeleteDialog";
import { usePersonaController } from "./usePersonaController";
import { PersonaDialog } from "@/components/dialogs/PersonaDialog";
import { PersonaPage } from "@/pages/PersonaPage";

export function PersonaScreen() {
  const config = useConfigController();
  const personas = usePersonaController(config.setAppConfig);

  return (
    <>
      <PersonaPage
        personas={personas.personas}
        onCreatePersona={personas.openCreate}
        onEditPersona={personas.openEdit}
        onRequestDeletePersona={personas.requestDelete}
        onSelectPersona={personas.setDefault}
      />
      <PersonaDialog
        open={personas.isDialogOpen}
        isEditing={personas.editingId !== null}
        isSaving={personas.isSaving}
        draft={personas.draft}
        onOpenChange={personas.setDialogOpen}
        onDraftChange={personas.setDraft}
        onSave={personas.save}
      />
      <PersonaDeleteDialog
        open={personas.deleteTarget !== null}
        persona={personas.deleteTarget}
        isDeleting={personas.isDeleting}
        onOpenChange={(open) => {
          if (!open && !personas.isDeleting) personas.setDeleteTarget(null);
        }}
        onConfirm={() => void personas.confirmDelete()}
      />
    </>
  );
}
