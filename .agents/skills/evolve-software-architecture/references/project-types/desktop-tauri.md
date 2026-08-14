# Desktop and Tauri adapter

## Recognition signals

Treat a repository as Desktop/Tauri when at least two of these agree:

- `src-tauri/tauri.conf.json`, `tauri.conf.json`, or an equivalent Tauri configuration;
- a Rust `Cargo.toml` paired with a web UI package manifest;
- native desktop packaging, updater, capability, or bundle configuration;
- an executable desktop shell that hosts a web view;
- OS integration such as global shortcuts, tray, permissions, audio devices, filesystem, or window lifecycle.

## Inspect first

Map the boundaries between:

- the web UI and the Rust/native process;
- commands, events, and shared state across IPC;
- audio, filesystem, OS, and other native adapters;
- durable state, in-memory session state, and configuration;
- startup, shutdown, updater, crash, offline, and permission flows;
- unit, Rust integration, UI, IPC, and packaged-app tests.

Trace one real user flow across these boundaries. Record who owns each state transition and what happens when the other process is unavailable, restarted, or on an older version.

## Quality attributes that commonly dominate

Choose only those evidenced by the task:

- **Process-boundary stability** — IPC contracts, versioning, serialization, error mapping, and event ordering.
- **Platform safety** — capabilities, permissions, OS differences, filesystem/audio access, and updater behaviour.
- **Startup and lifecycle correctness** — initialization order, teardown, retries, and recovery after native failure.
- **Testability** — replacing native adapters and testing the contract without requiring a packaged desktop runtime.
- **Resource behaviour** — memory, CPU, audio latency, battery, bundle size, and offline operation.

## Common traps to test rather than assume

- Treating an IPC command as a free local function and ignoring serialization, versioning, or failure.
- Letting UI state, Rust state, and durable state become interchangeable owners.
- Building a generic native abstraction before two real OS or provider variants exist.
- Testing only pure UI/Rust functions while the bugs live in lifecycle, IPC, permissions, or packaging.
- Applying browser deployment or server scaling advice to a local desktop process.

## Verification

Prefer checks that cross the intended seam:

- contract tests for commands/events and error shapes;
- native-adapter fakes for UI and Rust logic;
- startup, shutdown, restart, offline, permission-denied, and updater scenarios;
- packaged-app smoke tests on each supported OS where the risk warrants it;
- measurements for startup, audio latency, memory, and bundle size when they are decision drivers.
