import { Loader2Icon } from "lucide-react";

import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import type { Persona } from "@/types";

type PersonaDeleteDialogProps = {
  open: boolean;
  persona: Persona | null;
  isDeleting: boolean;
  onOpenChange: (open: boolean) => void;
  onConfirm: () => void;
};

export function PersonaDeleteDialog({
  open,
  persona,
  isDeleting,
  onOpenChange,
  onConfirm,
}: PersonaDeleteDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>删除人格？</DialogTitle>
          <DialogDescription>
            删除「{persona?.name ?? "当前人格"}
            」后无法恢复。当前人格会切换为通用人格。
          </DialogDescription>
        </DialogHeader>
        <DialogFooter>
          <Button
            type="button"
            variant="outline"
            onClick={() => onOpenChange(false)}
            disabled={isDeleting}
          >
            取消
          </Button>
          <Button
            type="button"
            variant="destructive"
            onClick={onConfirm}
            disabled={isDeleting || !persona}
          >
            {isDeleting ? (
              <Loader2Icon className="size-4 animate-spin" aria-hidden="true" />
            ) : null}
            删除人格
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
