# 0003 — Desktop security baseline

Date: 2026-11-20 · Status: Accepted

## Context
`context.md` requires all typical desktop-app security measures. The webview
will render untrusted content (evidence, AI output), so a webview compromise
must not grant broad native access.

## Decision
- **Command capabilities:** every app command is registered in `build.rs`
  (`AppManifest::commands`), so Tauri only allows it if a capability grants
  it. `capabilities/main.json` grants only the app's own commands to the
  `main` window — no fs, shell, http or opener plugins.
- **Isolation pattern:** all IPC passes through `isolation/index.js`, which
  drops any command not on an allowlist (defence in depth).
- **CSP:** `default-src 'self'`, no inline/remote scripts, no `eval`,
  `object-src 'none'`, `base-uri 'none'`, `form-action 'none'`,
  `frame-ancestors 'none'`; `connect-src` limited to Tauri IPC. A separate
  `devCsp` allows only the local Vite dev server.
- **`freezePrototype: true`**, `withGlobalTauri: false`.
- **Navigation lock:** the main window is built in code with `on_navigation`
  allowing only app origins (plus the dev server in debug builds).
- **IPC inputs** are parsed into validated domain types (`Name`, UUIDs,
  length/character limits) before use; **errors** crossing IPC are generic
  codes + messages — no paths, SQL or secrets; details are logged in Rust.
- `unsafe_code = "forbid"` across the workspace; clippy `unwrap_used` /
  `expect_used` warnings.

## Consequences
- Adding a command needs four edits: the handler, `build.rs`, the capability
  file and the isolation allowlist. This friction is deliberate.
- Planned additions: sanitised Markdown rendering (DOMPurify) when AI/evidence
  text is shown, confirmation before opening external links, signed updater.
