# spanda-typecheck

Physical unit algebra, compile-time type name resolution, message/module registries, and the full program `TypeChecker`.

Domain-specific validation (libraries, SoC, security capabilities, reliability) is injected through [`TypeCheckHost`](src/host.rs); `spanda-core` supplies [`CoreTypeCheckHost`](../spanda-core/src/type_check_host.rs).
