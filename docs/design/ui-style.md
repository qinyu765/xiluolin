# UI 视觉与交互规范

## 1. 设计定位

XiLuoLin 是桌面语音输入效率工具，界面应当服务于办公、写作和编程输入，而不是像营销网站一样争夺注意力。

当前整体方向：

- 主窗口采用 Notion 式 warm minimalism；
- 内容结构清晰、克制、可扫描；
- Notion Blue 用于主操作和键盘焦点；
- 录音悬浮状态条采用接近 Dynamic Island 的纯黑胶囊；
- 动画只表达状态或确认操作，不做无意义装饰。

## 2. 基础视觉

### 主窗口

- 暖白背景；
- 白色工具卡片；
- 黑色主文本；
- 柔和边框和低强调度辅助背景；
- 品牌标题可以使用 serif fallback；
- 正文、表单和数据使用系统无衬线字体；
- 不使用大面积装饰图形、复杂渐变或营销页式 hero。

### 录音状态条

- 使用近纯黑半透明表面；
- 胶囊外围完全透明；
- 只保留低对比度边框、轻阴影和极弱内高光；
- 不使用多段彩色渐变；
- 不增加没有语义的装饰线；
- 状态颜色只用于圆点、音频条和成功/失败边框；
- 必须在浅色、深色和复杂桌面背景上保持可读。

## 3. 交互状态标准

所有可交互组件都必须同时考虑鼠标、按下、键盘和禁用状态。

### Hover

- 明确显示 `cursor-pointer`；
- 通过背景、边框、文字或阴影中的一到两项表达；
- 按钮允许最多上移 `1px`，避免大幅移动导致布局抖动；
- 不用高饱和发光或过度放大；
- 输入框 hover 只增强边框和轻微背景，不应表现得像主按钮。

### Active

- 点击时恢复 hover 位移；
- 可以缩放到约 `0.98`；
- 反馈时间保持在约 `100–180ms`；
- 不应改变布局占位。

### Focus Visible

- 键盘焦点必须可见；
- 使用统一的 ring 和边框色；
- 不因为增加 hover 效果而删除 outline；
- 鼠标点击不必永久显示强焦点环，但键盘导航必须显示。

### Disabled

- 禁用组件不显示 pointer cursor；
- 不执行 hover 位移或 active 缩放；
- 降低对比度，但文字仍需可读；
- 禁止用“看起来可点击但点击无效”的方式表示禁用。

## 4. 组件规范

### Button

- 主按钮使用主色背景；
- Outline 按钮 hover 时增强边框并使用轻辅助背景；
- Ghost 按钮 hover 时显示低强调度背景；
- Destructive 操作保持红色语义，但不要通过持续动画制造焦虑；
- 图标按钮必须有 `title` 或可访问名称。

### Tabs 与导航

- 未选中项 hover 时显示轻背景和前景色变化；
- 选中项使用稳定背景，不依赖 hover 才可辨认；
- active 反馈要克制，不能让导航项跳动；
- 左侧导航和页面内 Tabs 应共享相同交互语言。

### Input 与 Textarea

- 默认边框低强调；
- hover 增强边框；
- focus-visible 使用主色 ring；
- placeholder 和 disabled 文本保持足够对比度；
- 不在输入区域使用会影响文字可读性的复杂背景。

### Select

- Trigger 使用 pointer cursor；
- 展开项在 hover/highlight 时必须有清晰背景；
- 当前选项需要独立于 hover 的选中标记；
- disabled 项不能显示可点击反馈。

### Switch

- 整个轨道区域可点击；
- hover 使用轻 ring；
- active 可以轻微缩放；
- checked 和 unchecked 必须通过轨道颜色及 thumb 位置同时区分。

### Dialog

- Overlay 只用于建立层级，不应过暗；
- 关闭按钮需要 pointer、hover、active 和 focus-visible；
- 危险操作应使用明确按钮文案，不只使用图标。

### Icon Picker

- 每个图标项都必须可聚焦；
- hover 显示边框、背景或阴影；
- selected 状态不能只依赖 hover；
- 点击时提供缩放确认。

## 5. 动效规范

- 通用交互过渡使用约 `150ms`；
- 状态变化可以使用 `180–300ms`；
- 录音和处理中允许循环动画，因为它表达持续状态；
- 成功和失败不使用无限循环动画；
- 支持 `prefers-reduced-motion`；
- 禁止大面积漂浮、呼吸或无意义渐变动画。

## 6. 悬浮窗口约束

录音状态条当前为鼠标穿透窗口，因此：

- 不能提供 hover 反馈；
- 不能直接添加可点击按钮；
- 不得获取焦点；
- 不得拦截目标应用点击；
- 显示和隐藏不能改变录音开始时捕获的目标窗口；
- 如果未来需要可交互版本，必须先设计区域 hit test 和焦点恢复。

透明窗口平台边界和完整决策见 [`../retrospectives/recording-overlay-and-ui-interaction.md`](../retrospectives/recording-overlay-and-ui-interaction.md)。

## 7. 不采用的内容

- 营销页式 hero；
- 商品展示、品牌官网或目录式布局；
- 大面积装饰图形或复杂渐变；
- 没有语义的发光线条；
- 只支持鼠标、不支持键盘的交互；
- 在业务页面重复实现通用 hover class；
- 为追求材质效果牺牲跨平台可读性。

## 8. 验证清单

每次修改 UI primitives 或悬浮窗口时至少检查：

- 浅色与深色背景；
- hover、active、focus-visible 和 disabled；
- 鼠标与键盘操作；
- 减少动态效果；
- 文本溢出和窗口缩放；
- Windows 与 macOS 差异；
- `pnpm build`；
- 关键页面浏览器或桌面截图；
- 测试服务器和 Playwright 进程清理。

## 9. 参考来源

- Notion 风格基础：`https://getdesign.md/notion/design-md`
- HP 风格曾用于评估企业化布局，但不作为当前主风格：`https://getdesign.md/hp/design-md`
- Voicebox 用于参考录音状态模型和悬浮窗口生命周期，不作为 XiLuoLin 的完整产品或视觉模板。
