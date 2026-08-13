import { useConfigController } from "@/features/settings/useConfigController";
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
        status={personas.status}
        onCreatePersona={personas.openCreate}
        onEditPersona={personas.openEdit}
        onDeletePersona={personas.deletePersona}
        onSetDefaultPersona={personas.setDefault}
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
    </>
  );
}
