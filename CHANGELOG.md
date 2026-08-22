# Changelog

## 0.2.0 — 2026-08-22

- Replaced the Node.js Agent Core and HTTP transport with a native Rust/Tauri implementation.
- Added native ingest, immutable source snapshots, Wikify reconciliation, provenance validation, Markdown Wiki storage, search, Research, graph, lint, and Git backup.
- Added OpenAI-compatible providers for OpenAI, DeepSeek, ChatGLM, Qwen, llama.cpp, and custom endpoints.
- Added private-network URL blocking, response limits, request timeouts, and separate protected API-key storage.
- Replaced frontend HTTP calls with typed Tauri IPC commands.
- Improved loading/error feedback, global search, Markdown rendering, citation navigation, accessibility, and persisted appearance settings.
- Removed all backend Node.js/Bun runtime dependencies and sidecars.

## 0.1.0

- Initial TypeScript prototype validating the ingest → Wikify → Markdown Wiki workflow.
