# Mneme Rust Agent Specification

## Invariants

1. Raw source content is immutable.
2. A model-generated claim is writable only when its exact source span validates.
3. All entity writes pass through one serialized Writer.
4. Optimistic hashes reject stale page updates.
5. Updates preserve history; contradictions preserve both sides.
6. Markdown files are the source of truth and remain readable without Mneme.

## Roles

| Role | Trigger | Responsibility |
|---|---|---|
| Ingest | new source | normalize input, snapshot content, create note and entity candidates |
| Wikify | ingest/manual | extract claims, locate pages, reconcile, validate, commit and cross-link |
| Research | user question | retrieve wiki evidence and answer with citations |
| Janitor | manual scan | report structural health issues |

## Structured model operations

### Note

```json
{
  "tldr": "string",
  "key_points": ["string"],
  "candidate_entities": [{"name":"string","kind":"person|org|concept|tech|topic|event|place","why":"string"}],
  "tags": ["string"]
}
```

### Extract

```json
{
  "entities": [{"name":"string","kind":"string","aliases":["string"]}],
  "claims": [{"entity_name":"string","text":"string","span":"exact source substring","confidence":0.9}]
}
```

`span` must be an exact contiguous source substring. Rust computes and stores UTF-16 offsets so the WebView can navigate to the same range.

### Reconcile

Possible decisions:

- `create_page`
- `append_fact`
- `update_fact`
- `supersede_fact`
- `flag_contradiction`
- no operation for deduplication

Every fact-changing operation references a known claim id. Cross-page operations may target another entity slug.

## Writer

For each page commit:

1. Acquire the process-wide Writer mutex.
2. Compare the current Markdown hash with the proposal base hash.
3. Validate every claim against the immutable source UTF-16 range.
4. Apply typed operations to the in-memory page.
5. Serialize YAML frontmatter, Markdown sections, anchors, and provenance footnotes.
6. Commit the vault change to Git on a best-effort basis.

## Retrieval

The current native retriever is deterministic and local. It matches slugs/titles/aliases for entity location and ranks title, summary, and facts for search. The interface is intentionally isolated so a future Rust vector index can replace ranking without changing Agent or UI contracts.

## Security

- URL ingest accepts only HTTP(S).
- DNS results are checked before fetching; private and local addresses are rejected.
- Fetches use bounded redirects, a timeout, status validation, and a size limit.
- API keys are excluded from the vault and returned to the UI only as `hasKey`.
- File paths supplied by the UI are validated as single path components.
