import { useState } from "react";
import { Toaster } from "sonner";

import { HomeDashboard } from "@/features/history/HomeDashboard";
import { HotwordScreen } from "@/features/hotword/HotwordScreen";
import { PersonaScreen } from "@/features/persona/PersonaScreen";
import { SettingsScreen } from "@/features/settings/SettingsScreen";
import { CaptureEventToasts } from "@/platform/tauri/CaptureEventToasts";
import type { Page } from "@/types";

import { AppSidebar } from "./AppSidebar";
import { AppShell } from "./AppShell";

export function App() {
  const [page, setPage] = useState<Page>("home");
  return (
    <AppShell sidebar={<AppSidebar page={page} onPageChange={setPage} />}>
      <Toaster position="top-center" richColors />
      <CaptureEventToasts />
      {page === "home" && <HomeDashboard />}
      {page === "persona" && <PersonaScreen />}
      {page === "hotword" && <HotwordScreen />}
      {page === "settings" && <SettingsScreen />}
    </AppShell>
  );
}
