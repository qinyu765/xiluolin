export type Hotword = {
  id: string;
  source_text: string;
  target_text: string;
  category: string;
  enabled: boolean;
};

export type HotwordDraft = {
  source_text: string;
  target_text: string;
  category: string;
  enabled: boolean;
};

export const emptyHotwordDraft: HotwordDraft = {
  source_text: "",
  target_text: "",
  category: "",
  enabled: true,
};
