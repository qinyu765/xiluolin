import { useMemo, useState } from "react";
import {
  PencilIcon,
  PlusIcon,
  SaveIcon,
  SearchIcon,
  Trash2Icon,
} from "lucide-react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import { getEnabledHotwordAsrLimitNotice } from "@/lib/hotword-limit";
import type { Hotword } from "@/types";

type HotwordPageProps = {
  hotwords: Hotword[];
  bulkText: string;
  bulkCount: number;
  isLoading: boolean;
  isBulkDirty: boolean;
  isBulkSaving: boolean;
  onBulkTextChange: (value: string) => void;
  onSaveBulk: () => void;
  onClearBulk: () => void;
  onCreateHotword: () => void;
  onEditHotword: (hotword: Hotword) => void;
  onDeleteHotword: (id: string) => void;
  onHotwordEnabledChange: (hotword: Hotword, enabled: boolean) => void;
};

export function HotwordPage({
  hotwords,
  bulkText,
  bulkCount,
  isLoading,
  isBulkDirty,
  isBulkSaving,
  onBulkTextChange,
  onSaveBulk,
  onClearBulk,
  onCreateHotword,
  onEditHotword,
  onDeleteHotword,
  onHotwordEnabledChange,
}: HotwordPageProps) {
  const [query, setQuery] = useState("");
  const asrLimitNotice = getEnabledHotwordAsrLimitNotice(hotwords);
  const filteredHotwords = useMemo(() => {
    const normalizedQuery = query.trim().toLocaleLowerCase();
    if (!normalizedQuery) return hotwords;
    return hotwords.filter((hotword) =>
      [hotword.text, hotword.category]
        .join(" ")
        .toLocaleLowerCase()
        .includes(normalizedQuery),
    );
  }, [hotwords, query]);

  return (
    <div className="space-y-10">
      <header className="flex flex-col gap-5 sm:flex-row sm:items-end sm:justify-between">
        <div>
          <h1 className="text-3xl font-semibold tracking-tight">热词</h1>
          <p className="mt-2 text-base text-muted-foreground">
            添加那些只属于你的重要词汇
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Button
            type="button"
            variant="ghost"
            onClick={onClearBulk}
            disabled={!bulkText || isBulkSaving}
          >
            <Trash2Icon className="size-4" aria-hidden="true" />
            清空
          </Button>
          <Button
            type="button"
            onClick={onSaveBulk}
            disabled={!isBulkDirty || isBulkSaving}
          >
            <SaveIcon className="size-4" aria-hidden="true" />
            {isBulkSaving ? "保存中…" : "保存"}
          </Button>
        </div>
      </header>

      <section className="rounded-2xl border bg-card p-1 shadow-xs">
        {isLoading ? (
          <div className="min-h-64 motion-safe:animate-pulse rounded-[0.9rem] bg-muted/40" />
        ) : (
          <Textarea
            value={bulkText}
            onChange={(event) => onBulkTextChange(event.target.value)}
            placeholder={
              "请按每行一个热词输入，点击保存后添加。\n首尾空格会自动过滤，空白行不会保存。"
            }
            aria-label="添加热词"
            className="min-h-64 resize-none rounded-[0.9rem] border-0 bg-transparent p-6 text-base leading-7 shadow-none focus-visible:ring-0"
          />
        )}
        <div className="flex justify-end px-5 pb-4 pt-2 text-sm text-muted-foreground">
          {bulkCount} 个待添加热词
        </div>
      </section>

      <section className="space-y-5">
        <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
          <div>
            <h2 className="text-lg font-semibold">管理热词</h2>
            {asrLimitNotice ? (
              <p className="mt-2 text-sm text-amber-700 dark:text-amber-400">
                {asrLimitNotice}
              </p>
            ) : null}
          </div>
          <div className="flex items-center gap-2">
            <div className="relative w-full sm:w-64">
              <SearchIcon
                className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-muted-foreground"
                aria-hidden="true"
              />
              <Input
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                placeholder="搜索热词"
                aria-label="搜索热词"
                className="h-9 pl-9"
              />
            </div>
            <Button
              type="button"
              variant="outline"
              size="icon"
              onClick={onCreateHotword}
              aria-label="新增热词"
              title="新增热词"
            >
              <PlusIcon className="size-4" aria-hidden="true" />
            </Button>
          </div>
        </div>

        {isLoading ? (
          <div className="grid gap-3 sm:grid-cols-2">
            <div className="h-16 motion-safe:animate-pulse rounded-xl bg-muted/40" />
            <div className="h-16 motion-safe:animate-pulse rounded-xl bg-muted/40" />
          </div>
        ) : filteredHotwords.length > 0 ? (
          <div className="grid gap-3 sm:grid-cols-2">
            {filteredHotwords.map((hotword) => (
              <section
                key={hotword.id}
                className="flex min-w-0 items-center justify-between gap-4 rounded-xl border bg-card px-5 py-4 transition-colors hover:border-primary/35"
              >
                <div className="min-w-0">
                  <p className="truncate text-base font-medium">
                    {hotword.text}
                  </p>
                  {hotword.category ? (
                    <p className="mt-1 truncate text-sm text-muted-foreground">
                      {hotword.category}
                    </p>
                  ) : null}
                </div>
                <div className="flex shrink-0 items-center gap-1">
                  <Switch
                    checked={hotword.enabled}
                    onCheckedChange={(enabled) =>
                      onHotwordEnabledChange(hotword, enabled)
                    }
                    aria-label={`切换 ${hotword.text} 热词状态`}
                  />
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    onClick={() => onEditHotword(hotword)}
                    aria-label={`编辑 ${hotword.text}`}
                    title="编辑"
                  >
                    <PencilIcon className="size-4" aria-hidden="true" />
                  </Button>
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    className="text-muted-foreground hover:text-destructive"
                    onClick={() => onDeleteHotword(hotword.id)}
                    aria-label={`删除 ${hotword.text}`}
                    title="删除"
                  >
                    <Trash2Icon className="size-4" aria-hidden="true" />
                  </Button>
                </div>
              </section>
            ))}
          </div>
        ) : (
          <section className="rounded-xl border border-dashed p-8 text-center text-sm text-muted-foreground">
            {query ? "没有匹配的热词。" : "还没有热词。"}
          </section>
        )}
      </section>
    </div>
  );
}
