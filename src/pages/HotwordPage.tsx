import React from "react";
import { PencilIcon, PlusIcon, Trash2Icon } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import type { Hotword } from "@/types";

type HotwordPageProps = {
  hotwords: Hotword[];
  hotwordContext: string;
  hotwordStatus: string;
  enabledHotwordCount: number;
  onCreateHotword: () => void;
  onEditHotword: (hotword: Hotword) => void;
  onDeleteHotword: (id: string) => void;
  onHotwordEnabledChange: (hotword: Hotword, enabled: boolean) => void;
};

export function HotwordPage({
  hotwords,
  hotwordContext,
  hotwordStatus,
  enabledHotwordCount,
  onCreateHotword,
  onEditHotword,
  onDeleteHotword,
  onHotwordEnabledChange,
}: HotwordPageProps) {
  return (
    <div className="space-y-6">
      <Card>
        <CardHeader>
          <div>
            <p className="mb-2 text-xs font-semibold tracking-normal text-primary uppercase">
              T005 热词词典
            </p>
            <CardTitle className="text-2xl">热词修正</CardTitle>
            <CardDescription className="mt-2">
              维护专有名词、项目名和技术词，启用后的热词会作为文本整理上下文。
            </CardDescription>
          </div>
          <CardAction>
            <Button type="button" size="sm" onClick={onCreateHotword}>
              <PlusIcon className="size-4" aria-hidden="true" />
              新增热词
            </Button>
          </CardAction>
        </CardHeader>

        <CardContent className="space-y-5">
          <div className="grid gap-3">
            {hotwords.length > 0 ? (
              hotwords.map((hotword) => (
                <section
                  key={hotword.id}
                  className="grid gap-3 rounded-lg border bg-muted/30 p-4 sm:grid-cols-[1fr_auto] sm:items-center"
                >
                  <div className="min-w-0 space-y-2">
                    <div className="flex flex-wrap items-center gap-2">
                      <p className="truncate text-sm font-semibold">
                        {hotword.source_text}
                        <span className="mx-2 text-muted-foreground">→</span>
                        {hotword.target_text}
                      </p>
                      {hotword.category ? (
                        <span className="inline-flex h-6 items-center rounded-md border bg-background px-2 text-xs text-muted-foreground">
                          {hotword.category}
                        </span>
                      ) : null}
                    </div>
                    <p className="text-xs text-muted-foreground">
                      {hotword.enabled ? "已启用" : "已停用"}
                    </p>
                  </div>

                  <div className="flex items-center gap-2">
                    <Switch
                      checked={hotword.enabled}
                      onCheckedChange={(enabled) =>
                        onHotwordEnabledChange(hotword, enabled)
                      }
                      aria-label={`切换 ${hotword.target_text} 热词状态`}
                    />
                    <Button
                      type="button"
                      variant="outline"
                      size="icon"
                      onClick={() => onEditHotword(hotword)}
                      aria-label={`编辑 ${hotword.target_text}`}
                    >
                      <PencilIcon className="size-4" aria-hidden="true" />
                    </Button>
                    <Button
                      type="button"
                      variant="outline"
                      size="icon"
                      onClick={() => onDeleteHotword(hotword.id)}
                      aria-label={`删除 ${hotword.target_text}`}
                    >
                      <Trash2Icon className="size-4" aria-hidden="true" />
                    </Button>
                  </div>
                </section>
              ))
            ) : (
              <section className="rounded-lg border border-dashed bg-muted/20 p-5 text-sm leading-6 text-muted-foreground">
                暂无热词。可以先添加项目名、技术词或常见误识别词。
              </section>
            )}
          </div>

          <div className="grid gap-3 border-t pt-4">
            <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
              <p className="text-sm leading-6 text-muted-foreground">
                {hotwordStatus}
              </p>
              <span className="inline-flex h-8 w-fit items-center rounded-md bg-secondary px-3 text-xs font-medium text-secondary-foreground">
                已启用 {enabledHotwordCount} 个
              </span>
            </div>
            <Textarea
              value={hotwordContext || "暂无启用热词上下文。"}
              readOnly
              className="min-h-24 resize-none bg-background text-sm"
              aria-label="启用热词上下文"
            />
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
