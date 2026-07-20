import type { Persona, PersonaDraft } from "@/generated/tauri-bindings";

export type { Persona, PersonaDraft };

export const emptyPersonaDraft: PersonaDraft = {
  name: "",
  description: "",
  icon: "Sparkles",
};
