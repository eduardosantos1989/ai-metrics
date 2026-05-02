# CLI Quickstart

Run from the repository root during development:

```bash
cargo run -p ai-ledger-cli -- init
cargo run -p ai-ledger-cli -- event append --run-id run_local
cargo run -p ai-ledger-cli -- event replay
```

Append an event with tags:

```bash
cargo run -p ai-ledger-cli -- event append \
  --run-id run_local \
  --tag environment=dev \
  --tag service=rag-api
```

Append a specific LLM request payload:

```bash
cargo run -p ai-ledger-cli -- event append \
  --run-id run_local \
  --payload-json '{"status":"success","prompt_hash":"sha256:prompt","latency_ms":823}'
```
