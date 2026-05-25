export type Hotword = {
  id: string;
  text: string;
  category: string;
  enabled: boolean;
};

export type HotwordDraft = {
  text: string;
  category: string;
  enabled: boolean;
};

export const emptyHotwordDraft: HotwordDraft = {
  text: "",
  category: "",
  enabled: true,
};
