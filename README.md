# Aster-git

A native Linux process monitor and manager, built in Rust.

Aster helps developers understand, monitor, and manage system processes with a fast, modern TUI interface.

## Archivos
| Tecnología | Archivo |
| :--- | :--- |
| Rust | lib.rs |
| Rust | main.rs |

## Información del Proyecto
| Campo | Valor |
| :--- | :--- |
| Versión | 0.1.0 |
| Edición | 2024 |

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
