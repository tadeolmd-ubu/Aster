# Aster-git

## Resumen General

| Tecnología |
| :--- |
| Rust |

| Tecnología | Archivo |
| :--- | :--- |
| Rust | lib.rs |
| Rust | main.rs |

## Información del Proyecto

| Campo | Valor |
| :--- | :--- |
| Versión | 0.1.0 |
| Edición | 2024 |

## Estructura del Proyecto

```
repository/
├── .gitignore
├── ARCHITECTURE.md
├── Cargo.lock
├── Cargo.toml
├── LICENSE
├── README.md
├── ROADMAP.md
├── VISION.md
├── aster-core
│   ├── Cargo.toml
│   └── src
│       ├── lib.rs
│       ├── monitor
│       │   ├── collector.rs
│       │   └── mod.rs
│       ├── process
│       │   ├── enumerator.rs
│       │   ├── mod.rs
│       │   ├── model.rs
│       │   └── snapshot.rs
│       ├── signal
│       │   ├── mod.rs
│       │   └── sender.rs
│       └── system
│           ├── cpu.rs
│           ├── memory.rs
│           └── mod.rs
├── aster-ui
│   ├── Cargo.toml
│   └── src
│       └── main.rs
└── docs
    ├── aster-core-src.md
    ├── aster-core.md
    ├── aster-ui-src.md
    ├── aster-ui.md
    ├── repository.md
    └── src.md
```

## Módulos

| Módulo | Ruta | Archivos |
| :--- | :--- | :--- |
| [aster-core](docs/aster-core.md) | `aster-core` | 1 |
| [src](docs/aster-core-src.md) | `aster-core/src` | 1 |
| [monitor](docs/monitor.md) | `aster-core/src/monitor` | 2 |
| [process](docs/process.md) | `aster-core/src/process` | 4 |
| [signal](docs/signal.md) | `aster-core/src/signal` | 2 |
| [system](docs/system.md) | `aster-core/src/system` | 3 |
| [aster-ui](docs/aster-ui.md) | `aster-ui` | 1 |
| [src](docs/aster-ui-src.md) | `aster-ui/src` | 1 |