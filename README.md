# Aster

A native Linux process monitor and manager, built in Rust.

Aster helps developers understand, monitor, and manage system processes with a fast, modern TUI interface.

## Project Structure

Aster is organized as a Cargo workspace with two crates:

- **`aster-core`** — Library crate containing all domain logic: process enumeration, system metrics, monitoring, and signal management.
- **`aster-ui`** — Binary crate providing the terminal user interface (TUI).

## Build & Run

```sh
cargo run -p aster-ui
```

## Documentation

- [VISION.md](VISION.md) — Project goals and philosophy.
- [ARCHITECTURE.md](ARCHITECTURE.md) — Architectural decisions and module structure.
- [ROADMAP.md](ROADMAP.md) — Planned features and milestones.

## License

MIT
