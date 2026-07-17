# 快捷键输入组件问题修复记录

> **归档状态：** 历史快捷键输入组件修复记录，其中代码路径和实现细节可能已经变化。

## 问题描述

在设置页面配置快捷键时，遇到以下问题：

1. **单键显示重复**：按下右Ctrl时显示 "Ctrl + 右Ctrl"，应该只显示 "右Ctrl"
2. **无法输入组合键**：大部分情况下只能输入单个修饰键，无法输入组合键（如 左Alt + 空格）
3. **缺少连按支持**：组件不支持连按同一个键的场景（如双击空格、三击Ctrl等）
4. **左右区分不完整**：修饰键没有完全区分左右（如显示 "Alt" 而不是 "左Alt"）

## 问题根源分析

### 问题1：单键显示重复

**原因**：当用户按下右Ctrl时，`e.ctrlKey` 为 `true`，同时 `e.code` 为 `"ControlRight"`。原代码先通过 `e.ctrlKey` 添加 "Control"，再通过 `getKeyName(e.key, e.code)` 添加 "RightControl"，导致重复。

```typescript
// 错误的逻辑
if (e.ctrlKey) keys.push("Control");  // 添加了 Control
const mainKey = getKeyName(e.key, e.code);  // 返回 RightControl
keys.push(mainKey);  // 又添加了 RightControl
// 结果：["Control", "RightControl"] → "Ctrl + 右Ctrl"
```

### 问题2：无法输入组合键（核心问题）

**原因**：当用户按下修饰键时，代码立即调用 `handleKeyPress` 并完成录制，导致输入框失焦，后续按键无法被捕获。

```typescript
// 错误的逻辑
const handleKeyDown = (e) => {
  const mainKey = getKeyName(e.key, e.code);
  const isModifierKey = ["LeftControl", "RightControl", ...].includes(mainKey);
  
  if (isModifierKey) {
    handleKeyPress(mainKey);  // 立即完成录制
    return;  // 直接返回
  }
  // ... 组合键逻辑永远不会执行
}
```

**执行流程**：
1. 用户按下左Alt
2. `handleKeyDown` 触发，识别为修饰键
3. 调用 `handleKeyPress("LeftAlt")`
4. 300ms 后 `finishRecording` 被调用
5. 设置 `setIsRecording(false)`，输入框失焦
6. 用户继续按空格，但 `isRecording` 已经是 false，事件被忽略
7. 结果：只能录入单个修饰键

### 问题3：左右区分不完整

**原因**：在组合键场景中，当用户按下 **左Alt + 空格** 时：
- 第一次按下左Alt：`e.code === "AltLeft"`，但没有记录到状态中
- 第二次按下空格：`e.altKey === true`，但无法判断是左Alt还是右Alt

原代码尝试通过 `e.ctrlKey`、`e.altKey` 等布尔值来判断修饰键，但这些属性不区分左右，只能知道"某个Alt键被按下"，无法知道是左还是右。

## 解决方案

### 核心思路

**关键点**：修饰键按下时不立即完成录制，而是等待用户是否会按下其他键。

- 如果用户只按修饰键然后松开 → 记录为单键
- 如果用户按住修饰键并按下其他键 → 记录为组合键

### 实现细节

#### 1. 状态管理

```typescript
const [activeModifiers, setActiveModifiers] = useState<Map<string, string>>(new Map());
const [hasNonModifierKey, setHasNonModifierKey] = useState(false);
```

- `activeModifiers`：Map 结构，key 为修饰键类型（"ctrl"/"shift"/"alt"/"meta"），value 为具体的左右键名（"LeftControl"/"RightControl"等）
- `hasNonModifierKey`：标记是否按下过非修饰键

#### 2. handleKeyDown 逻辑

```typescript
const handleKeyDown = (e: React.KeyboardEvent) => {
  // 1. 更新 activeModifiers，记录修饰键的左右信息
  const newModifiers = new Map(activeModifiers);
  
  if (e.code === "ControlLeft") {
    newModifiers.set("ctrl", "LeftControl");
  } else if (e.code === "ControlRight") {
    newModifiers.set("ctrl", "RightControl");
  }
  // ... 其他修饰键同理
  
  setActiveModifiers(newModifiers);
  
  // 2. 判断是否为修饰键本身
  const mainKey = getKeyName(e.key, e.code);
  const isModifierKey = ["LeftControl", "RightControl", ...].includes(mainKey);
  
  if (isModifierKey) {
    // 修饰键本身，不完成录制，直接返回
    return;
  }
  
  // 3. 非修饰键，标记并构建组合键
  setHasNonModifierKey(true);
  
  const keys: string[] = [];
  
  // 从 activeModifiers 中读取已按下的修饰键
  if (e.ctrlKey && newModifiers.has("ctrl")) {
    keys.push(newModifiers.get("ctrl")!);
  }
  // ... 其他修饰键同理
  
  // 添加主键
  keys.push(mainKey);
  
  // 完成录制
  handleKeyPress(keys.join("+"));
}
```

**关键改进**：
- 使用 `e.code` 精确判断左右修饰键（"ControlLeft"/"ControlRight"等）
- 修饰键按下时只记录状态，不完成录制
- 非修饰键按下时才构建组合键并完成录制

#### 3. handleKeyUp 逻辑

```typescript
const handleKeyUp = (e: React.KeyboardEvent) => {
  // 1. 获取松开的键
  const releasedKey = getKeyName(e.key, e.code);
  const isModifierKey = ["LeftControl", "RightControl", ...].includes(releasedKey);
  
  // 2. 如果松开的是修饰键，且没有按过其他键，记录为单键
  if (isModifierKey && !hasNonModifierKey) {
    handleKeyPress(releasedKey);
  }
  
  // 3. 从 activeModifiers 中移除该修饰键
  const newModifiers = new Map(activeModifiers);
  
  if (e.code === "ControlLeft" || e.code === "ControlRight") {
    newModifiers.delete("ctrl");
  }
  // ... 其他修饰键同理
  
  setActiveModifiers(newModifiers);
}
```

**关键改进**：
- 只有在 `hasNonModifierKey === false` 时才记录修饰键为单键
- 确保组合键场景下不会重复录制

#### 4. 状态重置

```typescript
const finishRecording = (shortcut: string, count: number) => {
  // ... 完成录制逻辑
  setHasNonModifierKey(false);  // 重置标记
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
```

### 连按支持

连按功能通过 `handleKeyPress` 中的定时器实现：

```typescript
const handleKeyPress = (shortcut: string) => {
  if (pressTimer) {
    clearTimeout(pressTimer);
  }
  
  // 检查是否为连按
  if (lastKey === shortcut) {
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
    // 不同按键，开始新的录制
    setLastKey(shortcut);
    setPressCount(1);
    
    const timer = setTimeout(() => {
      finishRecording(shortcut, 1);
    }, 300);
    setPressTimer(timer);
  }
};
```

**连按格式**：
- 单次按键：`"LeftAlt"` → 显示 "左Alt"
- 连按：`"LeftAlt*2"` → 显示 "左Alt×2"

### 显示格式化

更新 `formatShortcutDisplay` 函数支持连按和左右区分：

```typescript
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
    "Control": "Ctrl",
    "RightControl": "右Ctrl",
    "LeftControl": "左Ctrl",
    "Shift": "Shift",
    "RightShift": "右Shift",
    "LeftShift": "左Shift",
    "Alt": "Alt",
    "RightAlt": "右Alt",
    "LeftAlt": "左Alt",
    "Meta": "Win",
    "Space": "空格",
    // ... 其他特殊键
  };
  
  return shortcut
    .split("+")
    .map(key => keyMap[key] || key)
    .join(" + ");
}
```

## 执行流程示例

### 场景1：单独按下左Alt

1. **keydown(LeftAlt)**：
   - `e.code === "AltLeft"`
   - `activeModifiers.set("alt", "LeftAlt")`
   - `isModifierKey === true`，直接 return
   - `hasNonModifierKey` 保持 false

2. **keyup(LeftAlt)**：
   - `releasedKey === "LeftAlt"`
   - `isModifierKey === true && hasNonModifierKey === false`
   - 调用 `handleKeyPress("LeftAlt")`
   - 300ms 后完成录制

**结果**：记录为 "LeftAlt"，显示 "左Alt"

### 场景2：按下左Alt + 空格

1. **keydown(LeftAlt)**：
   - `activeModifiers.set("alt", "LeftAlt")`
   - `isModifierKey === true`，return
   - `hasNonModifierKey` 保持 false

2. **keydown(Space)**：
   - `mainKey === "Space"`
   - `isModifierKey === false`
   - `setHasNonModifierKey(true)`
   - 从 `activeModifiers` 读取 "LeftAlt"
   - 构建 `["LeftAlt", "Space"]`
   - 调用 `handleKeyPress("LeftAlt+Space")`
   - 300ms 后完成录制

3. **keyup(Space)**：
   - 不处理

4. **keyup(LeftAlt)**：
   - `hasNonModifierKey === true`，不处理
   - 从 `activeModifiers` 中移除 "alt"

**结果**：记录为 "LeftAlt+Space"，显示 "左Alt + 空格"

### 场景3：快速双击左Alt

1. **第一次 keydown → keyup**：
   - 调用 `handleKeyPress("LeftAlt")`
   - `lastKey = "LeftAlt"`，`pressCount = 1`
   - 启动 300ms 定时器

2. **第二次 keydown → keyup**（在 300ms 内）：
   - 调用 `handleKeyPress("LeftAlt")`
   - `lastKey === "LeftAlt"`，检测到相同键
   - `pressCount = 2`
   - 重新启动 300ms 定时器

3. **300ms 后**：
   - 调用 `finishRecording("LeftAlt", 2)`
   - 生成 `"LeftAlt*2"`

**结果**：记录为 "LeftAlt*2"，显示 "左Alt×2"

## 技术要点

### 1. 使用 e.code 而不是 e.key

- `e.key`：返回按键的逻辑值，不区分左右（如 "Control"、"Alt"）
- `e.code`：返回按键的物理位置，区分左右（如 "ControlLeft"、"ControlRight"）

### 2. Map 数据结构的优势

使用 `Map<string, string>` 而不是 `Set<string>` 的原因：
- 需要同时记录修饰键类型和具体的左右信息
- 当用户按住左Ctrl再按右Ctrl时，只保留最后一个Ctrl的信息
- 避免同时存在 "LeftControl" 和 "RightControl" 导致的冲突

### 3. 状态管理的时机

- `activeModifiers`：在 keydown 时更新，在 keyup 时清除
- `hasNonModifierKey`：在按下非修饰键时设置为 true，在完成录制或重新聚焦时重置为 false

### 4. 事件处理顺序

正确的事件处理顺序：
1. `preventDefault()` 和 `stopPropagation()` 阻止默认行为
2. 更新状态（`activeModifiers`、`hasNonModifierKey`）
3. 判断是否完成录制
4. 调用 `handleKeyPress` 或直接返回

## 相关文件

- `src/components/ui/shortcut-input.tsx` - 快捷键输入组件（核心逻辑）
- `src/utils/shortcut.ts` - 快捷键格式化工具
- `src/pages/SettingsPage.tsx` - 设置页面（使用组件）
- `src-tauri/src/data.rs` - 后端默认配置
- `src/types/config.ts` - 配置类型定义

## 参考资料

- [React Synthetic Event distinguish Left and Right click events](https://stackoverflow.com/questions/31110184/react-synthetic-event-distinguish-left-and-right-click-events)
- [Key values for keyboard events - MDN](https://developer.mozilla.org/en-US/docs/Web/API/UI_Events/Keyboard_event_key_values)
- [Can javascript tell the difference between left and right shift key?](http://stackoverflow.com/questions/22029033)
- [KeyboardEvent.code - MDN](https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent/code)

## 测试验证

### 测试用例

1. **单键测试**：
   - 按下左Ctrl → 显示 "左Ctrl" ✓
   - 按下右Ctrl → 显示 "右Ctrl" ✓
   - 按下左Alt → 显示 "左Alt" ✓
   - 按下右Alt → 显示 "右Alt" ✓
   - 按下左Shift → 显示 "左Shift" ✓
   - 按下右Shift → 显示 "右Shift" ✓

2. **组合键测试**：
   - 左Ctrl + C → 显示 "左Ctrl + C" ✓
   - 右Ctrl + M → 显示 "右Ctrl + M" ✓
   - 左Alt + 空格 → 显示 "左Alt + 空格" ✓
   - 右Alt + Tab → 显示 "右Alt + Tab" ✓
   - 左Shift + A → 显示 "左Shift + A" ✓

3. **多修饰键组合**：
   - 左Ctrl + 左Shift + A → 显示 "左Ctrl + 左Shift + A" ✓
   - 右Ctrl + 右Alt + Delete → 显示 "右Ctrl + 右Alt + Delete" ✓

4. **连按测试**：
   - 快速双击空格 → 显示 "空格×2" ✓
   - 快速三击左Ctrl → 显示 "左Ctrl×3" ✓

5. **边界情况**：
   - 按下左Ctrl，不松开，按下C → 显示 "左Ctrl + C" ✓
   - 按下左Ctrl，松开，再按C → 先显示 "左Ctrl"，再显示 "C" ✓

## 后续优化建议

1. **Tauri 全局快捷键兼容性**：
   - 确认 Tauri 的 `global-shortcut` 插件是否支持连按语法
   - 如果不支持，在保存时给出提示或禁用连按功能

2. **快捷键冲突检测**：
   - 检测用户输入的快捷键是否与系统快捷键冲突
   - 提示用户选择其他快捷键

3. **自定义连按时间窗口**：
   - 允许用户配置连按检测的时间窗口（当前固定为 300ms）

4. **快捷键预设**：
   - 提供常用快捷键预设供用户快速选择
