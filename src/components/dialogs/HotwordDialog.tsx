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
import { Switch } from "@/components/ui/switch";
import type { HotwordDraft } from "@/types";

type HotwordDialogProps = {
  open: boolean;
  isEditing: boolean;
  isSaving: boolean;
  draft: HotwordDraft;
  onOpenChange: (open: boolean) => void;
  onDraftChange: (draft: HotwordDraft) => void;
  onSave: (event: React.FormEvent<HTMLFormElement>) => void;
};

export function HotwordDialog({
  open,
  isEditing,
  isSaving,
  draft,
  onOpenChange,
  onDraftChange,
  onSave,
}: HotwordDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent>
        <form onSubmit={onSave} className="grid gap-4">
          <DialogHeader>
            <DialogTitle>
              {isEditing ? "编辑热词" : "新增热词"}
            </DialogTitle>
            <DialogDescription>
              定义需要 AI 准确识别的专业术语、技术词汇或特定表达。必填字段标记为 *。
            </DialogDescription>
          </DialogHeader>

          <div className="grid gap-4">
            <div className="grid gap-2">
              <Label htmlFor="hotword-text">
                热词 <span className="text-destructive">*</span>
              </Label>
              <Input
                id="hotword-text"
                value={draft.text}
                onChange={(event) =>
                  onDraftChange({
                    ...draft,
                    text: event.target.value,
                  })
                }
                placeholder="输入热词，如「Kubernetes」、「Next.js」、「TypeScript」"
                required
              />
            </div>

            <div className="grid gap-2">
              <Label htmlFor="hotword-category">分类（可选）</Label>
              <Input
                id="hotword-category"
                value={draft.category}
                onChange={(event) =>
                  onDraftChange({
                    ...draft,
                    category: event.target.value,
                  })
                }
                placeholder="用于组织热词，如「技术词」、「项目名」"
              />
            </div>

            <div className="flex items-center justify-between rounded-lg border p-3">
              <Label htmlFor="hotword-enabled">启用热词</Label>
              <Switch
                id="hotword-enabled"
                checked={draft.enabled}
                onCheckedChange={(enabled) =>
                  onDraftChange({ ...draft, enabled })
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
