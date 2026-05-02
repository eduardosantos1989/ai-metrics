# Event Log

The first event log format is append-only JSONL.

## Envelope

Each line is one event envelope with:

- `event_id`
- `event_type`
- `schema_version`
- `timestamp_ns`
- `run_id`
- `privacy_mode`
- `payload`
- `tags`

## Initial event types

- `llm_request`
- `dataset_manifest_created`
- `eval_case_failed`
- `release_gate_failed`

## Replay

`ai-ledger event replay` reads the JSONL file, validates each event, and writes JSONL to stdout by default. `--pretty` prints a JSON array for inspection.

Malformed records return an error that includes the failing line number.
