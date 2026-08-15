export function normalizeHotwordLines(value: string) {
  const lines = value.split(/\r?\n/);
  const normalized: string[] = [];
  for (const line of lines) {
    const text = line.trim();
    if (text && !normalized.includes(text)) {
      normalized.push(text);
    }
  }
  return normalized;
}
