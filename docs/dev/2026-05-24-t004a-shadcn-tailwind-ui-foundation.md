# T004A 建立 shadcn/ui + Tailwind 前端基础

## 任务目标

为 `T004A 建立 shadcn/ui + Tailwind 前端基础` 建立可复用的前端 UI 基础，方便后续热词词典、历史记录、统计卡片和设置页使用统一组件与样式规则。

## 实际改动

- 引入 Tailwind CSS v4、`@tailwindcss/vite`、shadcn/ui 所需的 Radix primitives、`class-variance-authority`、`clsx`、`tailwind-merge`、`lucide-react` 和 `tw-animate-css`。
- 配置 Vite Tailwind 插件和 `@/*` 路径别名。
- 新增 `components.json`、`src/lib/utils.ts` 和首批 `src/components/ui/*` 组件。
- 将默认人格选择面板迁移到 shadcn/ui 组件和 Tailwind class。
- 在 `README.md` 和 `docs/solution-design.md` 中补充 UI 技术路线、依赖用途和性能边界。
- 评估 `https://getdesign.md/hp/design-md` 的 HP 风格，决定不完整注入，仅吸收克制的白底、蓝色主操作和高可读正文色。

## 为什么这么做

T005 热词词典会需要表单、选择器、开关、弹窗、列表和状态反馈。如果先建立 shadcn/ui + Tailwind 基础，后续功能可以复用同一套交互和视觉语言，减少重复 CSS 和临时组件。

shadcn/ui 的组件源码进入项目，适合在黑客松 MVP 阶段按任务逐步定制；Tailwind CSS 在构建阶段生成实际使用到的样式，按需添加组件后对运行时性能影响可控。

HP design-md 的完整风格更偏官网和产品目录。当前项目是桌面效率工具，因此只采用其中适合工具界面的清晰蓝色和白底信息层级，避免让主界面变成营销页。

## 涉及文件

- `package.json`
- `pnpm-lock.yaml`
- `vite.config.ts`
- `tsconfig.json`
- `tsconfig.node.json`
- `components.json`
- `src/lib/utils.ts`
- `src/components/ui/button.tsx`
- `src/components/ui/card.tsx`
- `src/components/ui/dialog.tsx`
- `src/components/ui/input.tsx`
- `src/components/ui/label.tsx`
- `src/components/ui/select.tsx`
- `src/components/ui/switch.tsx`
- `src/components/ui/tabs.tsx`
- `src/components/ui/textarea.tsx`
- `src/main.tsx`
- `src/styles.css`
- `README.md`
- `docs/solution-design.md`
- `docs/dev/task-tracker.md`
- `docs/dev/2026-05-24-t004a-shadcn-tailwind-ui-foundation.md`

## 工作分支与审批

- 当前工作分支：`dev`
- 是否已获批提交：否
- 是否已获批创建 `dev -> main` 的 PR：否

## 测试与验证

- `pnpm build`：通过。Vite 构建成功，生成 CSS gzip 约 6.84 kB，JS gzip 约 100.46 kB。
- `pnpm exec tsc --noEmit`：通过。
- `cargo check`：通过。
- `cargo test --test local_data_layer`：通过，6 个测试全部通过。
- 桌面端手动交互验证：未执行。当前任务主要建立前端 UI 基础，后续可在 `pnpm tauri dev` 中检查默认人格选择的实际交互。

## 未完成事项

- 尚未提交 commit。
- 尚未创建 PR。

## 后续建议

继续执行 `T005 实现热词词典`，复用本任务新增的 `Input`、`Label`、`Select`、`Switch`、`Dialog` 和 `Button`。
