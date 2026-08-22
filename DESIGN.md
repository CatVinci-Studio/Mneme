# Mneme Architecture

## Product

Mneme is a local-first read-it-later application that turns saved material into a continuously growing Markdown wiki. Raw sources remain immutable; every generated fact carries a source id and character range.

## Runtime

```text
Tauri desktop process
├── React/Vite WebView
└── Native Rust Agent Core
    ├── Tauri commands
    ├── Ingest / Wikify / Research / Janitor
    ├── OpenAI-compatible LLM client
    ├── Rust lexical retrieval
    ├── Markdown Vault / serialized Wiki Writer
    └── Git versioning and backup
```

Node.js is a frontend build tool only. The installed application has no Node.js runtime, local HTTP server, CORS boundary, or sidecar.

## Data model

```text
vault/
├── .git/
├── .gitignore
├── log.md
├── raw/<source_id>/
│   ├── content.md
│   ├── meta.json
│   └── original.html       # web sources only
└── wiki/
    ├── notes/<source_id>.md
    └── entities/<slug>.md
```

- A source id includes the normalized content hash, so changed content creates a new immutable snapshot.
- Entity pages are readable Markdown with YAML frontmatter, fact anchors, and provenance footnotes.
- API keys live in the operating-system credential store and never enter the vault or Git.

## Agent pipeline

### Ingest

1. Validate text or URL input.
2. Reject private, loopback, link-local, and unsupported URL targets.
3. Fetch with timeout, redirect limit, HTTP status checks, and a 10 MB cap.
4. Extract text from HTML or PDF.
5. Write the immutable raw snapshot.
6. Ask the configured model for a note and candidate entities.

### Wikify

1. **Extract** atomic claims with exact source spans.
2. **Locate** existing pages by slug, title, and aliases.
3. **Reconcile** each claim as append, refinement, supersession, contradiction, or deduplication.
4. **Commit** through the serialized Writer using an optimistic base hash.
5. **Cross-link** entities that co-occur in one source.

The Writer verifies every UTF-16 source range before writing. Superseded facts move to History; contradictions remain visible; information is never silently deleted.

### Research

Rust retrieval ranks title, summary, and fact text. The selected pages and fact-level provenance are supplied to the model, which must answer with wiki and source citations.

### Janitor

A deterministic health scan reports empty pages, orphan pages, contradictions, source counts, and fact counts.

## Frontend boundary

The frontend communicates only through typed Tauri `invoke` commands:

- configuration and key status
- source ingest/list/read/Wikify
- entity list/read/search/graph
- Research and health scan
- Git backup

The Rust errors are normalized into JavaScript `Error` objects by `ui/src/api.ts`.

## Providers

One Rust OpenAI-compatible client supports OpenAI, DeepSeek, ChatGLM, Qwen, llama.cpp, and custom endpoints. Structured tasks request JSON objects; the deterministic mock provider keeps development and tests offline.

## Release

Tauri compiles the Rust Agent Core into the application itself. No sidecar packaging is required. Unsigned local builds are produced with `npm run tauri build`; Apple signing and notarization are intentionally deferred.
