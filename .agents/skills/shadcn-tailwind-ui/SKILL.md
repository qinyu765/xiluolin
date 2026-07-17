---
name: shadcn-tailwind-ui
description: Use when adding or modifying React UI in this Tauri project with shadcn/ui, Tailwind CSS, Radix primitives, reusable components, design tokens, or frontend style conventions.
---

# shadcn/ui + Tailwind UI

## Purpose

This project uses shadcn/ui + Tailwind CSS as the preferred UI foundation for the XiLuoLin desktop voice input assistant.

Use this skill when building or refactoring visible React UI, including settings, persona selection, hotword dictionary, history, statistics, recording controls, dialogs, tabs, forms, and empty/error states.

## Project Constraints

- Keep UI aligned with the product: a quiet, efficient desktop assistant for voice input, writing, office work, and programming.
- Prefer dense, scannable, work-focused layouts over landing-page composition.
- Use shadcn/ui components for common controls before writing custom widgets.
- Use Tailwind utility classes for layout, spacing, typography, state, and responsive behavior.
- Keep component changes local and reusable only when repeated by real UI needs.
- Do not add marketing sections, decorative gradients, visual clutter, or unrelated product features.

## Dependency Rules

- Install shadcn/ui through the official CLI when implementing the UI foundation task.
- Use Tailwind v4 for the Vite + React project unless the project already pins another version.
- Add shadcn/ui components incrementally with the CLI, starting from the components actually needed.
- Keep `components.json`, Tailwind entry CSS, path aliases, and generated `components/ui/*` files in Git.
- Document every new dependency and its purpose in `README.md`.

## Component Selection

Start with these components for the project UI foundation:

- `button`
- `card`
- `input`
- `label`
- `select`
- `textarea`
- `tabs`
- `switch`
- `dialog`

Add more only when a task needs them. For T005 hotwords, likely additions are table/list layout, badge, dropdown menu, and alert.

## Styling Rules

- Keep Tailwind classes explicit and searchable. Avoid dynamically building class names such as `text-${tone}-600`.
- Use `cn()` for conditional classes.
- Prefer shadcn/ui variants before adding custom CSS.
- Use small radius and restrained borders for tool surfaces.
- Avoid nested cards. Use cards for discrete repeated items, dialogs, and bounded tools.
- Keep text sizes stable; do not scale font size with viewport width.
- Ensure buttons, select triggers, tabs, and inputs have stable dimensions and no text overflow.

## Accessibility

- Prefer Radix-backed shadcn/ui components for keyboard behavior and ARIA semantics.
- Every input needs a visible label or equivalent accessible name.
- Keep focus states visible.
- Use dialogs for blocking edits and inline panels for quick edits.

## Verification

For UI foundation or component changes, run:

```bash
pnpm build
cargo check
cargo test --test local_data_layer
```

For visible UI changes, also run the local app and inspect the relevant viewport when feasible:

```bash
pnpm tauri dev
```

Record any skipped verification and exact blocker in the task document.
