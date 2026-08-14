# Assessment framework

Use this sequence to keep architecture advice evidence-led and portable across project types.

## Investigation order

1. **Instructions and vocabulary** — read the nearest `AGENTS.md`, `CLAUDE.md`, contribution rules, domain glossary, roadmap, and ADRs.
2. **Identity** — inspect manifests, build files, entry points, deployment targets, process model, and generated-code boundaries.
3. **Change history** — inspect recent commits and repeated hotspots to find where change cost is already visible.
4. **Runtime flows** — trace the user or API path through state ownership, boundaries, persistence, external systems, and failure handling.
5. **Seams and tests** — identify the interfaces callers and tests cross, and note where tests need to reach into implementations.
6. **Constraints** — record platform, compatibility, release, security, latency, offline, team, and operational constraints.

## Evidence table

Use a compact table in the review:

| Claim | Evidence | Kind | Confidence | Consequence |
| --- | --- | --- | --- | --- |
| A statement about the current system | file, symbol, command, commit, or user statement | fact / inference / unknown | high / medium / low | why it changes the decision |

Do not turn a missing search result into proof of absence. If a repository instruction conflicts with a generic practice, follow the repository instruction and call out the conflict.

## Classification signals

Classify from multiple signals, not a directory name alone:

- language and package manifests;
- executable entry points and process boundaries;
- deployment and distribution target;
- persistence and integration model;
- test and build tooling;
- platform-specific configuration;
- the user's described failure mode.

When signals disagree, keep the classification provisional and use only the core workflow until the uncertainty is resolved.

## Architecture vocabulary

Use these terms consistently when they fit:

- **Module** — anything with an interface and an implementation.
- **Interface** — everything a caller must know, including invariants, ordering, errors, configuration, and performance.
- **Seam** — the place where behaviour can be changed without editing the caller.
- **Adapter** — a concrete implementation that satisfies an interface at a seam.
- **Depth** — leverage delivered through a small interface.
- **Locality** — how much change, knowledge, and verification stay in one place.

These terms describe an observation; they do not force a layered architecture.
