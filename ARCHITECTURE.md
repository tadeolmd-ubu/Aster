# Architecture

## Overview

Aster uses a Cargo workspace with two crates:

```
aster-core  (library)  →  Business logic, workspace model, backend
aster-ui    (binary)   →  UI layer, depends on aster-core
```

The core crate holds all domain logic and is completely independent of any GUI framework.

## Crate split

| Crate       | Role                 | Depends on           |
|-------------|----------------------|----------------------|
| aster-core  | Business logic       | —                    |
| aster-ui    | User interface       | aster-core (path)    |
