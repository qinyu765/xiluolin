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
  return `已启用 ${uniqueTexts.length} 个去重热词。使用智谱 ASR 时仅前 ${MAX_ASR_HOTWORDS} 个用于语音识别；全部热词仍会用于文本整理。`;
}
