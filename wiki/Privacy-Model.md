# Privacy Model

AI Ledger is privacy-first by default.

## Default

The default privacy mode is `hash_only`. Events should contain hashes and operational metadata instead of raw sensitive content.

## Modes

| Mode | Behavior |
|---|---|
| `hash_only` | Store hashes of sensitive fields and operational metadata. |
| `metadata_only` | Store operational metadata without content hashes. |
| `local_raw` | Future explicit opt-in for local raw content capture. |
| `off` | Disable event capture. |

## Sensitive data stance

Do not capture these by default:

- Raw prompts.
- Raw completions.
- Raw RAG documents.
- API keys or bearer tokens.
- Usernames, hostnames, customer names, or full file paths.
- Full dataset records.
