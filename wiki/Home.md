# AI Ledger Wiki

AI Ledger is a Rust-native AI observability and assurance engine. The early project goal is to prove what changed, what was tested, and whether an AI release is safe enough to ship.

## Start here

- [Architecture](Architecture.md)
- [Privacy Model](Privacy-Model.md)
- [Event Log](Event-Log.md)
- [CLI Quickstart](CLI-Quickstart.md)
- [Roadmap](Roadmap.md)

## Current implementation status

Implemented:

- Cargo workspace skeleton.
- Core typed IDs, privacy modes, hash algorithm enum, and errors.
- Event envelope and initial event payload types.
- Append-only JSONL event log with replay.
- CLI commands for `init`, `event append`, and `event replay`.

Not implemented yet:

- Dataset/config manifests.
- YAML evaluation suite loader.
- OpenAI-compatible eval runner.
- JSON/Markdown report generator.
- Policy release gates.
- Splunk and OpenTelemetry exporters.
- Ingestion benchmark command.
