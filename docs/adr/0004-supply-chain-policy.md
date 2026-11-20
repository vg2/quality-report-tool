# 0004 — Supply-chain policy

Date: 2026-11-20 · Status: Accepted

## Decision
- `cargo deny check` in CI (`deny.toml`): known vulnerabilities fail; licences
  restricted to a permissive allowlist (crate-scoped exceptions only, e.g.
  SQLCipher's SQLite "blessing" + libtomcrypt WTFPL); crates.io only; no
  wildcard versions.
- `pnpm audit` (all deps) and `--frozen-lockfile` installs in CI.
- CycloneDX SBOMs for Rust and npm generated on every CI run.
- Dependabot weekly for cargo, npm and GitHub Actions.
- Rust toolchain pinned (`rust-toolchain.toml`); pnpm pinned via
  `packageManager`.
- Unmaintained crates are replaced rather than ignored (e.g. `chrono` →
  `jiff`, RUSTSEC-2026-0543).

## Consequences / follow-ups
- GitHub Actions are pinned by major tag for now; pin to commit SHAs before
  the first signed release.
- Release workflow (Phase 1) adds code signing, notarisation and signed
  updater artefacts.
