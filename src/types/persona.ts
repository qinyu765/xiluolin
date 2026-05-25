export type Persona = {
  id: string;
  name: string;
  description: string;
  icon: string;
  is_default: boolean;
  created_at: string;
  updated_at: string;
};

export type PersonaDraft = {
  name: string;
  description: string;
  icon: string;
};

export const emptyPersonaDraft: PersonaDraft = {
  name: "",
  description: "",
  icon: "Sparkles",
};
