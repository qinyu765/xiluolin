import { BookmarkIcon, HomeIcon, SettingsIcon, UserIcon } from "lucide-react";

import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import type { Page } from "@/types";

const TAB_CLASS =
  "group h-9 flex-none justify-start gap-2 rounded-md px-3 text-muted-foreground transition-[color,background-color,box-shadow,transform] duration-150 hover:translate-x-0.5 hover:bg-card hover:text-foreground hover:shadow-sm data-[state=active]:bg-card data-[state=active]:text-foreground data-[state=active]:shadow-sm";
const ICON_CLASS =
  "size-4 text-muted-foreground transition-[color,transform] duration-150 group-hover:scale-105 group-hover:text-primary group-data-[state=active]:text-primary";

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
      <div className="border-b px-6 py-4">
        <h1 className="text-2xl font-semibold tracking-normal [font-family:Georgia,'Times_New_Roman',serif]">
          XiLuoLin
        </h1>
      </div>

      <TabsList className="flex h-auto min-h-0 w-full flex-1 flex-col items-stretch gap-1 rounded-none bg-transparent p-2 pb-3">
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
