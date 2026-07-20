import React, { useState } from "react";
import { XIcon, RotateCcwIcon } from "lucide-react";
import { Button } from "./button";
import { Input } from "./input";
import { formatShortcutDisplay } from "@/utils/shortcut";

type ShortcutInputProps = {
  value: string;
  defaultValue: string;
  onChange: (value: string) => void;
  placeholder?: string;
  disabled?: boolean;
};

export function ShortcutInput({
  value,
  defaultValue,
  onChange,
  placeholder = "点击后按下快捷键",
  disabled = false,
}: ShortcutInputProps) {
  const [isRecording, setIsRecording] = useState(false);
  const [pressCount, setPressCount] = useState(0);
  const [lastKey, setLastKey] = useState<string>("");
  const [pressTimer, setPressTimer] = useState<NodeJS.Timeout | null>(null);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (!isRecording) return;

    e.preventDefault();
    e.stopPropagation();

    // 获取主键名称
    const mainKey = getKeyName(e.key, e.code);

    // 检查主键是否为修饰键本身
    const isModifierKey = ["Control", "Shift", "Alt", "Meta"].includes(mainKey);

    if (isModifierKey) {
      // 按下的是修饰键本身，不完成录制，等待用户是否会按其他键
      return;
    }

    // 按下的是非修饰键，构建组合键
    const keys: string[] = [];

    // 添加修饰键（不区分左右）
    if (e.ctrlKey || e.metaKey) {
      keys.push("CommandOrControl");
    }
    if (e.shiftKey) {
      keys.push("Shift");
    }
    if (e.altKey) {
      keys.push("Alt");
    }

    // 添加主键
    if (mainKey) {
      keys.push(mainKey);
    }

    if (keys.length > 0) {
      const shortcut = keys.join("+");
      handleKeyPress(shortcut);
    }
  };

  const handleKeyUp = () => {
    // 在标准格式下，不支持单独的修饰键作为快捷键
    // 因此 keyUp 事件不需要特殊处理
    return;
  };

  const handleKeyPress = (shortcut: string) => {
    // 清除之前的定时器
    if (pressTimer) {
      clearTimeout(pressTimer);
    }

    // 检查是否为连按
    if (lastKey === shortcut) {
      // 相同按键，增加计数
      const newCount = pressCount + 1;
      setPressCount(newCount);

      // 限制最大连按次数为 5
      if (newCount >= 5) {
        finishRecording(shortcut, newCount);
        return;
      }

      // 继续等待可能的下一次按键
      const timer = setTimeout(() => {
        finishRecording(shortcut, newCount);
      }, 300);
      setPressTimer(timer);
    } else {
      // 不同按键，完成之前的录制（如果有）
      if (lastKey && pressCount > 0) {
        finishRecording(lastKey, pressCount);
      }

      // 开始新的按键录制
      setLastKey(shortcut);
      setPressCount(1);

      // 启动定时器等待可能的连按
      const timer = setTimeout(() => {
        finishRecording(shortcut, 1);
      }, 300);
      setPressTimer(timer);
    }
  };

  const finishRecording = (shortcut: string, count: number) => {
    if (pressTimer) {
      clearTimeout(pressTimer);
      setPressTimer(null);
    }

    // 生成最终的快捷键字符串
    const finalShortcut = count > 1 ? `${shortcut}*${count}` : shortcut;
    onChange(finalShortcut);

    // 重置状态
    setIsRecording(false);
    setPressCount(0);
    setLastKey("");
  };

  const handleFocus = () => {
    setIsRecording(true);
  };

  const handleBlur = () => {
    setIsRecording(false);
  };

  const handleClear = () => {
    onChange("");
  };

  const handleReset = () => {
    onChange(defaultValue);
  };

  // 将快捷键字符串转换为中文显示
  const displayValue = formatShortcutDisplay(value);

  return (
    <div className="flex gap-2">
      <Input
        value={isRecording ? "请按下快捷键..." : displayValue}
        onKeyDown={handleKeyDown}
        onKeyUp={handleKeyUp}
        onFocus={handleFocus}
        onBlur={handleBlur}
        placeholder={placeholder}
        readOnly
        disabled={disabled}
        className="flex-1"
      />
      {value ? (
        <Button
          type="button"
          variant="outline"
          size="icon"
          onClick={handleClear}
          disabled={disabled}
          title="清空快捷键"
        >
          <XIcon className="size-4" />
        </Button>
      ) : (
        <Button
          type="button"
          variant="outline"
          size="icon"
          onClick={handleReset}
          disabled={disabled}
          title="恢复默认值"
        >
          <RotateCcwIcon className="size-4" />
        </Button>
      )}
    </div>
  );
}

// 辅助函数：获取按键名称
function getKeyName(key: string, code: string): string {
  // 特殊键映射
  const specialKeys: Record<string, string> = {
    " ": "Space",
    ArrowUp: "Up",
    ArrowDown: "Down",
    ArrowLeft: "Left",
    ArrowRight: "Right",
    Enter: "Enter",
    Escape: "Escape",
    Backspace: "Backspace",
    Tab: "Tab",
    Delete: "Delete",
    Home: "Home",
    End: "End",
    PageUp: "PageUp",
    PageDown: "PageDown",
  };

  // 处理修饰键（不区分左右）
  if (code.startsWith("Control")) return "Control";
  if (code.startsWith("Shift")) return "Shift";
  if (code.startsWith("Alt")) return "Alt";
  if (code.startsWith("Meta")) return "Meta";

  // 检查特殊键
  if (specialKeys[key]) return specialKeys[key];

  // 字母和数字键直接大写
  if (key.length === 1) {
    return key.toUpperCase();
  }

  return key;
}
