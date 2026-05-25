export type Persona = {
  id: string;
  name: string;
  description: string;
  scene: string;
  tone: string;
  output_structure: string;
  prompt: string;
  is_builtin: boolean;
  is_default: boolean;
};

export type PersonaDraft = {
  name: string;
  description: string;
  scene: string;
  tone: string;
  output_structure: string;
  prompt: string;
};

export const emptyPersonaDraft: PersonaDraft = {
  name: "",
  description: "",
  scene: "",
  tone: "",
  output_structure: "",
  prompt: "",
};
