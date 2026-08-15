---
name: evolve-software-architecture
description: Evidence-based software architecture guidance for understanding an existing repository, identifying structural friction, and choosing a durable evolution path across project types. Use when the user asks for architecture evaluation, module boundaries, extensibility, long-term maintainability, technical-debt direction, cross-module or cross-process design within an existing system, major refactoring strategy, or architecture trade-offs. Invoke explicitly for a repository health review. Keep ordinary local fixes, styling, renames, and routine dependency updates focused unless they expose an architectural decision.
---

# Evolve Software Architecture

Provide architecture guidance that lowers future change cost without pretending that one design can predict every future requirement. Ground every recommendation in the repository's current facts, project type, constraints, and evidence. Prefer a small number of deep, well-placed seams over a broad layer scheme or framework fashion.

## Operating contract

Produce advice before implementation. Stop at a decision-ready recommendation unless the user explicitly asks for a change. Treat “long-term” as reversible evolution: state what should be stable, what may vary, what is still unknown, and how to learn before committing to an expensive abstraction.

Keep these distinctions visible:

- **Fact** — directly observed in source, configuration, documentation, history, or a user statement.
- **Inference** — a reasoned interpretation of facts; state the reasoning.
- **Unknown** — important information not yet established; propose the cheapest way to learn it.
- **Constraint** — a limit that the design must respect, including platform, product, team, release, security, and operational limits.

Use the user's language for the answer. Use the project's existing domain vocabulary after locating it. Do not import web-application assumptions into desktop, SDK, CLI, mobile, data, or embedded work.

## Workflow

### 1. Establish the repository context

Inspect before judging. Read the closest `AGENTS.md` or `CLAUDE.md`, repository instructions, manifests, entry points, current architecture documents, roadmap, and relevant ADRs. Inspect the recent history and the files that have changed repeatedly. Record the repository type and confidence in that classification.

Use the read-only helper when a compact inventory is useful:

```bash
python3 <skill-root>/scripts/collect_repo_signals.py --repo <repository>
```

Read [assessment-framework.md](references/core/assessment-framework.md) for the evidence table and investigation order. Read [project-type-selection.md](references/project-types/project-type-selection.md), then load only the project-type adapter that matches the observed signals. If no adapter is justified, use the core workflow and label the missing domain knowledge as an unknown.

### 2. Model the current system

Describe the important runtime and build-time flows, ownership of state, module interfaces, process boundaries, external dependencies, and test seams. Locate where a change currently spreads and where knowledge is duplicated. Name the current seam; do not invent a target architecture before understanding the existing one.

Separate:

- symptoms from root causes;
- accidental complexity from domain complexity;
- a real variation point from a hypothetical one;
- a missing boundary from a boundary that would only add indirection.

### 3. Choose quality attributes deliberately

Read [quality-attributes.md](references/core/quality-attributes.md). Select the few attributes that actually govern this decision, rank them, and explain the trade-offs. Consider maintainability, extensibility, testability, operability, performance, security, portability, and cost as competing dimensions rather than a checklist to maximize simultaneously.

### 4. Compare options

Present at least two viable options, including keeping the current shape when it remains defensible. For each option state:

- the boundary and ownership it creates;
- the changes it enables and the assumptions it introduces;
- migration and rollback cost;
- operational and testing consequences;
- the evidence that would make the option wrong.

Use deep-module reasoning as one tool: ask whether a small interface earns its complexity through leverage and locality. Do not treat it as a universal architecture style.

### 5. Recommend an evolution path

Recommend one option only after the comparison. Split the path into reversible steps, identify the first useful vertical slice, preserve behaviour during migration, and specify observable exit criteria. State what not to build yet and what signal would justify revisiting it. Surface decisions that deserve an ADR using [decision-record.md](references/core/decision-record.md).

### 6. Verify the recommendation

Give concrete checks: tests through the intended interface, dependency or architecture checks, performance or failure-mode checks, and a review of the resulting diff. Revisit the recommendation when a new repository type, constraint, or source of evidence invalidates an assumption.

## Output contract

Structure the answer with these sections unless the user asks for a different format:

1. **Scope and confidence** — what decision is being considered and how the repository was classified.
2. **Observed facts** — evidence with paths, symbols, commands, or history references.
3. **Current friction** — the change amplification, coupling, or missing ownership that matters.
4. **Quality-attribute priorities** — ranked attributes and explicit trade-offs.
5. **Options** — at least two, including the current design when reasonable.
6. **Recommendation** — the chosen direction, rationale, and rejected alternatives.
7. **Migration and verification** — incremental steps, rollback, tests, observability, and completion criteria.
8. **Open decisions** — only questions whose answers can change the recommendation.

Mark facts, inferences, and unknowns inline when confusing them would change the decision. Keep repository-specific observations in the current review or its evaluation case; promote a rule into a core reference only after it survives materially different project types.

## Resources

- [assessment-framework.md](references/core/assessment-framework.md) — evidence-first repository investigation.
- [quality-attributes.md](references/core/quality-attributes.md) — selecting and balancing quality attributes.
- [decision-record.md](references/core/decision-record.md) — ADR-ready decision format.
- [project-type-selection.md](references/project-types/project-type-selection.md) — adapter selection and promotion rules.
- [desktop-tauri.md](references/project-types/desktop-tauri.md) — desktop/Tauri-specific concerns.
