// 将快捷键字符串转换为中文显示
export function formatShortcutDisplay(shortcut: string): string {
  if (!shortcut) return "";

  // 检查是否为连按格式 (如 "Space*2")
  const multiPressMatch = shortcut.match(/^(.+)\*(\d+)$/);
  if (multiPressMatch) {
    const [, baseShortcut, count] = multiPressMatch;
    const formatted = formatSingleShortcut(baseShortcut);
    return `${formatted}×${count}`;
  }

  return formatSingleShortcut(shortcut);
}

function formatSingleShortcut(shortcut: string): string {
  const keyMap: Record<string, string> = {
    Control: "Ctrl",
    CommandOrControl: "Ctrl",
    Shift: "Shift",
    Alt: "Alt",
    Meta: "Win",
    Super: "Win",
    Space: "空格",
    Enter: "回车",
    Backspace: "退格",
    Tab: "Tab",
    Escape: "Esc",
    Up: "↑",
    Down: "↓",
    Left: "←",
    Right: "→",
    Delete: "Delete",
    Home: "Home",
    End: "End",
    PageUp: "PageUp",
    PageDown: "PageDown",
  };

  return shortcut
    .split("+")
    .map((key) => keyMap[key] || key)
    .join(" + ");
}
