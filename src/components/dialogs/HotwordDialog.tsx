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
              原始说法用于匹配口述识别结果，修正写法会进入 AI 整理上下文。
            </DialogDescription>
          </DialogHeader>

          <div className="grid gap-4">
            <div className="grid gap-2">
              <Label htmlFor="hotword-source">原始说法</Label>
              <Input
                id="hotword-source"
                value={draft.source_text}
                onChange={(event) =>
                  onDraftChange({
                    ...draft,
                    source_text: event.target.value,
                  })
                }
                placeholder="next 点 js"
              />
            </div>

            <div className="grid gap-2">
              <Label htmlFor="hotword-target">修正写法</Label>
              <Input
                id="hotword-target"
                value={draft.target_text}
                onChange={(event) =>
                  onDraftChange({
                    ...draft,
                    target_text: event.target.value,
                  })
                }
                placeholder="Next.js"
              />
            </div>

            <div className="grid gap-2">
              <Label htmlFor="hotword-category">分类</Label>
              <Input
                id="hotword-category"
                value={draft.category}
                onChange={(event) =>
                  onDraftChange({
                    ...draft,
                    category: event.target.value,
                  })
                }
                placeholder="技术词"
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
