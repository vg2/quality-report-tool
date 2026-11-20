# 0002 — Encrypted SQLite via rusqlite + SQLCipher, keys in OS keychain

Date: 2026-11-20 · Status: Accepted

## Context
Default storage must be self-hosted, zero-ops and encrypted at rest, with a
path to Postgres / Jira / Azure DevOps later (`docs/plan.md` §10). The plan
originally named `sqlx`.

## Decision
- **`rusqlite` with `bundled-sqlcipher-vendored-openssl`**, not `sqlx`. Getting
  `sqlx` to use SQLCipher means overriding the `libsqlite3-sys` features it
  shares with other crates, which breaks easily; `rusqlite` supports it directly.
  A synchronous driver is adequate for a single-user desktop app; Tauri
  commands run off the main thread (`#[tauri::command(async)]`).
- **Layout:** an app-level `registry.db` lists workspaces; each workspace has
  its own `workspaces/<uuid>.db`. File paths derive only from UUIDs, never
  from user text.
- **Keys:** one random 256-bit key per database, stored hex-encoded in the OS
  keychain (`qrt-secrets`, `keyring` crate). SQLCipher's raw-key form is used
  (no PBKDF2 — the key is already full-entropy). Keys are zeroised on drop
  and redacted in `Debug`.
- **Hardening:** `secure_delete=ON`, WAL, foreign keys, owner-only file
  permissions (0600/0700) on Unix; forward-only migrations via
  `PRAGMA user_version`, refusing databases from newer app versions.
- **Abstraction:** `WorkspaceRegistry` and `SystemRepository` traits in
  `qrt-store`; SQLite is one implementation.

## Consequences
- Per-workspace keys allow a workspace to be exported/deleted independently
  and limit the blast radius of one key leaking.
- Losing the keychain entry makes that database unrecoverable. Export/backup
  with an explicit passphrase is needed before team use (Phase 2).
- SQLCipher + vendored OpenSSL increases first build time (~minutes).
- Bundled SQLite must track security releases (see RUSTSEC-2026-0459, fixed by
  `sqlcipher-src` 420 / SQLite 3.54.1, which `rusqlite` 0.43 pulls in).
