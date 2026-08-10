type HotwordLike = {
  enabled: boolean;
  text: string;
};

const MAX_ASR_HOTWORDS = 100;

export function getEnabledHotwordAsrLimitNotice(
  hotwords: HotwordLike[],
): string | null {
  const uniqueTexts: string[] = [];
  for (const hotword of hotwords) {
    const text = hotword.text.trim();
    if (hotword.enabled && text && !uniqueTexts.includes(text)) {
      uniqueTexts.push(text);
    }
  }

  if (uniqueTexts.length <= MAX_ASR_HOTWORDS) return null;
  return `已启用 ${uniqueTexts.length} 个去重热词。语音识别仅使用前 ${MAX_ASR_HOTWORDS} 个，其余热词仍会用于文本整理。`;
}
