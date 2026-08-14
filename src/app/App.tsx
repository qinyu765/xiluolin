import { useState } from "react";
import { Toaster } from "sonner";

import { HomeDashboard } from "@/features/history/HomeDashboard";
import { HotwordScreen } from "@/features/hotword/HotwordScreen";
import { PersonaScreen } from "@/features/persona/PersonaScreen";
import { SettingsScreen } from "@/features/settings/SettingsScreen";
import { CaptureEventToasts } from "@/platform/tauri/CaptureEventToasts";
import type { Page } from "@/types";

import { AppSidebar } from "./AppSidebar";

export function App() {
  const [page, setPage] = useState<Page>("home");
  return (
    <main className="flex min-h-screen">
      <Toaster position="top-center" richColors />
      <CaptureEventToasts />
      <AppSidebar page={page} onPageChange={setPage} />

      <div className="ml-44 flex-1 overflow-y-auto overflow-x-hidden">
        <div className="mx-auto max-w-5xl px-6 py-8">
          {page === "home" && <HomeDashboard />}
          {page === "persona" && <PersonaScreen />}
          {page === "hotword" && <HotwordScreen />}
          {page === "settings" && <SettingsScreen />}
        </div>
      </div>
    </main>
  );
}
