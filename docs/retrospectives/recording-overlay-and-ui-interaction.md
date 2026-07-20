# 录音悬浮窗与桌面交互设计复盘

## 1. 文档目的

本文提炼 XiLuoLin 录音悬浮状态条和通用 UI 交互反馈改造中的可复用经验。它重点回答：

- 为什么一个看似简单的胶囊会涉及原生窗口、WebView、焦点和多显示器；
- 为什么第一版“玻璃效果”失败；
- 为什么最终选择纯黑胶囊；
- 为什么 hover 反馈应该在 UI primitives 层统一实现；
- Windows 与 macOS 透明窗口有哪些边界；
- 以后修改悬浮窗和桌面 UI 时应遵循什么检查顺序。

具体时间线、代码和命令见 [`../dev/2026-07-20-t037-recording-overlay-and-ui-feedback.md`](../dev/2026-07-20-t037-recording-overlay-and-ui-feedback.md)。

## 2. 录音状态条不是普通网页组件

普通网页组件只需要考虑自身 DOM 区域。桌面录音状态条同时存在四层：

```text
操作系统窗口
  └─ Tauri WebviewWindow
      └─ WebView 页面背景
          └─ 胶囊组件
```

用户看到的矩形背景可能来自任意一层：

- 原生窗口不透明；
- WebView 默认背景不透明；
- `html` 或 `body` 有背景；
- 胶囊自身尺寸或阴影溢出；
- 隐藏窗口仍作为透明点击区域存在。

因此排查顺序应该是：

1. 原生窗口是否启用 transparent；
2. WebView 页面是否透明；
3. 页面是否有默认 margin；
4. 胶囊是否只占预期区域；
5. 窗口是否关闭系统 decorations 和 shadow；
6. 隐藏后是否仍拦截鼠标；
7. 是否在显示时错误地获取焦点。

只修改 CSS 背景颜色，无法解决原生窗口层的问题。

## 3. 状态反馈的真正目标

录音状态条只有三个核心任务：

1. 告诉用户系统是否正在录音；
2. 告诉用户录音结束后是否仍在处理；
3. 告诉用户最终成功还是失败。

因此信息优先级应为：

```text
状态名称 > 录音时长 > 状态运动 > 装饰材质
```

音频条在当前实现中是过程反馈，不是真实声压计。这样做有几个好处：

- 不需要从 Rust 录音线程高频发送电平事件；
- 不增加跨 WebView 通信压力；
- 不影响录音实时性；
- 不会让用户误以为动画能够精确反映输入音量。

如果未来实现真实电平，应明确标注其采样窗口、平滑算法和静音阈值，不能只把随机动画替换成未经处理的瞬时幅值。

## 4. 玻璃设计为什么容易失败

“Apple 风格玻璃”常被错误简化为：

```text
高 blur + 高 saturation + 多段渐变 + 白色描边 + 多层阴影
```

这些手段单独使用没有问题，但同时堆叠在高度只有 42px 的胶囊上，会产生：

- 边缘过亮；
- 材质层次互相打架；
- 状态色不再突出；
- 不同背景下可读性不稳定；
- Windows 与 macOS 渲染差异放大；
- 视觉像营销页装饰，而不是系统状态反馈。

第一版左侧发光竖线也暴露了另一个问题：装饰没有编码任何新信息。录音圆点和音频条已经表达状态，再增加竖线只会形成重复视觉。

最终选择近纯黑胶囊，是因为它具备以下优点：

- 在浅色和深色应用上都有稳定对比度；
- 不依赖底层 WebView 是否能真正模糊桌面内容；
- 状态色只需占很小面积即可被感知；
- 与录音、系统状态、Dynamic Island 的语义更接近；
- Windows 和 macOS 更容易保持一致。

这不是拒绝玻璃效果，而是把 blur 降为增强项，把纯黑表面作为可靠降级基础。

## 5. 透明窗口的平台边界

### macOS

在当前 Tauri WebviewWindow 方案中，整个窗口透明需要启用：

```json
"app": {
  "macOSPrivateApi": true
}
```

这使当前应用不以 Mac App Store 分发为目标，但不影响 DMG、GitHub Release 或官网下载。

需要准确区分：

- Tauri 内置透明 WebviewWindow 路径需要该配置；
- macOS 并非理论上只有私有 API 才能做透明浮层；
- 也可以自建 `NSPanel`、透明 `NSWindow` 或 `NSVisualEffectView`，但需要平台专用原生实现。

### Windows

Windows 可以直接使用 Tauri 透明窗口，不需要对应的私有 API 开关。

但 CSS `backdrop-filter` 不保证等同于系统原生 Acrylic 或 Mica。要实现真正的 Windows 系统材质，需要额外评估原生窗口效果接口。当前纯黑半透明胶囊不依赖这些能力，因此能够稳定降级。

## 6. 焦点与交互是相互制约的

XiLuoLin 的录音状态条必须满足：

- 不抢走用户正在输入的应用焦点；
- 不拦截目标应用顶部区域的点击；
- 不出现在任务栏；
- 可以跨桌面显示。

因此采用：

```text
focusable(false)
set_ignore_cursor_events(true)
skip_taskbar(true)
always_on_top(true)
visible_on_all_workspaces(true)
```

这也意味着胶囊不应该同时承担可点击按钮、hover 菜单或错误详情展开。若以后要增加“停止录音”按钮，就必须重新解决：

- 点击时是否获取焦点；
- 如何保存和恢复目标窗口；
- 非按钮区域是否继续穿透；
- Windows/macOS 是否支持区域级 hit test；
- 错误状态消失和点击操作是否竞态。

不能在当前完全穿透窗口上简单添加一个 `<button>`，然后期待它可交互。

## 7. 多显示器定位的通用原则

显示器宽高不包含显示器在虚拟桌面中的位置。正确公式必须加入 origin：

```text
x = monitor.origin.x + (monitor.width - window.width) / 2
y = monitor.origin.y + monitor.height × topRatio
```

尤其需要测试：

- 副屏在主屏左侧，`x < 0`；
- 副屏在主屏上方，`y < 0`；
- 不同缩放比例；
- 窗口隐藏后 `current_monitor()` 返回空；
- 当前显示器不可用时回退主显示器。

位置计算适合抽成纯函数测试，不应全部埋在窗口 API 调用中。

## 8. 为什么交互反馈必须从 primitives 开始

逐页补 hover class 会造成：

- 同类按钮反馈不同；
- 新页面继续遗漏；
- dark mode 和 disabled 状态不一致；
- 设计规范只能靠人工记忆。

更可维护的方式是在 primitives 中定义统一状态：

| 状态 | 反馈 |
|---|---|
| 默认 | 清晰边界和文字层级 |
| Hover | 颜色、边框或阴影变化，必要时最多上移 1px |
| Active | 恢复位移并轻微缩放 |
| Focus Visible | 明确的键盘焦点环 |
| Disabled | 降低对比度，取消指针和 transform |

本轮将该规则应用到 Button、Tabs、Input、Textarea、Select、Switch、Dialog Close 和 IconPicker。页面因此自动继承反馈，而不需要业务组件知道具体动画参数。

## 9. 不要把 hover 当成唯一交互反馈

桌面应用需要同时覆盖：

- 鼠标：hover 和 pointer cursor；
- 触控板点击：active；
- 键盘：focus-visible；
- 减少动态效果：避免强制循环动画；
- 禁用状态：不能保留假反馈；
- 高对比度和深色模式：颜色不能只靠透明度区分。

仅添加 `hover:bg-*` 不能算完成交互设计。没有 focus-visible 的控件对键盘用户不可见；没有 active 的控件点击时缺乏确认感；disabled 仍上浮则会制造错误暗示。

## 10. 开发工具生命周期也是 UI 验证的一部分

视觉测试常需要启动：

- Vite dev server；
- Vite preview；
- Tauri dev；
- Playwright 浏览器；
- Python 临时 HTTP server。

如果不清理，下一次 `pnpm tauri dev` 会因为 1420 被占用而失败。XiLuoLin 的 `devUrl` 固定为 1420，不能依赖 Vite 自动换端口。

推荐收尾检查：

```bash
lsof -nP -iTCP:1420 -sTCP:LISTEN
lsof -nP -iTCP:4173 -sTCP:LISTEN
ps -ax -o pid=,ppid=,etime=,command= | grep -E 'tauri|vite|scripts/dev.mjs|playwright'
```

只终止父进程不一定会清理全部子进程。发现暂停状态或孤儿进程时，应先确认 PID 与仓库路径，再精确终止，避免误伤其他项目。

## 11. 可复用检查清单

### 悬浮窗口

- [ ] 原生窗口透明；
- [ ] HTML、body 和根组件透明；
- [ ] decorations、shadow 和 taskbar 行为符合预期；
- [ ] 不主动获取焦点；
- [ ] 鼠标穿透或点击策略明确；
- [ ] 每次显示重新定位；
- [ ] 支持负坐标副屏；
- [ ] 完成、失败和新会话之间没有延迟隐藏竞态；
- [ ] 减少动态效果可用；
- [ ] 浅色和深色背景均可读。

### 通用交互组件

- [ ] pointer cursor；
- [ ] hover 可辨认但不过度；
- [ ] active 有确认反馈；
- [ ] focus-visible 清晰；
- [ ] disabled 不产生误导反馈；
- [ ] dark mode 可读；
- [ ] 动画不会导致布局抖动；
- [ ] 相同组件在不同页面表现一致。

### 验证收尾

- [ ] `pnpm build`；
- [ ] Rust fmt/check/test；
- [ ] 主页面关键控件 hover 检查；
- [ ] 悬浮页面六种状态检查；
- [ ] 计时增长、冻结、重置；
- [ ] 多显示器实机验证；
- [ ] 清理测试服务器和浏览器进程；
- [ ] 检查 1420/4173 等端口释放；
- [ ] 删除临时截图和 `.playwright-cli/`。

## 12. 后续演进方向

### 可交互悬浮窗

只有明确需要停止、取消或复制错误时再实施。优先研究区域 hit test 和焦点恢复，不应直接取消整个窗口的鼠标穿透。

### 真实音量反馈

可以从录音线程计算 RMS/peak，经过节流和平滑后发送低频事件。但必须先确认它对用户判断麦克风工作状态是否真正有价值。

### 原生材质

- macOS：评估 `NSPanel` + `NSVisualEffectView`；
- Windows：评估 Acrylic/Mica 或 DWM backdrop；
- 仅在原生材质明显提升体验且维护成本可接受时实施。

### 视觉回归测试

建立独立 UI showcase 或组件预览页，固定窗口尺寸和状态，保存关键截图，避免每次都依赖完整 Tauri 流程才能审查 primitive 和状态胶囊。

## 13. 结论

本轮最重要的结论不是“把胶囊改成黑色”，而是建立了三个边界：

1. 原生窗口透明和 Web 页面透明是两件事；
2. 悬浮状态反馈应优先稳定、可读和不打扰，而不是追求复杂材质；
3. 桌面交互反馈必须在组件基础层统一，同时覆盖鼠标、按下、键盘和禁用状态。

这些边界可以减少后续 UI 改造中的重复试错，也能防止视觉优化破坏语音输入链路最关键的焦点和可靠性。
