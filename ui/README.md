# Mneme UI

React 18 + TypeScript + Vite frontend hosted by Tauri. All runtime operations use typed `invoke` calls into the native Rust Agent Core.

## Development

```bash
npm install
npm run tauri dev
```

Running `npm run dev` alone starts only the Vite frontend; native features require the Tauri process.

## Checks

```bash
npm run build
cd src-tauri
cargo test
cargo clippy --all-targets -- -D warnings
```

## Unsigned build

```bash
npm run tauri build
```

The resulting app is intentionally unsigned until Apple signing/notarization is configured.
