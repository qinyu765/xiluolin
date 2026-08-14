# Quality attributes

Select the attributes that govern the decision. Rank them with the user instead of claiming that every attribute can be maximized.

| Attribute | Ask | Useful evidence |
| --- | --- | --- |
| Maintainability | Can a future change stay local and understandable? | change hotspots, duplication, review effort, ownership |
| Extensibility | Can a named variation be added without editing unrelated callers? | actual variants, stable interfaces, migration seams |
| Testability | Can behaviour be verified through the intended interface? | test setup, fakes, integration seams, flakiness |
| Operability | Can failures be detected, diagnosed, recovered, and upgraded safely? | logs, metrics, crash handling, rollout and rollback |
| Performance | Which latency, throughput, memory, startup, or battery budgets matter? | measurements, budgets, profiling, representative workloads |
| Security | Which trust boundaries, capabilities, secrets, and data lifetimes matter? | permission manifests, threat model, audit and failure paths |
| Portability | Which platforms, runtimes, or distribution channels must remain viable? | supported matrix, platform APIs, build and packaging constraints |
| Cost | What does this choice cost to build, operate, learn, and change? | time, complexity, infrastructure, support, opportunity cost |

For each selected attribute, write:

1. the target or budget;
2. the current evidence;
3. the option that improves it;
4. the attribute that may regress;
5. the verification that would catch that regression.

Architecture advice is incomplete when it names a quality attribute without a trade-off or a measurement strategy.
