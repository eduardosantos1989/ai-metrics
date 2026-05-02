# Architecture

Last updated: 2026-05-02

## Goal

AI Ledger records evidence about AI lifecycle activity with low application-visible overhead and privacy-first defaults. The first implementation slice focuses on append-only local event capture and replay.

## Runtime split

The design follows the starting specification's three-path model:

- Hot path: validate a minimal event, hash or omit sensitive fields, append locally, return.
- Control path: replay, export, report, and gate events after capture.
- Batch path: evaluation, manifest generation, report generation, and benchmark workflows.

## Workspace crates

| Crate | Current role |
|---|---|
| `ai-ledger-core` | Shared errors, typed IDs, timestamp utility, privacy modes, hash algorithm enum. |
| `ai-ledger-event` | Event envelope, event type enum, initial typed payloads, validation. |
| `ai-ledger-log` | Append-only JSONL event log and replay. |
| `ai-ledger-cli` | Initial CLI for `init`, `event append`, and `event replay`. |
| `ai-ledger-manifest` | Placeholder for deterministic dataset/config/model manifests. |
| `ai-ledger-eval` | Placeholder for YAML suite loading and OpenAI-compatible eval runs. |
| `ai-ledger-policy` | Placeholder for release gate policy evaluation. |
| `ai-ledger-report` | Placeholder for JSON and Markdown report generation. |
| `ai-ledger-export-splunk` | Placeholder for Splunk HEC export. |
| `ai-ledger-export-otel` | Placeholder for OpenTelemetry export. |
| `ai-ledger-bench` | Placeholder for ingestion benchmarks. |

## Event flow

```text
AI application or CLI
  -> EventEnvelope + typed payload
  -> validation
  -> JSONL append
  -> replay/export/report later
```

## Privacy position

The schema starts with hashes and operational metadata. Raw prompts, completions, RAG context, secrets, usernames, hostnames, and file paths are not captured by default.

## Current limitations

- No file rotation yet.
- No hashing utilities or manifests yet.
- No eval runner, report generator, gate engine, or exporters yet.
- CLI append accepts typed payload JSON but does not yet offer specialized subcommands per event type.
