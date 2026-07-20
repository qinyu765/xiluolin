import type { Hotword, HotwordDraft } from "@/generated/tauri-bindings";

export type { Hotword, HotwordDraft };

export const emptyHotwordDraft: HotwordDraft = {
  text: "",
  category: "",
  enabled: true,
};
