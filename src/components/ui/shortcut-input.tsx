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
  const [activeModifiers, setActiveModifiers] = useState<Map<string, string>>(new Map());
  const [hasNonModifierKey, setHasNonModifierKey] = useState(false);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (!isRecording) return;

    e.preventDefault();
    e.stopPropagation();

    // 更新活跃的修饰键状态
    const newModifiers = new Map(activeModifiers);

    if (e.code === "ControlLeft") {
      newModifiers.set("ctrl", "LeftControl");
    } else if (e.code === "ControlRight") {
      newModifiers.set("ctrl", "RightControl");
    }

    if (e.code === "ShiftLeft") {
      newModifiers.set("shift", "LeftShift");
    } else if (e.code === "ShiftRight") {
      newModifiers.set("shift", "RightShift");
    }

    if (e.code === "AltLeft") {
      newModifiers.set("alt", "LeftAlt");
    } else if (e.code === "AltRight") {
      newModifiers.set("alt", "RightAlt");
    }

    if (e.code === "MetaLeft" || e.code === "MetaRight") {
      newModifiers.set("meta", "Meta");
    }

    setActiveModifiers(newModifiers);

    // 获取主键名称
    const mainKey = getKeyName(e.key, e.code);

    // 检查主键是否为修饰键本身
    const isModifierKey = ["LeftControl", "RightControl", "LeftShift", "RightShift", "LeftAlt", "RightAlt", "Meta"].includes(mainKey);

    if (isModifierKey) {
      // 按下的是修饰键本身，不完成录制，等待用户是否会按其他键
      return;
    }

    // 按下的是非修饰键，标记并构建组合键
    setHasNonModifierKey(true);

    const keys: string[] = [];

    // 从 activeModifiers 中获取当前按下的修饰键
    if (e.ctrlKey && newModifiers.has("ctrl")) {
      keys.push(newModifiers.get("ctrl")!);
    }
    if (e.shiftKey && newModifiers.has("shift")) {
      keys.push(newModifiers.get("shift")!);
    }
    if (e.altKey && newModifiers.has("alt")) {
      keys.push(newModifiers.get("alt")!);
    }
    if (e.metaKey && newModifiers.has("meta")) {
      keys.push(newModifiers.get("meta")!);
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

  const handleKeyUp = (e: React.KeyboardEvent) => {
    if (!isRecording) return;

    // 获取松开的键
    const releasedKey = getKeyName(e.key, e.code);
    const isModifierKey = ["LeftControl", "RightControl", "LeftShift", "RightShift", "LeftAlt", "RightAlt", "Meta"].includes(releasedKey);

    if (isModifierKey && !hasNonModifierKey) {
      // 松开的是修饰键，且没有按过其他键，记录为单键
      handleKeyPress(releasedKey);
    }

    // 移除释放的修饰键
    const newModifiers = new Map(activeModifiers);

    if (e.code === "ControlLeft" || e.code === "ControlRight") {
      newModifiers.delete("ctrl");
    }
    if (e.code === "ShiftLeft" || e.code === "ShiftRight") {
      newModifiers.delete("shift");
    }
    if (e.code === "AltLeft" || e.code === "AltRight") {
      newModifiers.delete("alt");
    }
    if (e.code === "MetaLeft" || e.code === "MetaRight") {
      newModifiers.delete("meta");
    }

    setActiveModifiers(newModifiers);
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
    setHasNonModifierKey(false);
  };

  const handleFocus = () => {
    setIsRecording(true);
    setActiveModifiers(new Map());
    setHasNonModifierKey(false);
  };

  const handleBlur = () => {
    setIsRecording(false);
    setActiveModifiers(new Map());
    setHasNonModifierKey(false);
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
    "ArrowUp": "Up",
    "ArrowDown": "Down",
    "ArrowLeft": "Left",
    "ArrowRight": "Right",
    "Enter": "Enter",
    "Escape": "Escape",
    "Backspace": "Backspace",
    "Tab": "Tab",
    "Delete": "Delete",
    "Home": "Home",
    "End": "End",
    "PageUp": "PageUp",
    "PageDown": "PageDown",
  };

  // 处理修饰键（带左右区分）
  if (code === "ControlRight") return "RightControl";
  if (code === "ControlLeft") return "LeftControl";
  if (code === "ShiftRight") return "RightShift";
  if (code === "ShiftLeft") return "LeftShift";
  if (code === "AltRight") return "RightAlt";
  if (code === "AltLeft") return "LeftAlt";
  if (code === "MetaLeft" || code === "MetaRight") return "Meta";

  // 检查特殊键
  if (specialKeys[key]) return specialKeys[key];

  // 字母和数字键直接大写
  if (key.length === 1) {
    return key.toUpperCase();
  }

  return key;
}
