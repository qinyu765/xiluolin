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
              定义人格的名称、描述、适用场景和整理提示词。
            </DialogDescription>
          </DialogHeader>

          <div className="grid gap-4">
            <div className="grid gap-2">
              <Label htmlFor="persona-name">人格名称</Label>
              <Input
                id="persona-name"
                value={draft.name}
                onChange={(event) =>
                  onDraftChange({
                    ...draft,
                    name: event.target.value,
                  })
                }
                placeholder="例如：技术文档助手"
              />
            </div>

            <div className="grid gap-2">
              <Label htmlFor="persona-description">人格描述</Label>
              <Textarea
                id="persona-description"
                value={draft.description}
                onChange={(event) =>
                  onDraftChange({
                    ...draft,
                    description: event.target.value,
                  })
                }
                placeholder="简要说明这个人格的用途"
                className="min-h-20 resize-none"
              />
            </div>

            <div className="grid gap-2">
              <Label htmlFor="persona-scene">适用场景</Label>
              <Input
                id="persona-scene"
                value={draft.scene}
                onChange={(event) =>
                  onDraftChange({
                    ...draft,
                    scene: event.target.value,
                  })
                }
                placeholder="例如：技术文档、API 说明"
              />
            </div>

            <div className="grid gap-2">
              <Label htmlFor="persona-tone">输出语气</Label>
              <Input
                id="persona-tone"
                value={draft.tone}
                onChange={(event) =>
                  onDraftChange({
                    ...draft,
                    tone: event.target.value,
                  })
                }
                placeholder="例如：专业、准确、简洁"
              />
            </div>

            <div className="grid gap-2">
              <Label htmlFor="persona-structure">输出结构</Label>
              <Input
                id="persona-structure"
                value={draft.output_structure}
                onChange={(event) =>
                  onDraftChange({
                    ...draft,
                    output_structure: event.target.value,
                  })
                }
                placeholder="例如：标题、要点、代码示例"
              />
            </div>

            <div className="grid gap-2">
              <Label htmlFor="persona-prompt">整理提示词</Label>
              <Textarea
                id="persona-prompt"
                value={draft.prompt}
                onChange={(event) =>
                  onDraftChange({
                    ...draft,
                    prompt: event.target.value,
                  })
                }
                placeholder="输入用于指导 AI 整理文本的系统提示词"
                className="min-h-32 resize-none"
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
