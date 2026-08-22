# Mneme 0.2 Status

## Completed

- [x] Native Rust Agent Core embedded in Tauri
- [x] Typed Tauri IPC; no HTTP backend or sidecar
- [x] Text, static HTML, and PDF ingest
- [x] Immutable content-addressed source snapshots
- [x] OpenAI-compatible provider presets and offline mock provider
- [x] Note and claim extraction
- [x] Entity locate, reconcile, provenance validation, optimistic locking, and serialized writes
- [x] History preservation, contradiction flags, and co-occurrence links
- [x] Markdown entity serialization and parsing
- [x] UTF-16 citation navigation back to source excerpts
- [x] Native lexical search, Research, graph, and health scan
- [x] Automatic vault Git commits and optional remote backup
- [x] API keys stored in the operating-system credential store
- [x] URL scheme, private-network, timeout, redirect, status, and response-size protection
- [x] React UI with global search, loading/error feedback, Markdown rendering, themes, i18n, and responsive layout
- [x] Rust unit and end-to-end mock pipeline tests
- [x] Frontend production build and dependency audit

## Deferred after 0.2

- [ ] Native vector/semantic retrieval in Rust
- [ ] Persistent background job queue and crash recovery
- [ ] Browser extension and JavaScript-rendered page capture
- [ ] System Keychain integration on macOS/Windows Credential Manager
- [ ] Signed and notarized Apple release
- [ ] Windows and Linux release validation
- [ ] Automatic updater and release channels
