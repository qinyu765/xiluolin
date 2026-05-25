# Keytap 集成方案：支持左右修饰键区分

## 📋 方案概述

集成 `keytap` Rust 库以支持区分左右修饰键（如右 Ctrl、左 Alt），实现更灵活的全局快捷键配置。

---

## 🎯 目标

- 支持用户配置单独的左/右修饰键作为快捷键（如：单独的右 Ctrl 键）
- 保持现有组合键快捷键功能
- 跨平台兼容（Windows、macOS、Linux）

---

## 📊 技术调研

### Keytap 库特性

**优势：**
- ✅ 专门设计用于保留左右修饰键身份
- ✅ 跨平台支持（Windows、macOS、Linux）
- ✅ 只读监听模式，不干扰系统
- ✅ 类型安全的事件处理
- ✅ 优雅的错误处理

**限制：**
- ⚠️ 需要系统权限（macOS 需要辅助功能权限，Linux 需要 input 设备访问权限）
- ⚠️ 可能与 Tauri 的事件循环产生冲突
- ⚠️ 增加应用体积（约 50-100KB）
- ⚠️ 需要处理权限请求的用户体验

### 与现有方案对比

| 特性 | 当前方案（global-hotkey） | Keytap 方案 |
|------|--------------------------|-------------|
| 左右键区分 | ❌ 不支持 | ✅ 支持 |
| 组合键支持 | ✅ 完整支持 | ⚠️ 需要手动实现 |
| 系统权限 | ✅ 无需额外权限 | ⚠️ 需要额外权限 |
| 跨平台兼容 | ✅ 完全兼容 | ✅ 完全兼容 |
| 实现复杂度 | ✅ 简单 | ⚠️ 中等 |
| 维护成本 | ✅ 低 | ⚠️ 中等 |

---

## ✅ 已完成的准备工作

在实施 keytap 集成之前，项目已经完成了以下基础设施，这些将大大简化集成工作：

### 1. 完整的快捷键系统 (`src-tauri/src/hotkey.rs`)

**已实现功能：**
- ✅ 快捷键状态管理（`HotkeyState`）
- ✅ 长按模式和切换模式支持
- ✅ 快捷键注册/注销接口
- ✅ 事件处理和录音触发逻辑
- ✅ 详细的日志输出用于调试

**核心结构：**
```rust
pub struct HotkeyState {
    pub longpress_registered: bool,
    pub toggle_registered: bool,
    pub longpress_shortcut: Option<String>,
    pub toggle_shortcut: Option<String>,
    pub is_recording_via_hotkey: bool,
}

pub enum RecordingMode {
    LongPress,
    Toggle,
}
```

**关键接口：**
- `register_hotkey()` - 注册单个快捷键
- `register_both_hotkeys()` - 同时注册长按和切换模式
- `unregister_hotkey()` - 注销所有快捷键
- `handle_hotkey_event()` - 统一的事件处理入口
- `handle_long_press_mode()` - 长按模式处理
- `handle_toggle_mode()` - 切换模式处理

### 2. 录音指示器窗口 (`src-tauri/src/indicator.rs`)

**已实现功能：**
- ✅ 浮动窗口创建和管理
- ✅ 显示/隐藏接口
- ✅ 跨平台路径处理
- ✅ 透明窗口、始终置顶、无边框

**核心接口：**
```rust
pub fn show_indicator(app: &AppHandle) -> Result<(), String>
pub fn hide_indicator(app: &AppHandle) -> Result<(), String>
```

**窗口特性：**
- 尺寸：200x80 像素
- 样式：渐变紫色背景、脉动红点、波形动画
- 行为：自动居中、跳过任务栏、始终置顶

### 3. 录音状态集成

**已集成到快捷键处理：**
```rust
// 长按模式 - 按下时
let _ = crate::indicator::show_indicator(app);
match start_recording(recording_state, app.clone()).await {
    Ok(_) => { /* 更新状态 */ }
    Err(e) => { 
        let _ = crate::indicator::hide_indicator(app);
    }
}

// 长按模式 - 松开时
match stop_recording(recording_state).await {
    Ok(result) => {
        let _ = crate::indicator::hide_indicator(app);
        let _ = app.emit("recording-completed", result);
    }
}
```

### 4. 配置系统支持

**数据模型 (`src-tauri/src/data.rs`)：**
```rust
pub struct AppConfig {
    pub longpress_shortcut: String,
    pub toggle_shortcut: String,
    pub recording_mode: String,
    // ... 其他配置
}

pub fn default_app_config() -> AppConfig {
    AppConfig {
        longpress_shortcut: "CommandOrControl+Shift+R".to_string(),
        toggle_shortcut: "Alt+Space".to_string(),
        // ...
    }
}
```

**配置持久化：**
- 使用 `tauri-plugin-store` 存储配置
- 自动加载和保存
- 热更新快捷键（配置更新时自动重新注册）

### 5. 前端配置界面 (`src/components/dialogs/AppSettingsDialog.tsx`)

**已实现功能：**
- ✅ 快捷键输入框
- ✅ 录音模式选择（长按/切换）
- ✅ 配置保存和验证
- ✅ 用户友好的提示文本

**当前支持的快捷键格式：**
- 组合键：`CommandOrControl+Shift+R`、`Alt+Space`
- 修饰符：`CommandOrControl`、`Alt`、`Shift`、`Super`
- 功能键：`F1`-`F24`
- 字母数字：`KeyA`-`KeyZ`、`Digit0`-`Digit9`

### 6. 应用初始化流程 (`src-tauri/src/lib.rs`)

**启动时自动注册快捷键：**
```rust
.setup(|app| {
    app.manage(Arc::new(Mutex::new(hotkey::HotkeyState::default())));
    
    let app_handle = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        match data::read_app_config(app_handle.clone()) {
            Ok(config) => {
                let longpress = if config.longpress_shortcut.is_empty() {
                    None
                } else {
                    Some(config.longpress_shortcut.clone())
                };
                let toggle = if config.toggle_shortcut.is_empty() {
                    None
                } else {
                    Some(config.toggle_shortcut.clone())
                };
                match hotkey::register_both_hotkeys(
                    app_handle,
                    longpress,
                    toggle,
                ).await {
                    Ok(_) => println!("快捷键注册成功"),
                    Err(e) => eprintln!("快捷键注册失败: {}", e),
                }
            }
            Err(e) => eprintln!("读取配置失败: {}", e),
        }
    });
    
    Ok(())
})
```

### 7. 调试和日志系统

**已添加详细日志：**
- 快捷键注册过程
- 事件触发和处理
- 录音启动/停止
- 指示器显示/隐藏
- 错误和异常情况

**示例日志输出：**
```
读取到配置: longpress=CommandOrControl+Shift+R, toggle=Alt+Space
register_both_hotkeys 被调用: longpress=Some("CommandOrControl+Shift+R"), toggle=Some("Alt+Space")
尝试注册长按模式快捷键: CommandOrControl+Shift+R
长按模式快捷键注册成功: CommandOrControl+Shift+R
快捷键事件触发: mode=LongPress, state=Pressed
长按模式: 按键按下，准备开始录音
指示器 HTML 路径: "D:\\...\\indicator.html"
长按模式: 录音启动成功
```

---

## 🏗️ 实现方案

### 架构设计

```
┌─────────────────────────────────────────────────────────┐
│                    Tauri 应用层                          │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌──────────────────┐      ┌──────────────────┐        │
│  │  Hotkey Manager  │      │  Keytap Listener │        │
│  │  (组合键)         │      │  (单键)           │        │
│  └────────┬─────────┘      └────────┬─────────┘        │
│           │                         │                   │
│           ├─────────────┬───────────┤                   │
│           │             │           │                   │
│  ┌────────▼─────┐  ┌────▼────┐  ┌──▼──────┐           │
│  │ global-hotkey│  │ keytap  │  │ 事件合并 │           │
│  │   插件        │  │  库     │  │  层      │           │
│  └──────────────┘  └─────────┘  └──────────┘           │
│                                                          │
├─────────────────────────────────────────────────────────┤
│                    操作系统层                            │
└─────────────────────────────────────────────────────────┘
```

### 核心模块

#### 1. Keytap 监听器模块 (`src-tauri/src/keytap_listener.rs`)

```rust
use keytap::{Listener, Event, Key};
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{AppHandle, Manager};

pub struct KeytapListener {
    listener: Option<Listener>,
    enabled: bool,
    registered_keys: Vec<Key>,
}

impl KeytapListener {
    pub fn new() -> Self {
        Self {
            listener: None,
            enabled: false,
            registered_keys: Vec::new(),
        }
    }

    /// 启动监听器
    pub fn start(&mut self, app: AppHandle) -> Result<(), String> {
        // 检查权限
        if !self.check_permissions() {
            return Err("需要系统权限才能监听键盘事件".to_string());
        }

        let listener = Listener::new()
            .map_err(|e| format!("创建监听器失败: {}", e))?;

        let app_clone = app.clone();
        listener.on_event(move |event| {
            handle_keytap_event(&app_clone, event);
        });

        self.listener = Some(listener);
        self.enabled = true;
        Ok(())
    }

    /// 停止监听器
    pub fn stop(&mut self) {
        self.listener = None;
        self.enabled = false;
    }

    /// 注册单键快捷键
    pub fn register_key(&mut self, key: Key) {
        if !self.registered_keys.contains(&key) {
            self.registered_keys.push(key);
        }
    }

    /// 注销单键快捷键
    pub fn unregister_key(&mut self, key: Key) {
        self.registered_keys.retain(|k| k != &key);
    }

    /// 检查系统权限
    fn check_permissions(&self) -> bool {
        #[cfg(target_os = "macos")]
        {
            // macOS 需要检查辅助功能权限
            // 这里需要调用系统 API
            true // 简化处理
        }

        #[cfg(target_os = "linux")]
        {
            // Linux 需要检查 /dev/input 访问权限
            std::fs::metadata("/dev/input").is_ok()
        }

        #[cfg(target_os = "windows")]
        {
            // Windows 通常不需要额外权限
            true
        }
    }
}

/// 处理 keytap 事件
fn handle_keytap_event(app: &AppHandle, event: Event) {
    let state = app.state::<Arc<Mutex<KeytapListener>>>();
    
    tokio::spawn(async move {
        let listener = state.lock().await;
        
        match event {
            Event::KeyPress(key) => {
                if listener.registered_keys.contains(&key) {
                    // 触发录音开始
                    handle_key_press(app, key).await;
                }
            }
            Event::KeyRelease(key) => {
                if listener.registered_keys.contains(&key) {
                    // 触发录音停止
                    handle_key_release(app, key).await;
                }
            }
        }
    });
}

async fn handle_key_press(app: &AppHandle, key: Key) {
    use crate::recording::{start_recording, RecordingState};
    use crate::indicator;

    println!("Keytap: 按键按下 {:?}", key);
    
    // 显示录音指示器
    let _ = indicator::show_indicator(app);
    
    // 开始录音
    let recording_state = app.state::<RecordingState>();
    match start_recording(recording_state, app.clone()).await {
        Ok(_) => println!("Keytap: 录音启动成功"),
        Err(e) => {
            eprintln!("Keytap: 启动录音失败: {:?}", e);
            let _ = indicator::hide_indicator(app);
        }
    }
}

async fn handle_key_release(app: &AppHandle, key: Key) {
    use crate::recording::{stop_recording, RecordingState};
    use crate::indicator;

    println!("Keytap: 按键松开 {:?}", key);
    
    // 停止录音
    let recording_state = app.state::<RecordingState>();
    match stop_recording(recording_state).await {
        Ok(result) => {
            println!("Keytap: 录音停止成功");
            let _ = indicator::hide_indicator(app);
            let _ = app.emit("recording-completed", result);
        }
        Err(e) => {
            eprintln!("Keytap: 停止录音失败: {:?}", e);
            let _ = indicator::hide_indicator(app);
        }
    }
}
```

#### 2. 统一快捷键管理器 (`src-tauri/src/unified_hotkey.rs`)

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::AppHandle;

pub enum HotkeyType {
    Combination(String),  // 组合键，如 "Ctrl+Shift+R"
    SingleKey(String),    // 单键，如 "RightControl"
}

pub struct UnifiedHotkeyManager {
    global_hotkey_registered: Vec<String>,
    keytap_registered: Vec<String>,
}

impl UnifiedHotkeyManager {
    pub fn new() -> Self {
        Self {
            global_hotkey_registered: Vec::new(),
            keytap_registered: Vec::new(),
        }
    }

    /// 注册快捷键（自动选择合适的后端）
    pub async fn register(
        &mut self,
        app: AppHandle,
        shortcut: String,
        mode: String,
    ) -> Result<(), String> {
        let hotkey_type = self.parse_hotkey_type(&shortcut);

        match hotkey_type {
            HotkeyType::Combination(_) => {
                // 使用 global-hotkey
                crate::hotkey::register_hotkey(app, shortcut.clone(), mode).await?;
                self.global_hotkey_registered.push(shortcut);
            }
            HotkeyType::SingleKey(key) => {
                // 使用 keytap
                self.register_single_key(app, key.clone()).await?;
                self.keytap_registered.push(key);
            }
        }

        Ok(())
    }

    /// 解析快捷键类型
    fn parse_hotkey_type(&self, shortcut: &str) -> HotkeyType {
        // 检查是否包含修饰符
        if shortcut.contains("+") {
            HotkeyType::Combination(shortcut.to_string())
        } else if self.is_single_modifier_key(shortcut) {
            HotkeyType::SingleKey(shortcut.to_string())
        } else {
            HotkeyType::Combination(shortcut.to_string())
        }
    }

    /// 判断是否为单个修饰键
    fn is_single_modifier_key(&self, key: &str) -> bool {
        matches!(
            key,
            "RightControl" | "LeftControl" | 
            "RightAlt" | "LeftAlt" | 
            "RightShift" | "LeftShift" |
            "RightSuper" | "LeftSuper"
        )
    }

    /// 注册单键快捷键
    async fn register_single_key(
        &self,
        app: AppHandle,
        key: String,
    ) -> Result<(), String> {
        let keytap_state = app.state::<Arc<Mutex<crate::keytap_listener::KeytapListener>>>();
        let mut listener = keytap_state.lock().await;

        let keytap_key = self.convert_to_keytap_key(&key)?;
        listener.register_key(keytap_key);

        Ok(())
    }

    /// 转换为 keytap Key 枚举
    fn convert_to_keytap_key(&self, key: &str) -> Result<keytap::Key, String> {
        use keytap::Key;

        match key {
            "RightControl" => Ok(Key::RightControl),
            "LeftControl" => Ok(Key::LeftControl),
            "RightAlt" => Ok(Key::RightAlt),
            "LeftAlt" => Ok(Key::LeftAlt),
            "RightShift" => Ok(Key::RightShift),
            "LeftShift" => Ok(Key::LeftShift),
            _ => Err(format!("不支持的单键: {}", key)),
        }
    }
}
```

#### 3. Tauri 命令接口

```rust
#[tauri::command]
pub async fn register_unified_hotkey(
    app: AppHandle,
    shortcut: String,
    mode: String,
) -> Result<(), String> {
    let manager = app.state::<Arc<Mutex<UnifiedHotkeyManager>>>();
    let mut manager = manager.lock().await;
    manager.register(app, shortcut, mode).await
}

#[tauri::command]
pub async fn request_keyboard_permission(app: AppHandle) -> Result<bool, String> {
    #[cfg(target_os = "macos")]
    {
        // 请求 macOS 辅助功能权限
        // 需要调用系统 API
        Ok(true)
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(true)
    }
}
```

#### 4. 前端配置界面更新

```typescript
// src/components/dialogs/AppSettingsDialog.tsx

// 添加快捷键类型选择
<div className="grid gap-2">
  <Label htmlFor="longpress-shortcut-type">长按模式快捷键类型</Label>
  <Select
    value={shortcutType}
    onValueChange={setShortcutType}
  >
    <SelectTrigger>
      <SelectValue />
    </SelectTrigger>
    <SelectContent>
      <SelectItem value="combination">组合键（推荐）</SelectItem>
      <SelectItem value="single">单键（需要额外权限）</SelectItem>
    </SelectContent>
  </Select>
</div>

{shortcutType === "single" && (
  <div className="rounded-lg border border-yellow-500 bg-yellow-50 p-3">
    <p className="text-sm text-yellow-800">
      ⚠️ 单键模式需要系统权限：
      <ul className="mt-2 list-disc pl-5">
        <li>macOS: 辅助功能权限</li>
        <li>Linux: /dev/input 访问权限</li>
        <li>Windows: 无需额外权限</li>
      </ul>
    </p>
    <Button
      size="sm"
      variant="outline"
      className="mt-2"
      onClick={requestPermission}
    >
      请求权限
    </Button>
  </div>
)}

<div className="grid gap-2">
  <Label htmlFor="longpress-shortcut">长按模式快捷键</Label>
  <Input
    id="longpress-shortcut"
    value={appConfig?.longpress_shortcut ?? ""}
    onChange={(event) => onConfigChange({
      ...appConfig!,
      longpress_shortcut: event.target.value
    })}
    placeholder={
      shortcutType === "combination" 
        ? "例如：CommandOrControl+Shift+R" 
        : "例如：RightControl"
    }
  />
  <p className="text-xs text-muted-foreground">
    {shortcutType === "combination"
      ? "组合键格式：CommandOrControl+Shift+R 或 Alt+Space"
      : "单键格式：RightControl, LeftAlt, RightShift 等"}
  </p>
</div>
```

---

## 📈 工作量评估

### 开发任务分解

| 任务 | 预计工时 | 难度 | 优先级 |
|------|---------|------|--------|
| 1. 添加 keytap 依赖并测试基本功能 | 2h | 低 | P0 |
| 2. 实现 KeytapListener 模块 | 4h | 中 | P0 |
| 3. 实现 UnifiedHotkeyManager | 3h | 中 | P0 |
| 4. 集成到现有 hotkey 系统 | 3h | 中 | P0 |
| 5. 实现权限检查和请求 | 4h | 高 | P1 |
| 6. 更新前端配置界面 | 3h | 低 | P1 |
| 7. 跨平台测试（Windows/macOS/Linux） | 6h | 高 | P1 |
| 8. 处理边缘情况和错误 | 4h | 中 | P2 |
| 9. 编写文档和用户指南 | 2h | 低 | P2 |
| 10. 性能优化和内存泄漏检查 | 3h | 中 | P2 |

**总计：34 小时（约 4-5 个工作日）**

### 风险评估

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|---------|
| keytap 与 Tauri 事件循环冲突 | 中 | 高 | 在独立线程中运行 keytap，使用消息传递 |
| macOS 权限请求体验差 | 高 | 中 | 提供清晰的引导和说明 |
| Linux 权限问题 | 中 | 中 | 提供 udev 规则配置指南 |
| 性能开销 | 低 | 低 | 只在需要时启用 keytap |
| 维护成本增加 | 高 | 中 | 充分的文档和测试覆盖 |

---

## 🔄 实施步骤

### Phase 1: 基础集成（1-2 天）
1. ✅ 添加 keytap 依赖
2. ✅ 实现基本的 KeytapListener
3. ✅ 测试单键监听功能

### Phase 2: 系统集成（1-2 天）
4. ✅ 实现 UnifiedHotkeyManager
5. ✅ 集成到现有快捷键系统
6. ✅ 更新数据模型和配置

### Phase 3: 用户体验（1 天）
7. ✅ 实现权限检查和请求
8. ✅ 更新前端配置界面
9. ✅ 添加用户引导

### Phase 4: 测试和优化（1 天）
10. ✅ 跨平台测试
11. ✅ 性能优化
12. ✅ 文档编写

---

## 💰 成本收益分析

### 成本
- **开发时间**：34 小时
- **测试时间**：额外 10 小时
- **维护成本**：持续
- **应用体积增加**：~50-100KB
- **用户体验复杂度**：权限请求流程

### 收益
- ✅ 支持单键快捷键（如单独的右 Ctrl）
- ✅ 更灵活的快捷键配置
- ✅ 满足特定用户需求
- ⚠️ 但大多数用户可能不需要此功能

### ROI 评估
- **投入产出比**：低-中
- **用户需求强度**：待验证
- **技术债务风险**：中

---

## 🎯 替代方案对比

### 方案 A：保持现状（组合键）
- ✅ 零开发成本
- ✅ 稳定可靠
- ✅ 跨平台兼容
- ❌ 不支持单键快捷键

### 方案 B：集成 keytap（本方案）
- ⚠️ 34 小时开发成本
- ⚠️ 需要系统权限
- ⚠️ 增加维护负担
- ✅ 支持单键快捷键

### 方案 C：混合方案（推荐）
- ✅ 默认使用组合键
- ✅ 提供"高级模式"选项
- ✅ 仅在用户明确需要时启用 keytap
- ⚠️ 需要 20 小时开发（简化版）

---

## 🚀 快速集成指南

基于已完成的基础设施，集成 keytap 的步骤已大大简化：

### Step 1: 添加依赖 (5 分钟)

```toml
# src-tauri/Cargo.toml
[dependencies]
keytap = "0.4"
```

### Step 2: 创建 KeytapListener 模块 (1 小时)

创建 `src-tauri/src/keytap_listener.rs`，复用现有的录音和指示器逻辑：

```rust
use keytap::{Listener, Event, Key};
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::{AppHandle, Manager};

pub struct KeytapListener {
    listener: Option<Listener>,
    enabled: bool,
    registered_keys: Vec<Key>,
}

// 关键：复用现有的录音处理逻辑
async fn handle_key_press(app: &AppHandle, key: Key) {
    use crate::recording::{start_recording, RecordingState};
    use crate::indicator;

    // 直接调用现有的指示器和录音接口
    let _ = indicator::show_indicator(app);
    let recording_state = app.state::<RecordingState>();
    match start_recording(recording_state, app.clone()).await {
        Ok(_) => println!("Keytap: 录音启动成功"),
        Err(e) => {
            eprintln!("Keytap: 启动录音失败: {:?}", e);
            let _ = indicator::hide_indicator(app);
        }
    }
}
```

### Step 3: 扩展 HotkeyState (30 分钟)

在现有的 `hotkey.rs` 中添加 keytap 支持：

```rust
pub struct HotkeyState {
    pub longpress_registered: bool,
    pub toggle_registered: bool,
    pub longpress_shortcut: Option<String>,
    pub toggle_shortcut: Option<String>,
    pub is_recording_via_hotkey: bool,
    // 新增：keytap 状态
    pub keytap_enabled: bool,
    pub keytap_keys: Vec<String>,
}
```

### Step 4: 添加统一注册接口 (1 小时)

```rust
#[tauri::command]
pub async fn register_unified_hotkey(
    app: AppHandle,
    shortcut: String,
    mode: String,
) -> Result<(), String> {
    // 判断是组合键还是单键
    if shortcut.contains("+") {
        // 使用现有的 register_hotkey
        register_hotkey(app, shortcut, mode).await
    } else if is_single_modifier_key(&shortcut) {
        // 使用 keytap
        register_keytap_key(app, shortcut, mode).await
    } else {
        register_hotkey(app, shortcut, mode).await
    }
}

fn is_single_modifier_key(key: &str) -> bool {
    matches!(
        key,
        "RightControl" | "LeftControl" | 
        "RightAlt" | "LeftAlt" | 
        "RightShift" | "LeftShift"
    )
}
```

### Step 5: 更新前端配置 (1 小时)

在 `AppSettingsDialog.tsx` 中添加快捷键类型选择：

```typescript
const [shortcutType, setShortcutType] = useState<"combination" | "single">("combination");

// 添加类型选择器
<Select value={shortcutType} onValueChange={setShortcutType}>
  <SelectItem value="combination">组合键（推荐）</SelectItem>
  <SelectItem value="single">单键（实验性）</SelectItem>
</Select>

// 根据类型显示不同的提示
<Input
  placeholder={
    shortcutType === "combination" 
      ? "例如：CommandOrControl+Shift+R" 
      : "例如：RightControl"
  }
/>
```

### Step 6: 注册到应用 (15 分钟)

在 `lib.rs` 中初始化 keytap：

```rust
.setup(|app| {
    // 现有的 HotkeyState
    app.manage(Arc::new(Mutex::new(hotkey::HotkeyState::default())));
    
    // 新增：KeytapListener
    app.manage(Arc::new(Mutex::new(keytap_listener::KeytapListener::new())));
    
    // ... 现有的快捷键注册逻辑
    Ok(())
})
.invoke_handler(tauri::generate_handler![
    // ... 现有命令
    hotkey::register_unified_hotkey,  // 新增
])
```

### 总计时间：约 4 小时（基础版）

---

## 💾 已移除的左右修饰键支持代码（完整备份）

> **说明**：以下代码在 2026-05-25 从项目中移除，因为当前决策是使用标准组合键格式。
> 如果未来需要实现 keytap 集成，可以从这里恢复这些代码。

### 1. ShortcutInput 组件（支持左右修饰键版本）

**文件位置**：`src/components/ui/shortcut-input.tsx`

**完整代码**：
```typescript
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

    // 更新活跃的修饰键状态（区分左右）
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

// 辅助函数：获取按键名称（区分左右修饰键）
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
```

**关键特性**：
- ✅ 区分左右修饰键（`LeftControl` vs `RightControl`）
- ✅ 支持单键快捷键（松开修饰键时记录）
- ✅ 支持连按检测（如 `Space*2`）
- ✅ 实时显示按键状态

### 2. 快捷键格式化工具（支持左右修饰键版本）

**文件位置**：`src/utils/shortcut.ts`

**完整代码**：
```typescript
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
    "Control": "Ctrl",
    "RightControl": "右Ctrl",      // 支持右 Ctrl
    "LeftControl": "左Ctrl",       // 支持左 Ctrl
    "Shift": "Shift",
    "RightShift": "右Shift",       // 支持右 Shift
    "LeftShift": "左Shift",        // 支持左 Shift
    "Alt": "Alt",
    "RightAlt": "右Alt",           // 支持右 Alt
    "LeftAlt": "左Alt",            // 支持左 Alt
    "Meta": "Win",
    "Space": "空格",
    "Enter": "回车",
    "Backspace": "退格",
    "Tab": "Tab",
    "Escape": "Esc",
    "Up": "↑",
    "Down": "↓",
    "Left": "←",
    "Right": "→",
    "Delete": "Delete",
    "Home": "Home",
    "End": "End",
    "PageUp": "PageUp",
    "PageDown": "PageDown",
  };

  return shortcut
    .split("+")
    .map(key => keyMap[key] || key)
    .join(" + ");
}
```

### 3. 旧的默认值配置

**文件位置**：`src/pages/SettingsPage.tsx`

**旧的默认值**：
```typescript
// 长按模式
defaultValue="RightControl"
// 提示文本：按住快捷键录音，松开停止。默认：右Ctrl

// 切换模式
defaultValue="LeftAlt+Space"
// 提示文本：按一次开始录音，再按一次停止。默认：左Alt+空格
```

### 4. 恢复指南

如果未来需要恢复左右修饰键支持（实现 keytap 集成时）：

1. **恢复 `shortcut-input.tsx`**：
   - 将上面的完整代码复制回 `src/components/ui/shortcut-input.tsx`
   - 替换简化版本

2. **恢复 `shortcut.ts`**：
   - 添加左右修饰键的映射到 `keyMap`

3. **更新默认值**：
   - 在 `SettingsPage.tsx` 中恢复旧的默认值
   - 或者提供两种模式供用户选择

4. **后端集成**：
   - 实现 `keytap_listener.rs` 模块
   - 添加统一的快捷键注册接口
   - 根据快捷键格式自动选择 global-hotkey 或 keytap

---

## 📋 实施检查清单

### 准备阶段
- [x] 快捷键系统已实现
- [x] 录音指示器已实现
- [x] 配置系统已实现
- [x] 前端界面已实现
- [x] 日志系统已实现

### 集成阶段
- [ ] 添加 keytap 依赖
- [ ] 创建 KeytapListener 模块
- [ ] 扩展 HotkeyState 结构
- [ ] 实现统一注册接口
- [ ] 更新前端配置界面
- [ ] 注册到应用初始化

### 测试阶段
- [ ] Windows 平台测试
- [ ] macOS 平台测试（含权限）
- [ ] Linux 平台测试（含权限）
- [ ] 组合键和单键混用测试
- [ ] 快捷键冲突测试

### 文档阶段
- [ ] 更新 README
- [ ] 添加用户指南
- [ ] 记录已知问题

---

## 🔧 关键集成点

### 1. 事件处理统一

**现有架构：**
```
global-hotkey 事件 → handle_hotkey_event() → handle_long_press_mode()
                                            → handle_toggle_mode()
```

**集成后架构：**
```
global-hotkey 事件 → handle_hotkey_event() → handle_long_press_mode()
                                            → handle_toggle_mode()

keytap 事件 → handle_keytap_event() → handle_key_press()
                                     → handle_key_release()
                                     
两者都调用相同的：
  - indicator::show_indicator()
  - indicator::hide_indicator()
  - start_recording()
  - stop_recording()
```

### 2. 状态管理统一

**现有状态：**
- `HotkeyState` 管理 global-hotkey 状态
- `RecordingState` 管理录音状态

**集成后状态：**
- `HotkeyState` 扩展，同时管理两种快捷键
- `KeytapListener` 独立管理 keytap 状态
- 通过 `is_recording_via_hotkey` 标志协调两者

### 3. 配置存储扩展

**现有配置：**
```rust
pub struct AppConfig {
    pub longpress_shortcut: String,  // "CommandOrControl+Shift+R"
    pub toggle_shortcut: String,     // "Alt+Space"
}
```

**扩展配置（可选）：**
```rust
pub struct AppConfig {
    pub longpress_shortcut: String,
    pub toggle_shortcut: String,
    // 新增：标记快捷键类型
    pub longpress_type: String,  // "combination" | "single"
    pub toggle_type: String,     // "combination" | "single"
}
```

---

## 📝 推荐决策

### 建议：采用方案 C（混合方案）

**理由：**
1. **用户需求不明确**：大多数用户可能不需要单键快捷键
2. **开发成本可控**：简化版只需 20 小时
3. **风险可控**：默认不启用，降低兼容性风险
4. **灵活性**：满足高级用户需求

**实施建议：**
1. 先完成当前的组合键方案（已完成）
2. 收集用户反馈，验证单键需求
3. 如果需求强烈，再实施简化版 keytap 集成
4. 作为"实验性功能"发布，逐步完善

---

## 📚 参考资源

- [keytap crate 文档](https://docs.rs/keytap)
- [Tauri 插件开发指南](https://tauri.app/v2/plugin/)
- [macOS 辅助功能权限](https://developer.apple.com/documentation/accessibility)
- [Linux input 子系统](https://www.kernel.org/doc/html/latest/input/input.html)

---

## 📦 现有代码参考

### 完整的 hotkey.rs 结构

```rust
// src-tauri/src/hotkey.rs
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct HotkeyState {
    pub longpress_registered: bool,
    pub toggle_registered: bool,
    pub longpress_shortcut: Option<String>,
    pub toggle_shortcut: Option<String>,
    pub is_recording_via_hotkey: bool,
}

#[derive(Clone, Debug)]
pub enum RecordingMode {
    LongPress,
    Toggle,
}

// 关键函数：
// - register_hotkey() - 单个快捷键注册
// - register_both_hotkeys() - 批量注册
// - unregister_hotkey() - 注销
// - handle_hotkey_event() - 事件分发
// - handle_long_press_mode() - 长按处理（已集成 indicator）
// - handle_toggle_mode() - 切换处理（已集成 indicator）
```

### 完整的 indicator.rs 结构

```rust
// src-tauri/src/indicator.rs
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

const INDICATOR_LABEL: &str = "recording-indicator";

pub fn show_indicator(app: &AppHandle) -> Result<(), String> {
    // 检查窗口是否已存在
    if let Some(window) = app.get_webview_window(INDICATOR_LABEL) {
        let _ = window.show();
        return Ok(());
    }

    // 创建新窗口（开发模式使用 file:// URL）
    let indicator_url = if cfg!(dev) {
        let path = std::env::current_dir()?.join("indicator.html");
        WebviewUrl::External(format!("file:///{}", path.display()).parse()?)
    } else {
        WebviewUrl::App("indicator.html".into())
    };

    let window = WebviewWindowBuilder::new(app, INDICATOR_LABEL, indicator_url)
        .title("录音中")
        .inner_size(200.0, 80.0)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .transparent(true)
        .visible(false)
        .build()?;

    window.center()?;
    window.show()?;
    Ok(())
}

pub fn hide_indicator(app: &AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window(INDICATOR_LABEL) {
        let _ = window.hide();
    }
    Ok(())
}
```

### 配置更新时的热重载

```rust
// src-tauri/src/data.rs
pub fn update_app_config(app: tauri::AppHandle, config: AppConfig) -> Result<AppConfig, String> {
    // 保存配置
    let store = app.store(APP_CONFIG_STORE)?;
    let value = serde_json::to_value(&config)?;
    store.set(APP_CONFIG_KEY.to_string(), value);
    store.save()?;

    // 热更新快捷键
    let app_clone = app.clone();
    let config_clone = config.clone();
    tauri::async_runtime::spawn(async move {
        let longpress = if config_clone.longpress_shortcut.is_empty() {
            None
        } else {
            Some(config_clone.longpress_shortcut)
        };
        let toggle = if config_clone.toggle_shortcut.is_empty() {
            None
        } else {
            Some(config_clone.toggle_shortcut)
        };
        let _ = crate::hotkey::register_both_hotkeys(app_clone, longpress, toggle).await;
    });

    Ok(config)
}
```

### 前端配置界面关键代码

```typescript
// src/components/dialogs/AppSettingsDialog.tsx
const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
  e.preventDefault();
  if (!appConfig) return;

  const nextConfig = {
    ...appConfig,
    longpress_shortcut: appConfig.longpress_shortcut.trim(),
    toggle_shortcut: appConfig.toggle_shortcut.trim(),
  };

  if (!nextConfig.longpress_shortcut) {
    toast.error("长按模式快捷键不能为空");
    return;
  }

  if (!nextConfig.toggle_shortcut) {
    toast.error("切换模式快捷键不能为空");
    return;
  }

  invoke<AppConfig>("update_app_config", { config: nextConfig })
    .then((savedConfig) => {
      onConfigSaved(savedConfig);
      toast.success("应用设置已保存");
      onOpenChange(false);
    })
    .catch((error) => {
      toast.error(`保存应用设置失败：${String(error)}`);
    });
};
```

---

## 🎓 经验教训

### 1. 快捷键格式问题

**问题：** 最初使用 `RightControl` 和 `LeftAlt+Space` 导致注册失败。

**原因：** Tauri 的 global-hotkey 插件使用 muda 库，不支持区分左右修饰键。

**解决：** 改用标准格式 `CommandOrControl+Shift+R` 和 `Alt+Space`。

**教训：** 在集成 keytap 时，需要明确区分两种快捷键格式，并在 UI 中给用户清晰的提示。

### 2. 指示器窗口路径问题

**问题：** 开发模式下指示器窗口显示 "ERR_FILE_NOT_FOUND"。

**原因：** 工作目录是 `src-tauri`，但 `indicator.html` 在项目根目录。

**解决：** 
1. 复制文件到 `src-tauri` 目录
2. 使用 `std::env::current_dir()` 动态构建路径

**教训：** 开发模式和生产模式的资源路径处理需要分开处理。

### 3. 透明窗口兼容性

**注意：** Windows 上透明窗口可能需要特殊处理，某些系统配置下可能不生效。

**建议：** 提供降级方案（非透明窗口）或在文档中说明系统要求。

---

## 🔗 相关文件清单

### Rust 后端
- `src-tauri/src/lib.rs` - 应用入口和初始化
- `src-tauri/src/hotkey.rs` - 快捷键系统（✅ 已完成）
- `src-tauri/src/indicator.rs` - 录音指示器（✅ 已完成）
- `src-tauri/src/recording.rs` - 录音功能
- `src-tauri/src/data.rs` - 配置管理
- `src-tauri/src/keytap_listener.rs` - 待创建（keytap 集成）

### 前端
- `src/components/dialogs/AppSettingsDialog.tsx` - 设置对话框（✅ 已完成）
- `src/types/config.ts` - 配置类型定义
- `src/main.tsx` - 应用初始化

### 资源文件
- `indicator.html` - 指示器窗口 HTML（✅ 已完成）
- `src-tauri/indicator.html` - 开发模式副本（✅ 已完成）

### 配置文件
- `src-tauri/Cargo.toml` - Rust 依赖
- `src-tauri/tauri.conf.json` - Tauri 配置
- `C:\Users\86136\AppData\Roaming\com.xiluolin.app\settings.json` - 用户配置

---

## ✅ 下一步行动

1. **立即**：与产品/用户确认单键快捷键的需求强度
2. **本周**：如果需求明确，启动 Phase 1 开发
3. **下周**：完成基础集成并进行内部测试
4. **两周后**：发布实验性功能供用户测试

---

**文档版本**：v1.0  
**创建日期**：2026-05-25  
**作者**：开发团队  
**状态**：待评审
