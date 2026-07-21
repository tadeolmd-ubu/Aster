# Vision

Aster is a native Linux process monitor and manager for developers.

## Goals

- **Fast** — Rust-native, no Electron, no JVM. Starts in milliseconds.
- **Informative** — Show what matters: process state, resource usage, relationships, and history.
- **Interactive** — Not just monitoring. Send signals, manage processes, inspect details.
- **Developer-friendly** — Designed for people who live in the terminal and need quick, reliable process insights.
- **Reliable** — Built on direct `/proc` reads, no external daemons or agents.

## Non-goles

- Not a replacement for `htop` or `btop` — Aster focuses on developer workflow, not system administration dashboards.
- Not cross-platform — Linux only. No compromises.
- Not a container orchestrator — Processes, not pods.
