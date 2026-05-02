# Architecture

AI Ledger separates event capture from evaluation, reporting, and exporting.

## Paths

- Hot path: append minimal validated events locally.
- Control path: replay events into reports, exports, and gates.
- Batch path: run evaluation suites and benchmarks.

## Initial crates

- `ai-ledger-core`: shared domain types and errors.
- `ai-ledger-event`: event schema and validation.
- `ai-ledger-log`: append-only JSONL log.
- `ai-ledger-cli`: user-facing CLI.

Future crates are already present as placeholders so the workspace shape matches the starting specification.

## First useful loop

```bash
ai-ledger init
ai-ledger event append --run-id run_local
ai-ledger event replay
```
