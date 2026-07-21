# Architecture

## 1. Philosophy

Aster is a native Linux process monitor and manager for developers. The architecture is designed around four non-negotiable rules:

1. **Domain logic never depends on the UI.** `aster-core` is a standalone library. If you replace the TUI with a web interface, the core doesn't change.
2. **Each module has exactly one reason to change.** If CPU reading logic changes, it lives in one file. If signal sending changes, it lives in another.
3. **Modules communicate through explicit public interfaces.** No module reaches into another module's internals.
4. **The architecture supports growth without restructuring.** Adding a feature means adding files, not rewriting existing ones.

---

## 2. Complete File Tree

```
Aster/
├── Cargo.toml                          Workspace root
├── LICENSE
├── README.md
├── ARCHITECTURE.md                     This document
├── VISION.md
├── ROADMAP.md
│
├── aster-core/                         Domain logic library
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                      Crate root — module declarations + re-exports
│       ├── error.rs                    AsterError enum
│       │
│       ├── process/                    Process domain
│       │   ├── mod.rs                  Module declarations
│       │   ├── model.rs                ProcessInfo, ProcessState, Pid, User
│       │   ├── enumerator.rs           /proc parsing → Vec<ProcessInfo>
│       │   ├── snapshot.rs             Snapshot type + diff computation
│       │   └── tree.rs                 Parent-child process hierarchy
│       │
│       ├── system/                     System-level metrics
│       │   ├── mod.rs                  Module declarations
│       │   ├── cpu.rs                  CPU info and per-core usage
│       │   └── memory.rs              RAM and swap usage
│       │
│       ├── monitor/                    Real-time monitoring
│       │   ├── mod.rs                  Module declarations
│       │   ├── collector.rs            Periodic collection + diff emission
│       │   └── history.rs             Ring buffer of historical snapshots
│       │
│       └── signal/                     POSIX signals
│           ├── mod.rs                  Module declarations
│           └── sender.rs              Signal dispatch via libc::kill
│
├── aster-ui/                           TUI binary
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs                     Entry point, terminal lifecycle
│       ├── app.rs                      Application state machine
│       ├── event.rs                    Event loop, input dispatch
│       ├── ui.rs                       Rendering coordinator
│       └── view/                       Individual screens
│           ├── mod.rs                  Module declarations
│           ├── process_list.rs         Main process table
│           ├── process_detail.rs       Single process inspection
│           └── system_overview.rs      CPU/memory dashboard
│
├── assets/
│   ├── icons/
│   └── screenshots/
│
├── docs/
├── examples/
└── .github/workflows/
```

---

## 3. Workspace Definition

### Root `Cargo.toml`

```toml
[workspace]
members = ["aster-core", "aster-ui"]
resolver = "2"
```

Two crates. No more, no less for MVP. The workspace is flat — no nested workspaces, no feature flags across crates. Each crate owns its own `Cargo.toml` completely.

---

## 4. Crate Breakdown

### `aster-core` — Domain Logic Library

| Attribute | Value |
|-----------|-------|
| Type | `lib` |
| Edition | 2024 |
| Purpose | All business logic, models, and OS interaction |
| Public API | Functions and types that `aster-ui` calls |

**Allowed dependencies:**

| Dependency | Why | Version strategy |
|------------|-----|------------------|
| `libc` | POSIX signal sending (`kill` syscall). The only justified OS-level dependency. | Stable, pinned minor. |

**Forbidden dependencies:**

| Category | Reason |
|----------|--------|
| TUI frameworks (`ratatui`, `crossterm`, `termion`) | UI belongs in `aster-ui` only |
| Async runtimes (`tokio`, `async-std`) | Synchronous reads from `/proc` are sufficient. Async adds complexity without benefit here. |
| Serialization (`serde`, `toml`, `json`) | Not needed for MVP. Add when configuration is implemented. |
| Logging (`log`, `tracing`, `env_logger`) | Logging is a UI concern. Core returns errors; the UI decides how to display them. |
| CLI parsing (`clap`) | CLI is a UI concern. |

**Why these restrictions:** Every dependency in the core becomes a transitive dependency of the entire project. The core should be a thin, focused library. If a dependency is only used by one module, evaluate whether that module belongs in the core at all.

---

### `aster-ui` — Terminal User Interface

| Attribute | Value |
|-----------|-------|
| Type | `bin` |
| Edition | 2024 |
| Purpose | Render TUI, handle input, translate user actions into core calls |
| Public API | None (binary crate) |

**Allowed dependencies:**

| Dependency | Why | Version strategy |
|------------|-----|------------------|
| `aster-core` | All domain logic | `path = "../aster-core"` |
| `ratatui` | TUI framework (immediate-mode rendering) | Latest stable. |
| `crossterm` | Terminal backend (used by ratatui) | Latest stable, must match ratatui's requirement. |

**Optional dependencies (Phase 2+):**

| Dependency | When | Why |
|------------|------|-----|
| `clap` | When CLI flags are needed | `--sort`, `--filter`, `--pid` arguments |
| `serde` + `toml` | When config files are needed | User configuration |

**Forbidden dependencies:**

| Category | Reason |
|----------|--------|
| `/proc` parsing code | That's `aster-core`'s job |
| Signal sending code | That's `aster-core`'s job |
| Any OS interaction | Core handles all OS interaction |

---

## 5. Module Breakdown — `aster-core`

### `error.rs`

**File path:** `aster-core/src/error.rs`

**Responsibility:** Define the single error type for the entire core crate.

**What lives here:**
- The `AsterError` enum with variants: `Io`, `Parse`, `Permission`, `Os`, `NotFound`.
- `impl From<std::io::Error>` for ergonomic error propagation.
- `impl Display` and `impl std::error::Error`.

**What does NOT live here:**
- Error handling logic (that's the caller's job).
- User-facing messages (that's the UI's job).
- Logging or printing.

**Growth:** New variants are added as new modules encounter new error conditions. The error type grows vertically (more variants), never horizontally (no error modules).

**Depends on:** Nothing (standalone).

---

### `process/`

**Directory path:** `aster-core/src/process/`

**Responsibility:** Everything related to process data: modeling, enumeration, snapshots, and hierarchy.

**Why this module exists:** Processes are the central domain concept. Co-locating all process logic achieves high cohesion.

#### `model.rs`

**File path:** `aster-core/src/process/model.rs`

**Responsibility:** Core data types. Pure data structures with no I/O.

**What lives here:**
- `Pid` — Process identifier (wraps `i32` or `u32`).
- `ProcessState` — Enum: `Running`, `Sleeping`, `Stopped`, `Zombie`, `Idle`, etc.
- `ProcessInfo` — Struct with all process attributes: pid, ppid, name, state, user, cpu%, mem%, threads, start_time, etc.
- `User` — Process owner (uid, username).

**What does NOT live here:**
- Any function that reads `/proc`. That's `enumerator.rs`.
- Snapshot logic. That's `snapshot.rs`.
- Any `impl` block that performs I/O. Types here are data carriers.

**Growth:** New fields are added to `ProcessInfo` as new `/proc` files are parsed. New types are added for new process attributes (e.g., `ProcessIo`, `ProcessLimits`, `ProcessNamespaces`).

**Depends on:** Nothing (pure types).

---

#### `enumerator.rs`

**File path:** `aster-core/src/process/enumerator.rs`

**Responsibility:** Read `/proc`, parse process entries, return a list of `ProcessInfo`.

**What lives here:**
- `enumerate() -> Result<Vec<ProcessInfo>>` — Reads `/proc/[pid]/stat`, `/proc/[pid]/status`, `/proc/[pid]/cmdline` for each PID directory.
- Helper functions to parse individual `/proc` files.
- Filtering logic (skip kernel threads, handle permission errors gracefully).

**What does NOT live here:**
- Type definitions (those go in `model.rs`).
- Snapshot or diff logic (that goes in `snapshot.rs`).
- System metrics (that goes in `system/`).
- Signal sending (that goes in `signal/`).

**Growth:** New `/proc` files are parsed as new process attributes are exposed. Parsing becomes more robust (handling edge cases in `/proc` format).

**Depends on:** `model` (for `ProcessInfo`, `Pid`, `ProcessState`), `error` (for `AsterError`).

---

#### `snapshot.rs`

**File path:** `aster-core/src/process/snapshot.rs`

**Responsibility:** Take a point-in-time snapshot of all processes. Compute diffs between consecutive snapshots.

**What lives here:**
- `Snapshot` — A complete process state at a moment in time. Contains `Vec<ProcessInfo>` indexed by `Pid`.
- `SnapshotDiff` — The difference between two snapshots: new processes, dead processes, changed processes.
- `take() -> Result<Snapshot>` — Calls `enumerator::enumerate()` and wraps the result.
- `diff(old: &Snapshot, new: &Snapshot) -> SnapshotDiff` — Compares two snapshots.

**What does NOT live here:**
- Enumeration logic (that's `enumerator.rs`). Snapshot calls enumerator but doesn't parse `/proc` itself.
- Periodic collection (that's `monitor/collector.rs`).
- History storage (that's `monitor/history.rs`).

**Growth:** Diffing logic becomes more sophisticated — tracking resource usage deltas, detecting state transitions, computing per-process CPU/memory deltas.

**Depends on:** `model`, `enumerator`, `error`.

---

#### `tree.rs`

**File path:** `aster-core/src/process/tree.rs`

**Responsibility:** Build a parent-child process hierarchy from snapshot data.

**What lives here:**
- `ProcessTree` — A tree structure where each node is a `Pid` with children `Vec<ProcessTree>`.
- `build(snapshot: &Snapshot) -> ProcessTree` — Constructs the tree from `ppid` fields.

**What does NOT live here:**
- Snapshot logic (that's `snapshot.rs`).
- Rendering logic (that's `aster-ui`'s job).
- Any `/proc` parsing.

**Growth:** Tree diffing (showing which branches appeared/disappeared between snapshots). Flat vs. tree view toggling.

**Depends on:** `model` (for `Pid`), `snapshot` (for `Snapshot`).

---

### `system/`

**Directory path:** `aster-core/src/system/`

**Responsibility:** System-level metrics: CPU, memory, load.

**Why this module exists:** System metrics are orthogonal to process data. Keeping them separate prevents the process module from becoming a god module.

#### `cpu.rs`

**File path:** `aster-core/src/system/cpu.rs`

**Responsibility:** Read CPU info and per-core usage from `/proc/stat` and `/proc/cpuinfo`.

**What lives here:**
- `CpuInfo` — Struct with overall and per-core CPU usage percentages.
- `read() -> Result<CpuInfo>` — Parses `/proc/stat`.

**What does NOT live here:**
- Process data (that's `process/`).
- Memory data (that's `memory.rs`).
- Historical CPU data (that goes in `monitor/history.rs` when implemented).

**Growth:** Per-core metrics become more detailed. Load average is added. CPU frequency and temperature are added.

**Depends on:** `error`.

---

#### `memory.rs`

**File path:** `aster-core/src/system/memory.rs`

**Responsibility:** Read RAM and swap usage from `/proc/meminfo`.

**What lives here:**
- `MemoryInfo` — Struct with total, used, available, swap_total, swap_used.
- `read() -> Result<MemoryInfo>` — Parses `/proc/meminfo`.

**What does NOT live here:**
- Process memory usage (that's in `process/model.rs` as part of `ProcessInfo`).
- CPU data (that's `cpu.rs`).
- Historical memory data (that goes in `monitor/history.rs`).

**Growth:** Detailed memory breakdown (buffers, cache, huge pages, slab). Memory pressure indicators.

**Depends on:** `error`.

---

### `monitor/`

**Directory path:** `aster-core/src/monitor/`

**Responsibility:** Real-time monitoring orchestration. Collects snapshots, computes diffs, maintains history, emits events.

**Why this module exists:** Monitoring is a cross-cutting concern that ties together process and system data. It has its own lifecycle (start, tick, stop) that doesn't belong in `process/` or `system/`.

#### `collector.rs`

**File path:** `aster-core/src/monitor/collector.rs`

**Responsibility:** Periodic snapshot collection. Manages the collection loop. Computes diffs. Emits events.

**What lives here:**
- `Collector` — Stateful struct that holds the last snapshot and collection interval.
- `CollectorEvent` — Enum: `SnapshotReady(SnapshotDiff)`, `ProcessDied(Pid)`, `ProcessBorn(Pid)`, etc.
- `Collector::new() -> Collector` — Creates a new collector.
- `Collector::tick() -> Result<CollectorEvent>` — Takes a new snapshot, diffs with previous, emits event.

**What does NOT live here:**
- UI rendering (that's `aster-ui`).
- Signal sending (that's `signal/`).
- History storage (that's `history.rs`).

**Growth:** Event types become richer. Collection strategies become configurable (interval, on-demand, event-driven). Multi-source collection (process + system + disk).

**Depends on:** `process` (for snapshots), `system` (for CPU/memory), `error`.

---

#### `history.rs`

**File path:** `aster-core/src/monitor/history.rs`

**Responsibility:** Store historical snapshots in a ring buffer for trend analysis and graphing.

**What lives here:**
- `History` — Ring buffer of `(Timestamp, Snapshot)` pairs.
- `History::record(snapshot: &Snapshot)` — Add a snapshot to the buffer.
- `History::last_n(n: usize) -> Vec<&Snapshot>` — Get the last N snapshots.
- `History::process_history(pid: Pid) -> Vec<ProcessInfo>` — Get historical data for a specific process.

**What does NOT live here:**
- Snapshot collection (that's `collector.rs`).
- Graph rendering (that's `aster-ui`'s job).
- Data persistence (MVP is in-memory only).

**Growth:** Configurable buffer size. Persistence to disk. Aggregation (min, max, avg over time windows).

**Depends on:** `process` (for `Snapshot`, `Pid`, `ProcessInfo`).

---

### `signal/`

**Directory path:** `aster-core/src/signal/`

**Responsibility:** Send POSIX signals to processes.

**Why this module exists:** Signal sending is a distinct operation from monitoring. It has its own error modes (permission denied, process not found, invalid signal) and its own testing requirements.

#### `sender.rs`

**File path:** `aster-core/src/signal/sender.rs`

**Responsibility:** Send a specific signal to a specific PID.

**What lives here:**
- `Signal` — Enum: `SigTerm`, `SigKill`, `SigStop`, `SigCont`, `SigUsr1`, `SigUsr2`, `Sighup`, etc.
- `send(pid: Pid, signal: Signal) -> Result<()>` — Calls `libc::kill`.
- Permission checking before sending.

**What does NOT live here:**
- Process enumeration (that's `process/enumerator.rs`).
- Signal handling (receiving signals). This module only sends.
- UI feedback (that's `aster-ui`'s job).

**Growth:** Batch signal sending. Signal groups. Custom signal handling. Confirmation logic.

**Depends on:** `model` (for `Pid`), `error`, `libc`.

---

## 6. Module Breakdown — `aster-ui`

### `main.rs`

**File path:** `aster-ui/src/main.rs`

**Responsibility:** Entry point. Initialize terminal, create application instance, run event loop, restore terminal on exit.

**What lives here:**
- `main()` function.
- Terminal initialization (`crossterm::terminal::enable_raw_mode`).
- Application creation (`App::new()`).
- Event loop invocation (`event::run()`).
- Cleanup on panic or normal exit.

**What does NOT live here:**
- Business logic. All domain calls go through `app.rs`.
- Rendering logic. That's `ui.rs`.
- Event handling. That's `event.rs`.

**Growth:** CLI argument parsing with `clap`. Configuration file loading. Signal handling for graceful shutdown (Ctrl+C).

---

### `app.rs`

**File path:** `aster-ui/src/app.rs`

**Responsibility:** Application state machine. Holds all mutable UI state. Orchestrates calls to `aster-core`.

**What lives here:**
- `App` struct — Current view, selected process, sort order, filter criteria, collector state, history.
- State transition methods: `next_process()`, `previous_process()`, `select_process()`, `change_sort()`, `toggle_filter()`.
- Core delegation: methods that call `aster_core::enumerator::enumerate()`, `aster_core::signal::sender::send()`, etc.

**What does NOT live here:**
- Rendering logic (that's `ui.rs`).
- Event loop logic (that's `event.rs`).
- Any `/proc` parsing or OS interaction (that's `aster-core`).

**Growth:** New states for new views. Undo/redo for process management actions. Bookmarks/pinned processes.

**Depends on:** `aster_core` (all domain types and functions).

---

### `event.rs`

**File path:** `aster-ui/src/event.rs`

**Responsibility:** Event loop. Poll terminal events, translate them into application actions, dispatch to `App`.

**What lives here:**
- `run(app: &mut App)` — The main event loop.
- Event polling (`crossterm::event::read()`).
- Key mapping: arrow keys → navigation, Enter → select, `k`/`s` → signal, `q` → quit.
- Tick handling (periodic refresh for real-time updates).

**What does NOT live here:**
- Rendering logic (that's `ui.rs`).
- Business logic (that's `app.rs` and `aster-core`).
- Terminal setup/teardown (that's `main.rs`).

**Growth:** Mouse support. Custom keybindings. Event recording/replay for testing.

**Depends on:** `crossterm` (for terminal events), `app` (for dispatching).

---

### `ui.rs`

**File path:** `aster-ui/src/ui.rs`

**Responsibility:** Render the TUI. Takes the current `App` state and renders the appropriate view.

**What lives here:**
- `render(frame: &mut Frame, app: &App)` — Top-level render function that dispatches to the active view.
- Layout logic (splits, constraints, margins).
- Shared rendering helpers (status bar, title bar, borders).

**What does NOT live here:**
- Business logic (that's `app.rs`).
- Event handling (that's `event.rs`).
- State management (that's `app.rs`).

**Growth:** Theme support. Responsive layouts. Conditional rendering based on terminal size.

**Depends on:** `ratatui` (for rendering primitives), `app` (for state to render).

---

### `view/`

**Directory path:** `aster-ui/src/view/`

**Responsibility:** Individual screen implementations. Each view is a self-contained rendering module.

#### `process_list.rs`

**Responsibility:** Render the main process table. Columns: PID, Name, State, CPU%, MEM%, User, Threads.

**Growth:** Column customization. Inline sparklines. Color coding by state.

#### `process_detail.rs`

**Responsibility:** Render detailed view of a single process. Shows all `ProcessInfo` fields, open files, environment variables.

**Growth:** Tabbed sub-views (files, environment, limits, namespaces).

#### `system_overview.rs`

**Responsibility:** Render CPU and memory gauges, load average, top processes.

**Growth:** CPU history graph. Memory pressure indicator. Disk I/O panel.

---

## 7. Inter-Module Communication

### Dependency Graph Within `aster-core`

```
                    error
                   / | \ \
                  /  |  \ \
            model    |   \  \
           / | \ \   |    \  \
          /  |  \ \  |     \  \
   enumerator |  sender     \  \
        \     |    |          \  \
     snapshot |    |        cpu  memory
          \   |    |         |    |
       tree   |    |         |    |
              |    |         |    |
           collector ────────┘    |
              |                   |
           history ──────────────┘
```

**Rules:**
- Arrows point from dependent to dependency.
- No cycles. The graph is a DAG.
- `error` is at the root (used by all modules that do I/O).
- `model` is near the root (pure types, used by many modules).
- `system/cpu` and `system/memory` are independent leaves.
- `monitor/collector` is the integration point (depends on `process` and `system`).

### Communication Patterns

| Pattern | Where | Example |
|---------|-------|---------|
| Function call | Most interactions | `collector.tick()` calls `snapshot::take()` |
| Type sharing | Cross-module | `signal::sender` uses `model::Pid` |
| Event emission | monitor → UI | `Collector::tick()` returns `CollectorEvent` |
| Error propagation | All I/O modules | `enumerator::enumerate()` returns `Result<Vec<ProcessInfo>, AsterError>` |

### Inter-Crate Communication

`aster-ui` calls `aster-core` through its public API:

```
aster_core::error::AsterError
aster_core::process::model::{Pid, ProcessInfo, ProcessState}
aster_core::process::enumerator::enumerate
aster_core::process::snapshot::{take, diff, Snapshot, SnapshotDiff}
aster_core::process::tree::build
aster_core::system::cpu::read
aster_core::system::memory::read
aster_core::monitor::collector::{Collector, CollectorEvent}
aster_core::monitor::history::History
aster_core::signal::sender::{send, Signal}
```

The UI never reaches into internal module paths. It only uses what `lib.rs` re-exports.

---

## 8. Interface Contracts

These are the function signatures that form the boundary between `aster-core` and `aster-ui`. They are the public API.

### Process

```
process::enumerator::enumerate() -> Result<Vec<ProcessInfo>>
process::snapshot::take() -> Result<Snapshot>
process::snapshot::diff(old: &Snapshot, new: &Snapshot) -> SnapshotDiff
process::tree::build(snapshot: &Snapshot) -> ProcessTree
```

### System

```
system::cpu::read() -> Result<CpuInfo>
system::memory::read() -> Result<MemoryInfo>
```

### Monitor

```
monitor::collector::Collector::new(interval: Duration) -> Collector
monitor::collector::Collector::tick(&mut self) -> Result<CollectorEvent>
monitor::history::History::new(capacity: usize) -> History
monitor::history::History::record(&mut self, snapshot: &Snapshot)
monitor::history::History::last_n(&self, n: usize) -> Vec<&Snapshot>
```

### Signal

```
signal::sender::send(pid: Pid, signal: Signal) -> Result<()>
```

---

## 9. Dependency Rules — Summary

### `aster-core` Cargo.toml

```toml
[dependencies]
libc = "0.2"

[dev-dependencies]
# Testing utilities added as needed
```

### `aster-ui` Cargo.toml

```toml
[dependencies]
aster-core = { path = "../aster-core" }
ratatui = "0.29"
crossterm = "0.28"

[dev-dependencies]
# Testing utilities added as needed
```

### Rules

1. `aster-core` depends on `libc` only. No other external crates.
2. `aster-ui` depends on `aster-core`, `ratatui`, and `crossterm`. Nothing else for MVP.
3. No crate depends on itself (no path dependencies within a crate).
4. No optional dependencies for MVP. Feature flags are added when configuration is implemented.
5. Version pinning: use compatible version ranges (`"0.2"` not `"0.2.18"`), except for `libc` which is pinned more tightly due to FFI stability concerns.

---

## 10. Coupling Avoidance

### What keeps coupling low

| Mechanism | How it works |
|-----------|--------------|
| **One-way dependency** | Core never imports from UI. UI imports from core. |
| **Module independence** | `system/` doesn't know about `process/`. `signal/` doesn't know about `monitor/`. |
| **Trait-based abstraction** | When multiple implementations are needed (mock `/proc` for testing), traits are introduced at the module boundary. Not before. |
| **No global state** | All state is explicit in function parameters or struct fields. No `static mut`, no thread-local state in core. |
| **Minimal public API** | Each module exports only what `lib.rs` re-exports. Internal helpers are `pub(crate)` or private. |
| **Type-based interfaces** | Modules exchange data through structs and enums, not raw strings or integers. |

### What introduces coupling (and should be avoided)

| Anti-pattern | Symptom | Solution |
|--------------|---------|----------|
| God module | One file > 500 lines, does everything | Split by responsibility |
| Circular dependency | Module A imports B, B imports A | Extract shared types to a third module |
| Leaking abstractions | UI code references `/proc/[pid]/stat` | Core abstracts OS details behind types |
| Premature traits | `trait ProcessEnumerator` with one implementation | Wait until you have 2+ implementations (real + mock) |
| Stringly-typed APIs | `fn send(pid: &str, signal: &str)` | Use strong types: `Pid`, `Signal` |

---

## 11. Testing Strategy

### `aster-core` — Unit and Integration Tests

| Module | Test approach | Mock strategy |
|--------|---------------|---------------|
| `error` | Unit tests on error variants and conversions | None needed |
| `process::model` | Property tests on type invariants | None needed (pure types) |
| `process::enumerator` | Integration tests against real `/proc` | None (test on real Linux) |
| `process::snapshot` | Unit tests with mock enumerator data | Inject pre-built `Vec<ProcessInfo>` |
| `process::tree` | Unit tests with known parent-child relationships | Inject pre-built `Snapshot` |
| `system::cpu` | Integration tests against real `/proc/stat` | None (test on real Linux) |
| `system::memory` | Integration tests against real `/proc/meminfo` | None (test on real Linux) |
| `monitor::collector` | Unit tests with mock snapshots | Inject `Snapshot` objects |
| `monitor::history` | Unit tests on ring buffer behavior | None (pure data structure) |
| `signal::sender` | Integration tests (send `SIGUSR1` to self) | None (test with real signals) |

### `aster-ui` — Component and Snapshot Tests

| Module | Test approach |
|--------|---------------|
| `app` | Unit tests on state transitions (select next, sort change) |
| `event` | Unit tests on event dispatch (key → action mapping) |
| `ui` | Snapshot tests using ratatui's `TestBackend` |

### Test file locations

```
aster-core/
└── src/
    ├── process/
    │   ├── model.rs
    │   ├── enumerator.rs
    │   ├── snapshot.rs
    │   └── tree.rs
    └── tests/              ← Integration tests
        └── process_integration.rs

aster-ui/
└── src/
    └── app.rs
    └── tests/              ← Unit tests for app state
        └── app_test.rs
```

Rust convention: unit tests live in the same file (inside `#[cfg(test)]` blocks). Integration tests live in `tests/` directories. Both patterns are valid; use whichever fits the module.

---

## 12. Growth Strategy

### Phase 1 — MVP (current)

Process enumeration, basic TUI with process table, signal sending.

**Modules in scope:**
- `process/model.rs`, `process/enumerator.rs`, `process/snapshot.rs`
- `system/cpu.rs`, `system/memory.rs`
- `monitor/collector.rs`
- `signal/sender.rs`
- `error.rs`
- `aster-ui`: `main.rs`, `app.rs`, `event.rs`, `ui.rs`, `view/process_list.rs`

### Phase 2 — Enhanced monitoring

- Add `process/tree.rs` — Process hierarchy.
- Add `monitor/history.rs` — Historical data ring buffer.
- Add `view/process_detail.rs` — Single process inspection.
- Add `view/system_overview.rs` — CPU/memory dashboard.
- Add CPU/memory history graphs to the TUI.

### Phase 3 — Advanced features

- Add filtering and search (in `aster-core`, exposed via public API).
- Add configuration file support (user preferences: default sort, column selection, refresh rate).
- Add process bookmarks/pinned processes.
- Add batch signal operations.

### Phase 4 — Polish and packaging

- Add CLI argument parsing (`clap` in `aster-ui`).
- Add man page generation.
- Add packaging scripts (AUR, deb, rpm, Flatpak).
- Add accessibility features (high contrast themes, screen reader support).

### How each phase extends the architecture

| Phase | What's added | Where it goes | Existing code changes? |
|-------|-------------|---------------|----------------------|
| 1 | Core + TUI | New files only | No |
| 2 | Tree, history, views | New files in existing modules | No |
| 3 | Filtering, config | New files + new fields in existing types | Minimal (new fields) |
| 4 | CLI, packaging | New files + Cargo.toml changes | No |

The architecture accommodates growth by ensuring that **new features are additions, not modifications**. You never need to restructure existing modules to add new ones.

---

## 13. Conventions

### Naming

- Modules: `snake_case` (Rust convention).
- Types: `PascalCase`.
- Functions: `snake_case`.
- Files match module names exactly.
- `mod.rs` files contain only `pub mod` declarations, no logic.

### File size guidelines

- `model.rs` may grow large (many fields in `ProcessInfo`). This is acceptable — it's a data definition file.
- Logic files (`enumerator.rs`, `collector.rs`, `sender.rs`) should stay under 200 lines per function. If a file exceeds 300 lines, evaluate whether it should be split.
- `error.rs` should stay under 50 lines. If it grows beyond that, the error type is too broad.

### Documentation

- Each module has a doc comment at the top of `mod.rs` explaining its purpose.
- Public functions have doc comments with a one-line description.
- `ARCHITECTURE.md` is updated whenever a new module is added.

---

## 14. What This Architecture Does NOT Include (and Why)

| Omission | Reason |
|----------|--------|
| Plugin system | premature. Add when you have 2+ use cases for plugins. |
| Async runtime | Synchronous `/proc` reads are fast enough. Async adds complexity without benefit. |
| Event bus / message broker | Overkill for a single-binary TUI app. Function calls are sufficient. |
| Database / persistence | MVP is in-memory only. Persistence is a Phase 4 concern. |
| Multi-threading | `/proc` reads are I/O-bound but fast. Single-threaded collection is sufficient. |
| Abstraction traits in core | premature. Add when you have 2+ implementations (e.g., real vs. mock enumerator). |
| Configuration system | premature. Add in Phase 3 when user preferences are needed. |
| Logging framework | premature. Add when debugging requires structured logs. |
