# T037 录音悬浮状态条与桌面交互反馈改进

> **实施记录：** 本文记录 2026-07-17 至 2026-07-20 对录音悬浮状态条、透明窗口、跨平台视觉和通用控件交互反馈的完整实施过程。当前行为以代码、[`../usage-guide.md`](../usage-guide.md) 和 [`../design/ui-style.md`](../design/ui-style.md) 为准。

## 开源协作说明

关联 Issue：无，需求由本地产品体验评审和用户连续反馈直接触发。

实施分支：`fix/macos-voice-pipeline-reliability`。

参考项目：Voicebox 的 Capture Pill、Dictate Window 和快捷键状态设计。只借鉴交互模式和窗口生命周期，XiLuoLin 按自身技术栈重新实现，没有直接复制其组件代码。

## 任务背景

XiLuoLin 原有录音指示器已经可以显示以下状态：

```text
recording → transcribing → refining → delivering → completed / failed
```

但原实现只是一个 `220 × 54` 的无边框 WebviewWindow，页面中只有圆点和文字：

- 录音阶段没有时长；
- 处理阶段缺少可感知的连续反馈；
- 窗口位于屏幕约 70% 高度，容易遮挡输入区域；
- 多显示器定位没有考虑显示器坐标原点；
- macOS 外层原生窗口不透明，胶囊外围会出现矩形背景；
- 第一版玻璃效果使用多层渐变和左侧装饰线，视觉显得廉价；
- 应用内按钮、标签页、下拉框、开关和图标选择器缺少统一的 hover、active 和 cursor 反馈。

这不是单纯“换一组颜色”的问题。录音悬浮窗同时涉及：

1. Tauri 原生窗口是否透明；
2. WebView 页面自身是否透明；
3. 窗口是否抢焦点或拦截鼠标；
4. 状态机如何驱动计时和动画；
5. Windows 与 macOS 对透明窗口的支持差异；
6. 应用主界面是否具有统一的交互语言。

## 任务目标

- 让录音状态条在当前显示器顶部居中显示；
- 保持 always-on-top、不可聚焦和鼠标穿透；
- 增加录音时长，并在录音停止后冻结；
- 用装饰性音频条区分录音、处理、成功和失败；
- 让胶囊外围真正透明；
- 将胶囊视觉收敛为纯黑、克制、接近 Dynamic Island 的设计；
- 为通用可交互组件补齐 hover、active、focus-visible 和 pointer cursor；
- 不增加 Framer Motion 等新依赖；
- 不引入真实麦克风电平上报，避免扩大录音链路改动范围。

## 方案调研与范围取舍

### Voicebox 中值得借鉴的部分

Voicebox 的录音 pill 提供了几个重要启发：

- 独立透明窗口承载状态，不依赖主窗口是否可见；
- 录音、转写、整理和完成由统一状态驱动；
- 录音时显示计时，离开录音状态后冻结最终时长；
- 音频条只是状态动画，并不伪装成真实音量；
- 窗口显示时不获取焦点；
- 每次显示时重新计算位置；
- 隐藏窗口时需要避免留下不可见的鼠标拦截区域。

### 没有照搬的部分

本轮明确不做：

- 不把静态页面重写成 React 独立入口；
- 不增加 Framer Motion；
- 不在胶囊中提供停止或取消按钮；
- 不让胶囊响应鼠标，因为它必须保持穿透，不能破坏目标应用输入；
- 不实现真实音量波形；
- 不引入 TTS、Agent Speak 或 Voicebox 的后端服务架构。

继续使用 `public/indicator.html` 的原因是：当前指示器需求单一、状态接口稳定、没有复杂交互。使用原生 HTML/CSS/JavaScript 可以降低包体和运行时耦合，也避免悬浮窗口加载完整 React 应用。

## 实施过程

### 阶段一：从静态圆点升级为状态胶囊

在 `public/indicator.html` 中保留现有入口：

```javascript
window.setIndicatorStatus(status)
```

Rust 仍通过 `WebviewWindow::eval` 更新状态，因此不需要改动 pipeline 的调用接口。

新增内容：

- 状态圆点；
- 中文状态标签；
- 5 根错峰动画音频条；
- `m:ss` 录音时长；
- `aria-live="polite"`；
- `prefers-reduced-motion` 降级。

计时规则：

```text
进入 recording
  → elapsedMs 归零
  → recordingStartedAt = Date.now()
  → 每 200ms 更新显示

离开 recording
  → 计算并保存最后时长
  → 停止 interval
  → transcribing/refining/delivering/completed/failed 均显示冻结值

再次进入 recording
  → 清理旧 timer
  → 从 0:00 重新开始
```

浏览器验证得到：

```json
{
  "during": "0:01",
  "frozen": "0:01",
  "after": "0:01",
  "reset": "0:00"
}
```

说明录音阶段增长、处理阶段冻结和下一次录音重置均符合预期。

### 阶段二：顶部定位与多显示器坐标

原实现直接使用显示器宽高计算：

```text
x = (monitor.width - 220) / 2
y = monitor.height × 0.7
```

这在副屏位于主屏左侧或上方时不正确，因为没有加入 `monitor.position()`。

新实现抽取纯函数：

```text
x = monitor_x + (monitor_width - window_width) / 2
y = monitor_y + monitor_height × 0.04
```

并在每次 `show_indicator` 时重新定位，避免窗口一直停留在旧显示器位置。

增加测试覆盖：

- `1920 × 1080` 主显示器；
- 起点为 `(-2560, -120)` 的副显示器；
- 负坐标和顶部偏移计算。

### 阶段三：胶囊外围矩形背景问题

#### 现象

HTML 已经设置：

```css
html,
body {
  background: transparent;
}
```

但 macOS 上仍能看到窗口的矩形底色。

#### 根因

CSS 透明只影响 WebView 页面内容，无法把一个不透明的原生 `NSWindow` 变成透明窗口。XiLuoLin 当时只在非 macOS 平台调用：

```rust
#[cfg(not(target_os = "macos"))]
window_builder.transparent(true)
```

所以 Windows 窗口透明，macOS 外层窗口仍不透明。

#### 方案比较

1. **只支持 Windows**
   - 最简单；
   - 无法解决 macOS 实际体验。

2. **使用 Tauri 内置透明窗口**
   - 启用 `app.macOSPrivateApi`；
   - 改动最小，保留现有 WebviewWindow 架构；
   - 不能以 Mac App Store 为分发目标。

3. **自建原生 `NSPanel` / `NSVisualEffectView`**
   - 可获得更原生的材质和窗口控制；
   - 需要 Swift/Objective-C 或 Rust 原生插件；
   - 跨平台维护成本明显增加。

最终接受方案 2：

```json
{
  "app": {
    "macOSPrivateApi": true
  }
}
```

并让窗口构建器在 Windows 与 macOS 都调用：

```rust
.transparent(true)
```

需要注意：`macOSPrivateApi` 是 Tauri 内置透明 WebviewWindow 路径的要求，不代表 macOS 所有透明窗口技术都必须使用私有 API。

### 阶段四：玻璃效果失败与视觉收敛

第一版尝试 Apple 毛玻璃风格，使用：

- 多段灰蓝渐变；
- 高亮边框；
- 多层内外阴影；
- 左侧发光竖线；
- 高强度 blur 和 saturation。

虽然技术上实现了“玻璃”，但实际结果存在问题：

- 左侧竖线没有表达真实状态，只是装饰噪音；
- 多段渐变在小尺寸胶囊中显得廉价；
- 高亮边框、阴影、渐变和状态色同时竞争视觉注意力；
- 玻璃材质在 Windows WebView2 与 macOS WKWebView 上不一定一致；
- 录音状态条的首要任务是清晰、稳定，不是展示复杂材质。

最终改为接近 Dynamic Island 的纯黑设计：

```css
background: rgba(7, 7, 8, 0.94);
border: 1px solid rgba(255, 255, 255, 0.11);
box-shadow:
  0 10px 26px rgba(0, 0, 0, 0.3),
  inset 0 1px 0 rgba(255, 255, 255, 0.08);
```

保留轻微背景模糊作为增强，但视觉不依赖 blur 才能成立。状态颜色只出现在：

- 录音圆点；
- 音频条；
- 成功或失败时的克制边框。

删除左侧竖线和全部复杂渐变。

### 阶段五：通用交互反馈审查

检查 UI primitives 后发现，组件虽然多数已有 focus-visible 样式，但鼠标反馈不统一：

- Button 没有显式 `cursor-pointer`；
- TabsTrigger 只有 active 状态，hover 较弱；
- Input 和 Textarea 没有 hover 边框反馈；
- SelectItem 使用 `cursor-default`；
- Switch 缺少 hover 和 pressed 状态；
- Dialog Close 只有 opacity 变化；
- IconPicker 缺少 focus-visible 和按压反馈。

本轮没有逐页添加临时 class，而是在通用组件层建立统一交互语言：

```text
可点击：cursor-pointer
悬浮：颜色 / 边框 / 阴影变化，按钮最多上移 1px
按下：恢复位移并缩放到约 98%
键盘：保留 focus-visible ring
禁用：pointer-events-none、降低透明度、取消 transform
```

涉及组件：

- `Button`
- `TabsTrigger`
- `Input`
- `Textarea`
- `SelectTrigger` / `SelectItem`
- `Switch`
- `DialogClose`
- `IconPicker`

将反馈放在 primitive 层的好处是页面自动保持一致，后续新增页面也不需要重复补 hover class。

### 阶段六：开发服务器端口冲突

#### 现象

再次运行：

```bash
pnpm tauri dev
```

Vite 报错：

```text
Error: Port 1420 is already in use
```

#### 定位

通过：

```bash
lsof -nP -iTCP:1420 -sTCP:LISTEN
ps -ax -o pid=,ppid=,etime=,command=
```

确认旧的 `pnpm tauri dev`、`tauri dev`、`pnpm dev`、`scripts/dev.mjs` 和 esbuild 子进程仍然存在。

这些进程处于 `T` 状态，普通 `SIGTERM` 没有让它们退出，因此最终按精确 PID 清理整个进程链，并确认 1420 不再监听。

#### 经验

自动化视觉验证或桌面测试结束后必须：

1. 关闭 Tauri/Vite/Preview 父进程；
2. 检查其 Node、Cargo、esbuild 子进程；
3. 使用 `lsof` 确认 1420、4173 等测试端口释放；
4. 删除 `.playwright-cli/` 等临时目录；
5. 不通过修改 Vite 端口规避问题，因为 `tauri.conf.json` 的 `devUrl` 固定为 1420。

## 实际改动

### 悬浮窗口

- 将窗口尺寸调整为 `320 × 64`；
- 每次显示时定位到当前显示器顶部约 4%；
- 增加副屏负坐标测试；
- 保持 `always_on_top`、`skip_taskbar`、`focusable(false)`；
- 开启 `visible_on_all_workspaces`；
- 关闭系统窗口阴影，由胶囊自身控制阴影；
- Windows 与 macOS 都开启透明窗口；
- 启用 macOS Tauri 私有透明 API。

### 状态页面

- 新增录音计时和冻结逻辑；
- 新增录音、处理两组动画；
- 新增成功和失败视觉；
- 支持减少动态效果；
- 将视觉改为纯黑胶囊；
- 保持页面和胶囊外围透明；
- 保持鼠标穿透。

### 应用 UI

- 为通用交互组件补充 pointer、hover、active 和 focus-visible；
- 没有在业务页面复制样式；
- 没有改变组件 props 或公开类型。

### 文档

- 在使用指南中记录状态条行为和平台限制；
- 在 UI 视觉规范中固化交互反馈标准；
- 新增本实施记录和对应工程复盘。

## 涉及文件

核心实现：

- `public/indicator.html`
- `src-tauri/src/indicator.rs`
- `src-tauri/tauri.conf.json`
- `src/components/ui/*.tsx`

当前文档：

- `docs/usage-guide.md`
- `docs/design/ui-style.md`

## 验证过程

运行：

```bash
pnpm build
cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
git diff --check
```

结果：

- TypeScript 类型检查通过；
- Vite 构建通过；
- Rust 格式检查和编译通过；
- Rust 单元与集成测试通过；
- 外部 Whisper 模型 smoke test 按预期忽略；
- 浏览器确认胶囊尺寸为 `296 × 42`，位于 `320 × 64` 页面中央；
- Playwright 确认导航和按钮具有 pointer cursor；
- 计时增长、冻结和重置符合预期；
- 文档和代码 diff 无空白错误。

## 经验总结

1. Web 页面透明不等于原生窗口透明，必须分别检查 WebView 和 Window 两层。
2. “Apple 风格”不等于堆叠渐变、阴影和 blur；小型状态组件更适合克制的单一材质。
3. 状态颜色应该表达语义，而不是成为装饰。
4. 鼠标反馈应在组件 primitive 层实现，而不是在每个页面局部打补丁。
5. Hover、active 和 focus-visible 是三套输入反馈，不能只做其中一种。
6. 悬浮状态窗为了不破坏输入焦点，选择鼠标穿透后，就不应再要求它提供 hover 或点击交互。
7. CSS `backdrop-filter` 在不同系统 WebView 中可能表现不同，设计必须在不依赖真实背景模糊时仍可成立。
8. Tauri 开发服务包含多层父子进程，测试结束只关浏览器并不能保证端口释放。
9. 参考开源项目时应吸收状态模型和生命周期，而不是照搬产品范围或复杂依赖。

## 未完成事项

- 尚未使用 Windows 实机验证所有显示缩放比例和多显示器组合；
- 尚未使用打包后的 macOS `.app` 验证不同桌面、全屏应用和多 Space；
- 音频条不是实时麦克风电平；
- Windows 没有接入原生 Acrylic/Mica；
- macOS 没有改用原生 `NSVisualEffectView`；
- 悬浮窗没有停止、取消或错误复制交互，这是保持鼠标穿透的主动取舍；
- 主界面仍需持续做逐页视觉审查，本轮只建立通用交互基础。

## 后续建议

1. 在 Windows 100%、125%、150% 缩放和多显示器环境验证定位。
2. 在 macOS 打包签名版本中验证透明窗口和 Keychain 行为。
3. 为 UI primitives 增加 Storybook 或独立视觉测试页。
4. 建立键盘、鼠标和减少动态效果的可访问性检查清单。
5. 如果未来需要可交互悬浮窗，先重新设计焦点与点击穿透策略，而不是直接添加按钮。
6. 只有真实用户价值明确时再评估原生 Acrylic、Mica 或 `NSVisualEffectView`。
