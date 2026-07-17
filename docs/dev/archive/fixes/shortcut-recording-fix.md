# 快捷键录音问题修复说明

> **归档状态：** 历史快捷键录音修复记录。当前排查方法请见 [`../../../troubleshooting.md`](../../../troubleshooting.md)。

## 问题描述

用户按下快捷键后，录音完成但没有文字出现在光标处。

## 根本原因

前端缺少对后端 `recording-completed` 和 `recording-error` 事件的监听。

### 完整流程分析

1. **用户按下快捷键** → Rust 后端 `hotkey.rs` 捕获事件
2. **开始录音** → `recording.rs` 启动录音流
3. **用户松开/再按快捷键** → 停止录音
4. **后端发出事件** → `app.emit("recording-completed", result)` 
5. **❌ 前端未监听** → 事件丢失，录音文件未被处理
6. **结果** → 没有文字输出

## 修复内容

### 1. 添加事件监听 API 导入

**文件**: `src/main.tsx`

```typescript
import { listen } from "@tauri-apps/api/event";
```

### 2. 采用社区最佳实践实现事件监听

参考 [Tauri 官方文档 - 从 Rust 调用前端](https://v2.tauri.app/develop/calling-frontend/)，正确实现事件监听：

**关键点**：
- `listen()` 返回 `Promise<UnlistenFn>`，需要 `await`
- 在 `useEffect` 清理函数中调用 `unlisten()` 防止内存泄漏
- 使用异步初始化函数包装所有异步操作

```typescript
useEffect(() => {
  let unlistenCompleted: (() => void) | null = null;
  let unlistenError: (() => void) | null = null;

  async function initialize() {
    // 加载数据...
    
    // 正确的事件监听方式
    unlistenCompleted = await listen("recording-completed", handler);
    unlistenError = await listen("recording-error", handler);
  }

  initialize();

  // 清理监听器
  return () => {
    if (unlistenCompleted) unlistenCompleted();
    if (unlistenError) unlistenError();
  };
}, []);
```

### 3. 添加 `recording-completed` 事件监听器

当快捷键录音完成时：

1. 调用 `process_recording_file` 处理录音文件
2. 执行 ASR 语音识别
3. 调用 LLM 进行文本整理
4. 自动调用 `output_text` 输出到光标位置
5. 更新历史记录和统计数据

### 4. 添加 `recording-error` 事件监听器

处理录音过程中的错误：

- 麦克风权限缺失
- 未找到音频输入设备
- 其他录音错误

## 验证步骤

### 前置条件

1. 在设置页配置 ASR API Key（智谱 GLM）
2. 在设置页配置 OpenAI API Key
3. 确保麦克风权限已开启
4. 确保麦克风设备已连接

### 测试场景 1：长按模式（默认 Ctrl+Shift+R）

1. 打开任意文本编辑器（记事本、VS Code 等）
2. 将光标放在需要输入的位置
3. **按住** `Ctrl+Shift+R`
4. 说话（例如："这是一个测试"）
5. **松开** 快捷键
6. **预期结果**：
   - 应用显示"录音完成，正在执行 ASR 识别..."
   - 几秒后显示"语音处理完成，正在自动输出..."
   - 文字自动出现在光标位置
   - 弹出提示："已自动输入到光标位置"

### 测试场景 2：切换模式（默认 Alt+Space）

1. 打开任意文本编辑器
2. 将光标放在需要输入的位置
3. **按一次** `Alt+Space` 开始录音
4. 说话（例如："这是第二个测试"）
5. **再按一次** `Alt+Space` 停止录音
6. **预期结果**：同场景 1

### 测试场景 3：错误处理

#### 3.1 麦克风权限缺失

1. 在系统设置中关闭应用的麦克风权限
2. 按下快捷键
3. **预期结果**：弹出错误提示"麦克风权限缺失，请在系统设置中开启麦克风权限"

#### 3.2 麦克风未连接

1. 拔掉麦克风或禁用音频输入设备
2. 按下快捷键
3. **预期结果**：弹出错误提示"未找到麦克风设备，请检查麦克风连接"

### 测试场景 4：不同输出模式

#### 4.1 键盘注入模式（优先）

- 应该直接在光标位置输入文字
- 提示："已自动输入到光标位置"

#### 4.2 剪贴板粘贴模式（降级）

- 如果键盘注入失败，自动降级到剪贴板粘贴
- 提示："已通过剪贴板输入"

#### 4.3 手动粘贴模式（兜底）

- 如果自动粘贴也失败，至少复制到剪贴板
- 提示："已复制到剪贴板，请手动粘贴 (Ctrl+V)"

## 技术细节

### 事件流程图

```
用户按下快捷键
    ↓
Rust: hotkey.rs 捕获事件
    ↓
Rust: recording.rs 开始录音
    ↓
用户松开/再按快捷键
    ↓
Rust: recording.rs 停止录音
    ↓
Rust: app.emit("recording-completed", { file_path, duration_ms })
    ↓
前端: listen("recording-completed") 接收事件
    ↓
前端: invoke("process_recording_file")
    ↓
Rust: pipeline.rs 处理录音
    ├─ asr.rs: 语音识别
    └─ text_polish.rs: 文本整理
    ↓
前端: 收到处理结果
    ↓
前端: invoke("output_text")
    ↓
Rust: output.rs 输出文字
    ├─ 尝试键盘注入
    ├─ 降级到剪贴板粘贴
    └─ 兜底复制到剪贴板
    ↓
文字出现在光标位置 ✓
```

### 关键代码位置

- **后端事件发送**: `src-tauri/src/hotkey.rs:281` 和 `hotkey.rs:323`
- **前端事件监听**: `src/main.tsx:141-207`
- **录音处理**: `src-tauri/src/pipeline.rs:188-238`
- **文字输出**: `src-tauri/src/output.rs:22-48`

## 已知限制

1. **输出延迟**：从录音完成到文字出现需要 2-5 秒（取决于网络和 API 响应速度）
2. **特殊字符**：某些应用可能不支持键盘注入，会自动降级到剪贴板模式
3. **焦点切换**：如果在处理过程中切换了窗口，文字可能输出到错误的位置

## 后续优化建议

1. 添加录音指示器窗口，显示实时状态
2. 支持取消正在进行的处理
3. 添加音频波形可视化
4. 支持自定义输出模式优先级

## 参考资料

本次修复参考了以下社区资源和官方文档：

- [Tauri v2 官方文档 - 从 Rust 调用前端](https://v2.tauri.app/develop/calling-frontend/)
- [Tauri Event API 参考](https://v2.tauri.app/reference/javascript/api/namespaceevent/)
- [Tauri Global Shortcut 插件](https://tauri.app/plugin/global-shortcut/)
- [Stack Overflow - 在 Tauri with React 中正确清理事件监听器](https://stackoverflow.com/questions/76639536/in-tauri-with-react-how-do-you-properly-clean-up-listening-to-events-on-unmount)

### 关键技术点

1. **事件系统最佳实践**：
   - `listen()` 返回 `Promise<UnlistenFn>`，必须 `await`
   - 组件卸载时调用 `unlisten()` 防止内存泄漏
   - 事件 payload 必须实现 `Serialize` 和 `Clone`

2. **React + Tauri 集成模式**：
   - 在 `useEffect` 中使用异步初始化函数
   - 将 unlisten 函数存储在闭包变量中
   - 在清理函数中检查并调用 unlisten

3. **事件 vs Channels vs eval**：
   - 事件系统：适合简单通知、状态更新（本项目使用）
   - Channels：适合流式数据、大量消息
   - eval：适合简单脚本执行
