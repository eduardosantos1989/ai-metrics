# Known failures

## Initial implementation absent
Status: resolved
Area: repository skeleton
First seen: 2026-05-02

### Symptom
- Repository contains only a minimal hello-world Rust package at task start.

### Evidence
- src/main.rs contained only `println!("Hello, world!");`.
- Cargo.toml defined a single package named `ai-metrics` with no dependencies.
- Repository now has a Cargo workspace and `cargo test --workspace` passes.

### Suspected cause
- Repository was initialized before applying the ai-ledger starting specification.

### Disproven causes
- None yet.

### Next diagnostic step
- Continue with deterministic hashing and manifest implementation.
