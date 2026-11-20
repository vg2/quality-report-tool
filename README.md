# Quality Report Tool

Desktop app that assesses a software system's quality against ISO/IEC 25010
and produces white-labelled reports. See [`context.md`](context.md) for the
goal and [`docs/plan.md`](docs/plan.md) for the solution plan.

## Prerequisites
- Rust (pinned by `rust-toolchain.toml`; install via [rustup](https://rustup.rs))
- Node 24 + pnpm 10
- Tauri CLI: `cargo install tauri-cli --version "^2" --locked`
- Linux only: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libdbus-1-dev`

## Develop
```bash
pnpm --dir ui install
cd src-tauri && cargo tauri dev
```

## Check (same as CI)
```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
pnpm --dir ui lint && pnpm --dir ui typecheck && pnpm --dir ui test && pnpm --dir ui audit
```

## Layout
| Path | Purpose |
|---|---|
| `crates/qrt-domain` | Entities, IDs, validated value types (no IO) |
| `crates/qrt-secrets` | OS keychain access, database key management |
| `crates/qrt-store` | Repository traits + SQLCipher-encrypted SQLite |
| `src-tauri` | Tauri shell: window, IPC commands, capabilities, CSP |
| `isolation` | Tauri isolation-pattern IPC allowlist |
| `ui` | React + TypeScript frontend |
| `docs/adr` | Architecture decision records |
| `docs/security` | Threat model |
