---
name: frontend-design
description: 在前端开发中应用品牌级设计系统。引导用户选择预设风格（Claude、Vercel、Stripe 等 69+ 品牌）、designprompts.dev 上的 AI 设计 Prompt，或自定义 DESIGN.md，然后将其自动应用到项目 CSS 中。当用户提到想要某种风格、主题或视觉样式时触发。
---

# Frontend Design System Skill

在前端项目中应用品牌级视觉设计系统。基于 [awesome-design-md](https://github.com/VoltAgent/awesome-design-md) 提供的 DESIGN.md 规范文件，将设计 token（颜色、字体、间距、阴影等）自动映射到项目的 CSS 变量和组件样式中。

---

## 触发条件

当用户在前端开发过程中提到以下意图时触发：

- "做成 Claude / Vercel / Stripe 的风格"
- "想要一个暗色 / 暖色 / 极简的设计"
- "换个主题 / 样式 / 皮肤"
- "用某某网站的设计风格"
- "应用 DESIGN.md"
- "用 designprompts.dev 上的 XX 风格"（如 Bauhaus、Cyberpunk、Vaporwave 等）

---

## 第一步：引导用户明确需求（必须先完成）

**在写任何代码之前，必须先向用户提问以明确需求。** 按以下顺序收集信息：

### 1.1 确定风格来源

向用户展示以下选项，并等待回答：

> **你想让项目使用什么视觉风格？可以选择以下方式：**
>
> **A. 选择预设品牌风格**（69+ 可选，部分热门推荐见下方）
> **B. 描述你想要的氛围**（如"暗色科技感"、"暖色文艺风"、"极简黑白"）
> **C. 提供自定义 DESIGN.md 文件路径**
> **D. 从 [designprompts.dev](https://www.designprompts.dev/) 选一个设计风格**（30 种 AI 优化 prompt，含 Bauhaus、Cyberpunk、Vaporwave 等）
>
> **热门推荐：**
>
> | 风格 | 关键词 | `npx` 名称 |
> |------|--------|-----------|
> | Claude | 暖色赤陶、衬线标题、羊皮纸 | `claude` |
> | Vercel | 极简黑白、Geist 字体 | `vercel` |
> | Stripe | 紫色渐变、优雅轻盈 | `stripe` |
> | Linear | 超简洁、紫色强调 | `linear.app` |
> | Supabase | 暗色翡翠、代码优先 | `supabase` |
> | Notion | 暖色极简、衬线标题 | `notion` |
> | Spotify | 活力绿+暗色、大字体 | `spotify` |
> | Apple | 高端白色空间、SF Pro | `apple` |
> | Nike | 单色、巨大 Futura 大写 | `nike` |
> | Figma | 活力多彩、专业趣味 | `figma` |
>
> **designprompts.dev 部分风格：**
>
> | 风格 | 明暗 | 关键词 |
> |------|------|--------|
> | Bauhaus | 亮 | 几何、功能主义、包豪斯 |
> | Neo Brutalism | 亮 | 粗边框、高对比、无修饰 |
> | Cyberpunk | 暗 | 霓虹、科幻、赛博朋克 |
> | Vaporwave | 暗 | 渐变紫粉、复古合成器 |
> | Terminal | 暗 | 等宽字体、命令行风格 |
> | Claymorphism | 亮 | 3D 粘土感、柔和阴影 |
> | Art Deco | 暗 | 金色装饰、几何奢华 |
> | Swiss Minimalist | 亮 | 网格、无衬线、瑞士平面 |
> | Academia | 亮 | 衬线、复古纸感、学院派 |
> | Neumorphism | 亮 | 软阴影、立体浮雕感 |
>
> 完整列表见 `resources/catalogue.md` 或 https://github.com/VoltAgent/awesome-design-md
> designprompts.dev 完整风格列表：https://www.designprompts.dev/

如果用户选择 **B（描述氛围）**，根据关键词推荐最匹配的 2-3 个品牌风格供选择，同时也可推荐 designprompts.dev 中对应的风格名称。

如果用户选择 **D（designprompts.dev）**，请参考第二步 2.4 节获取该风格的 prompt 内容。

### 1.2 确认应用范围

> **你希望将这个风格应用到：**
> - 整个项目（全局 CSS 变量 + 所有组件）
> - 仅配色方案（只改颜色，保留现有布局）
> - 特定页面或组件

---

## 第二步：获取 DESIGN.md

### 2.0 同步最新品牌列表（每次触发时执行）

在向用户展示品牌选项之前，先从 GitHub 仓库获取最新的品牌列表：

```
https://raw.githubusercontent.com/VoltAgent/awesome-design-md/main/README.md
```

使用 `read_url_content` 抓取后，解析出所有品牌名称和 npx 名称，作为当前可选列表。如果请求失败，回退使用 `resources/catalogue.md` 中的静态列表。

根据用户的选择，通过以下方式获取设计规范文件：

### 2.1 预设品牌风格

在项目根目录执行：

```bash
npx -y getdesign@latest add <name>
```

其中 `<name>` 是品牌标识（如 `claude`、`vercel`、`stripe`）。

执行成功后在项目根目录生成 `DESIGN.md` 文件。

### 2.2 自定义 DESIGN.md

如果用户提供了自定义文件路径或 URL：
- **本地文件**：直接使用 `view_file` 读取
- **远程 URL**：使用 `read_url_content` 或 `web_fetch` 获取内容后保存为项目根目录的 `DESIGN.md`

### 2.3 本地已有的风格文件

检查 skill 的 `resources/designs/` 目录下是否有用户请求的风格文件。这些是预置的自定义设计系统，不依赖 npx 下载：

```
C:\Users\86136\.claude\skills\frontend-design\resources\designs\
├── <style-name>/
│   └── DESIGN.md
```

如果存在匹配，直接复制到项目根目录。

### 2.4 从 designprompts.dev 获取设计 Prompt

当用户选择 designprompts.dev 风格时，按以下步骤操作：

1. **构建 URL**：风格页面 URL 格式为 `https://www.designprompts.dev/<style-slug>`
   - 例：Bauhaus → `https://www.designprompts.dev/bauhaus`
   - 例：Neo Brutalism → `https://www.designprompts.dev/neo-brutalism`
   - 风格 slug 为风格名称的小写连字符形式

2. **获取 Prompt**：使用 `mcp_grok-search_web_fetch` 抓取该页面，提取其中的设计风格描述文本（通常包含颜色、排版、视觉特征等说明）

3. **转化为 DESIGN.md**：将抓取到的 prompt 内容整理成 DESIGN.md 格式（参考第三步的 Token 分类），保存到项目根目录

4. **如抓取失败**：访问 https://www.designprompts.dev/ 获取完整风格列表，人工核对 slug 后重试

---

## 第三步：解读 DESIGN.md

读取 `DESIGN.md` 后，提取以下设计 token：

### 3.1 必须提取的 Token

| Token 类别 | 说明 | CSS 变量命名规范 |
|-----------|------|-----------------|
| **背景色** | 主背景、卡片面、输入框 | `--bg-*` |
| **文字色** | 主文字、次要、辅助、链接 | `--text-*` |
| **强调色** | 品牌主色、hover 态、coral 变体 | `--accent`, `--accent-hover` |
| **语义色** | 成功、错误、警告 | `--success`, `--danger`, `--warning` |
| **边框色** | 浅色边框、深色边框 | `--border-*` |
| **阴影** | ring shadow、whisper shadow | `--shadow-*` |
| **圆角** | 标准、小、大 | `--radius`, `--radius-sm` |
| **字体** | 标题字体（serif/sans）、正文、代码 | 直接写在 `:root` 的 font-family |

### 3.2 可选提取

- 字体层级表（Display → Body → Caption 的 size/weight/line-height）
- 间距系统（base unit + scale）
- 动画 timing function
- 响应式断点

---

## 第四步：应用到项目 CSS

### 4.1 映射策略

将提取的 token 映射到项目现有的 CSS 变量。**核心原则**：

1. **仅修改 CSS 文件**，不改动 JS 逻辑和 HTML 结构
2. **保留现有 CSS 变量名**（如项目用 `--bg-primary`），仅替换值
3. 若项目无 CSS 变量，创建 `:root` 变量层并重构现有硬编码颜色
4. 标题元素（h1, h2, `.logo`, `.modal h2` 等）应用 DESIGN.md 指定的标题字体
5. 按 DESIGN.md 的阴影哲学替换所有 `box-shadow`

### 4.2 执行顺序

按以下顺序修改，每批 ≤2 个文件，每次 ≤50 行：

1. **CSS 变量层**（`:root` 块）— 替换所有颜色/阴影/圆角变量
2. **基础样式**（reset, body, a, button）— 应用新配色
3. **组件样式**（卡片、按钮、输入框、模态框）— 逐组件替换
4. **特殊组件**（header, nav, badge, tag）— 使用品牌色

### 4.3 注意事项

- DESIGN.md 中的自定义字体（如 `Anthropic Serif`）应替换为其 **fallback 字体**（如 `Georgia`）
- 颜色变量的命名应保持项目一致性，不要直接用品牌名（`--anthropic-black`）
- 如果 DESIGN.md 指定 gradient-free，删除所有 `linear-gradient`
- 暗色/亮色主题切换时需要两套变量

---

## 第五步：验证与清理

### 5.1 视觉验证

1. 启动 dev server
2. 使用 Playwright MCP 截图关键页面（登录页、主工作区、模态框）
3. 对比 DESIGN.md 描述的视觉特征

### 5.2 清理

- 将 `DESIGN.md` 加入 `.gitignore`（设计规范文件不提交到仓库）
- 提交 CSS 变更并推送

---

## 扩展：添加新的设计风格

### 通过 npx 获取

awesome-design-md 仓库持续更新，新增品牌直接用：

```bash
npx -y getdesign@latest add <new-brand-name>
```

### 通过本地文件扩展

将自定义 DESIGN.md 放入 skill 资源目录：

```
C:\Users\86136\.claude\skills\frontend-design\resources\designs\<name>\DESIGN.md
```

文件格式应遵循 [Stitch DESIGN.md format](https://stitch.withgoogle.com/docs/design-md/format/)，包含以下章节：

1. Visual Theme & Atmosphere
2. Color Palette & Roles
3. Typography Rules
4. Component Stylings
5. Layout Principles
6. Depth & Elevation
7. Do's and Don'ts
