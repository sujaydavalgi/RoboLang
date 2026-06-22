# spanda-typecheck

Physical unit algebra (`units`) and compile-time type name resolution (`type_system`) extracted for the Phase 4 compiler split.

The full program type checker (`TypeChecker` in `spanda-core::types`) remains in core for now; it depends on this crate for primitives.
