# Contributing to Mneme

Mneme is a Tauri app with a React/TypeScript renderer in `ui/src` and a native Rust Agent Core in `ui/src-tauri/src`.

## Setup

```sh
cd ui
npm install
npm run tauri dev
```

`npm run dev` starts only the renderer. Source ingest, settings, Wikify, search, and all other product features require the Tauri process.

## Before you push

Run the same checks as CI:

```sh
cd ui
npm run build
npm audit --audit-level=high

cd src-tauri
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Changes to persistence, provenance, provider wire formats, URL fetching, or Git backup require a regression test.

## Commit style

Use Conventional Commits, optionally with an area:

```text
feat(agent): preserve aliases during entity creation
fix(vault): atomically persist entity pages
security(fetch): pin validated DNS addresses
```

Comments should record invariants and failure modes rather than repeat the code. Rust and TypeScript representations of the same IPC contract should be changed together.

## Data-safety rules

- Never mutate an existing raw source snapshot.
- Never write a claim without validating its exact source range.
- Never bypass the serialized Wiki Writer.
- Never silently remove a fact; move it to History or record a contradiction.
- Never put API keys in the vault, logs, fixtures, screenshots, or repository.

## Releases

1. Update `ui/package.json`, `ui/src-tauri/Cargo.toml`, and `ui/src-tauri/tauri.conf.json` to the same version.
2. Update `CHANGELOG.md` and let Cargo update `Cargo.lock`.
3. Run all checks and build the unsigned DMG with `npm run tauri build -- --bundles app,dmg`.
4. Commit, tag `vX.Y.Z`, and push the tag.
5. The release workflow publishes the DMG as a GitHub pre-release until signing/notarization is enabled.
6. Update `CatVinci-Studio/homebrew-tap/Casks/mneme.rb` with the release DMG SHA-256.
