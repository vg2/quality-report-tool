# 0001 — Tauri 2 + Rust core + React/TypeScript UI

Date: 2026-11-20 · Status: Accepted

## Context
The tool must be a high-performance, self-hostable, cross-platform desktop app
handling sensitive customer IP (`context.md`). It needs rich visualisation
(heatmaps, drill-downs) and local processing of repos, artefacts and reports.

## Decision
- **Tauri 2** shell with a **Rust** core organised as a cargo workspace of
  `qrt-*` crates (`docs/plan.md` §1). Business logic lives in crates with no
  Tauri dependency, so it is testable and reusable (e.g. a future CLI/server).
- **React + TypeScript** UI (Vite) — largest ecosystem for charts and
  component libraries.
- `src-tauri` is a thin layer: window setup, IPC commands, wiring.

## Consequences
- Small binaries and a strong permission model compared with Electron.
- Platform webviews differ (WebKit on macOS/Linux, WebView2 on Windows), so UI
  must be tested on all three; CI builds each platform.
- Rust toolchain pinned in `rust-toolchain.toml`.
