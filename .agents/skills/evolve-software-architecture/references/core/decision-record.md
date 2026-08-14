# Decision record

Use this format for a decision that future reviewers would otherwise reopen without its reasoning.

```markdown
# ADR-NNNN: <decision title>

- Status: proposed | accepted | superseded | rejected
- Date: YYYY-MM-DD
- Scope: <module, process, or repository area>

## Context

Facts, constraints, and the decision that must be made.

## Decision drivers

The ranked quality attributes, compatibility requirements, and operational limits.

## Options considered

For each option: shape, benefits, costs, risks, migration, rollback, and evidence that would invalidate it.

## Decision

The selected option and the reason it wins under the stated drivers.

## Consequences

New responsibilities, constraints, tests, observability, and follow-up work.

## Revisit conditions

Concrete signals that justify reopening this decision.
```

Record a decision when its rationale is load-bearing for a future architecture review. Do not create an ADR for an ordinary implementation detail or a temporary preference.
