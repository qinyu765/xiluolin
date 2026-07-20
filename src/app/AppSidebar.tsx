import { BookmarkIcon, HomeIcon, SettingsIcon, UserIcon } from "lucide-react";

import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import type { Page } from "@/types";

const TAB_CLASS =
  "justify-start gap-2 rounded-none border-l-2 border-transparent px-5 py-3 text-muted-foreground transition-colors data-[state=active]:border-primary data-[state=active]:bg-background data-[state=active]:text-foreground data-[state=active]:shadow-none";
const ICON_CLASS = "size-4 shrink-0";

export function AppSidebar({
  page,
  onPageChange,
}: {
  page: Page;
  onPageChange: (page: Page) => void;
}) {
  return (
    <Tabs
      value={page}
      onValueChange={(value) => onPageChange(value as Page)}
      orientation="vertical"
      className="fixed left-0 top-0 z-10 flex h-screen w-48 flex-col border-r bg-muted/30"
    >
      <div className="border-b px-5 py-5">
        <p className="text-lg font-semibold tracking-tight">XiLuoLin</p>
        <p className="mt-1 text-xs text-muted-foreground">AI 语音输入助手</p>
      </div>
      <TabsList className="flex h-auto flex-1 flex-col items-stretch gap-1 rounded-none bg-transparent p-0 py-3">
        <TabsTrigger value="home" className={TAB_CLASS}>
          <HomeIcon className={ICON_CLASS} aria-hidden="true" />
          首页
        </TabsTrigger>
        <TabsTrigger value="persona" className={TAB_CLASS}>
          <UserIcon className={ICON_CLASS} aria-hidden="true" />
          人格
        </TabsTrigger>
        <TabsTrigger value="hotword" className={TAB_CLASS}>
          <BookmarkIcon className={ICON_CLASS} aria-hidden="true" />
          热词
        </TabsTrigger>
        <TabsTrigger
          value="settings"
          className={`${TAB_CLASS} relative mt-auto before:absolute before:-top-3 before:left-0 before:right-0 before:h-px before:bg-border/80 before:content-['']`}
        >
          <SettingsIcon className={ICON_CLASS} aria-hidden="true" />
          设置
        </TabsTrigger>
      </TabsList>
    </Tabs>
  );
}
