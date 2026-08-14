# Project-type adapter selection

Adapters add only constraints and signals that are specific to a project type. The core workflow remains responsible for evidence, quality attributes, alternatives, migration, and verification.

## Selection sequence

1. Collect identity signals from manifests, entry points, build files, packaging, and runtime topology.
2. Match a project type only when at least two independent signals agree.
3. Load one adapter and record why it was selected.
4. Keep adapter-specific claims separate from core claims.
5. If no adapter fits, continue with the core workflow and label the missing knowledge as an unknown.

## Adapter contract

Every adapter reference should define:

- recognition signals and confidence limits;
- project-specific quality attributes and budgets;
- state, process, deployment, and integration boundaries to inspect;
- platform failure modes and security constraints;
- common over-generalizations to avoid;
- tests, measurements, and operational checks;
- criteria for promoting a lesson into the core.

## Promotion gate

Keep a lesson in an evaluation case until it is either a platform invariant or has been observed in at least two repositories of that type. Promote a lesson into the core only after it survives at least two materially different project types without framework-specific assumptions.

The first adapter is Desktop/Tauri. Future SDK, Web/CLI, Mobile, and Data/AI adapters must be added one at a time and evaluated independently.
