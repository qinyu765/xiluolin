import React from "react";
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
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { IconPicker } from "@/components/ui/icon-picker";
import type { PersonaDraft } from "@/types";

type PersonaDialogProps = {
  open: boolean;
  isEditing: boolean;
  isSaving: boolean;
  draft: PersonaDraft;
  onOpenChange: (open: boolean) => void;
  onDraftChange: (draft: PersonaDraft) => void;
  onSave: (event: React.FormEvent<HTMLFormElement>) => void;
};

export function PersonaDialog({
  open,
  isEditing,
  isSaving,
  draft,
  onOpenChange,
  onDraftChange,
  onSave,
}: PersonaDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[90vh] overflow-y-auto">
        <form onSubmit={onSave} className="grid gap-4">
          <DialogHeader>
            <DialogTitle>
              {isEditing ? "编辑人格" : "新建人格"}
            </DialogTitle>
            <DialogDescription>
              定义人格的名称、风格描述和图标。必填字段标记为 *。
            </DialogDescription>
          </DialogHeader>

          <div className="grid gap-4">
            <div className="grid gap-2">
              <Label htmlFor="persona-name">
                人格名称 <span className="text-destructive">*</span>
              </Label>
              <Input
                id="persona-name"
                value={draft.name}
                onChange={(event) =>
                  onDraftChange({
                    ...draft,
                    name: event.target.value,
                  })
                }
                placeholder="简短的人格名称，如「技术文档助手」"
                required
              />
            </div>

            <div className="grid gap-2">
              <Label htmlFor="persona-description">
                风格描述 <span className="text-destructive">*</span>
              </Label>
              <Textarea
                id="persona-description"
                value={draft.description}
                onChange={(event) =>
                  onDraftChange({
                    ...draft,
                    description: event.target.value,
                  })
                }
                placeholder="描述这个人格的风格和输出要求，例如：将语音转换为清晰、可执行的 AI Prompt。输出结构：目标、上下文、约束、期望结果。"
                className="min-h-32 resize-none"
                required
              />
            </div>

            <div className="grid gap-2">
              <Label>图标</Label>
              <IconPicker
                value={draft.icon}
                onChange={(icon) =>
                  onDraftChange({
                    ...draft,
                    icon,
                  })
                }
              />
            </div>
          </div>

          <DialogFooter>
            <Button
              type="button"
              variant="outline"
              onClick={() => onOpenChange(false)}
            >
              取消
            </Button>
            <Button type="submit" disabled={isSaving}>
              {isSaving ? (
                <Loader2Icon className="size-4 animate-spin" aria-hidden="true" />
              ) : null}
              保存
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
