# ai-ledger-rs

`ai-ledger-rs` is a Rust-native AI observability and assurance engine for recording AI lifecycle events, producing reproducibility manifests, running security/evaluation suites, and enforcing CI/CD release gates.

The project is intentionally starting with the ledger path: privacy-first event capture, append-only local storage, replay, and documentation. Exporters, evaluation runners, manifests, reports, and policy gates are represented as workspace crates and will be implemented incrementally.

## Current scope

- Cargo workspace with spec-aligned crates.
- Strongly typed core IDs, privacy modes, hash algorithm enum, and shared errors.
- Event envelope and initial payloads for:
  - `llm_request`
  - `dataset_manifest_created`
  - `eval_case_failed`
  - `release_gate_failed`
- Append-only JSONL event log with replay and malformed-record errors.
- `ai-ledger` CLI with:
  - `ai-ledger init`
  - `ai-ledger event append`
  - `ai-ledger event replay`

## Quick start

```bash
cargo run -p ai-ledger-cli -- init
cargo run -p ai-ledger-cli -- event append --run-id run_local
cargo run -p ai-ledger-cli -- event replay
```

Append a hashed LLM request event:

```bash
cargo run -p ai-ledger-cli -- event append \
  --run-id run_local \
  --payload-json '{"status":"success","model_hash":"sha256:model","prompt_hash":"sha256:prompt","latency_ms":823}'
```

## Privacy defaults

The default privacy mode is `hash_only`. The early event schema is designed to carry hashes and operational metadata, not raw prompts, completions, RAG documents, usernames, hostnames, file paths, or API keys.

Supported privacy modes:

- `hash_only`
- `metadata_only`
- `local_raw`
- `off`

## Development

```bash
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
```

## Documentation

- GitHub Wiki source pages: [wiki/Home.md](wiki/Home.md)
