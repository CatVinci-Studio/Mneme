<p align="center">
  <img src="assets/mneme-logo.svg" alt="Mneme" width="128" height="128" />
</p>

<h1 align="center">Mneme</h1>

<p align="center">
  <strong>Save what you read. Let it grow into a wiki you own.</strong><br>
  A local-first read-it-later app with a native Rust agent and traceable Markdown knowledge base.
</p>

<p align="center">
  <a href="https://github.com/CatVinci-Studio/Mneme/releases/latest"><strong>Download</strong></a> ·
  <a href="./README.zh.md">中文</a>
</p>

<p align="center">
  <a href="https://github.com/CatVinci-Studio/Mneme/releases/latest"><img alt="version" src="https://img.shields.io/github/v/release/CatVinci-Studio/Mneme"></a>
  <img alt="platform" src="https://img.shields.io/badge/platform-macOS%20Apple%20Silicon-lightgrey">
  <a href="./LICENSE"><img alt="license" src="https://img.shields.io/badge/license-MIT-yellow"></a>
</p>

---

## What it is

Mneme (Μνήμη, Greek for “memory”) saves webpages, PDFs, and text as immutable local snapshots. Its Rust agent extracts sourced facts and continuously integrates them into a Markdown wiki: new facts are appended, refinements stay traceable, superseded facts move into history, and contradictions remain visible for review.

It is not just a reading queue. It turns what you save into a knowledge base that remains plain files, Git history, and yours to keep.

## Why

Traditional read-it-later apps often become a second inbox: content goes in and is rarely seen again. Hosted knowledge tools can also lock data into a database or disappear with the service.

Mneme is built around three constraints:

1. **Raw sources do not change.** A changed page becomes a new snapshot rather than overwriting old evidence.
2. **Generated facts point back to evidence.** Every fact stores a source id and exact character range.
3. **Files are the product.** The vault is readable Markdown with automatic Git history, not an opaque application database.

## Install

macOS (Apple Silicon) via [Homebrew](https://brew.sh):

```sh
brew install --cask catvinci-studio/tap/mneme
```

Or download `Mneme_X.Y.Z_aarch64.dmg` from [Releases](https://github.com/CatVinci-Studio/Mneme/releases/latest).

> Mneme is not Apple-notarized yet. If macOS blocks the first launch, right-click **Mneme.app** and choose **Open**, or run `xattr -cr /Applications/Mneme.app`.

## Quick start

1. Launch Mneme and open **Settings → AI service**.
2. Select OpenAI, DeepSeek, ChatGLM, Qwen, llama.cpp, or a custom OpenAI-compatible endpoint. The API key is stored in the operating-system credential store.
3. Click **Add**, paste a URL or text, and let Mneme create a note and Wiki entities.
4. Open a fact’s ↩ citation to jump back to the exact source excerpt.
5. Use **Search**, **Graph**, and **Research** to navigate and ask questions about the accumulated wiki.

The **Demo** provider runs entirely offline and is useful for trying the workflow without an API key.

## Features

- Webpage, PDF, and pasted-text ingest
- Immutable, content-addressed source snapshots
- AI notes, entity extraction, and fact-level Wikify
- Append / refine / supersede / contradiction semantics
- UTF-16 provenance ranges that match the WebView exactly
- Readable Markdown entity pages with YAML frontmatter
- Local Rust search, Research answers, graph, and health scan
- Automatic Git commits and optional private-remote backup
- System credential-store API keys
- Private-network URL blocking, pinned DNS resolution, redirects checked one by one, streaming size limits, and request timeouts
- Chinese and English UI, dark mode, responsive layout, and keyboard-accessible interactions

## Architecture

```text
React + TypeScript + Vite
          │ typed Tauri invoke
          ▼
Native Rust Agent Core
  ├── Ingest / Wikify / Research / Janitor
  ├── OpenAI-compatible providers
  ├── serialized Markdown Wiki Writer
  ├── local retrieval and graph
  ├── OS credential store
  └── Git versioning and backup
```

Node.js is used only to build the frontend. The installed app has no Node.js runtime, localhost HTTP API, CORS boundary, or sidecar process. See [DESIGN.md](DESIGN.md) and [AGENT_SPEC.md](AGENT_SPEC.md) for the data and Agent contracts.

## Data layout

```text
vault/
├── raw/<source_id>/content.md
├── raw/<source_id>/meta.json
├── raw/<source_id>/original.html
├── wiki/notes/<source_id>.md
├── wiki/entities/<slug>.md
└── log.md
```

The vault lives in the Tauri application-data directory and initializes its own Git repository. API keys never enter the vault or Git.

## Build from source

Requires Node.js 22+, stable Rust, and Xcode Command Line Tools on macOS.

```bash
git clone https://github.com/CatVinci-Studio/Mneme.git
cd Mneme/ui
npm install

npm run tauri dev      # native Rust app + Vite renderer
npm run tauri build    # unsigned .app and .dmg
npm run build          # TypeScript + Vite only

cd src-tauri
cargo test
cargo clippy --all-targets -- -D warnings
```

## Current scope

Version 0.2 targets macOS Apple Silicon. Native vector retrieval, background crash recovery, JavaScript-rendered page capture, signed/notarized releases, and Windows/Linux validation are tracked in [TASKS.md](TASKS.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for local checks, commit conventions, and the release process.

## License

[MIT](./LICENSE) © 2026 CatVinci Studio
