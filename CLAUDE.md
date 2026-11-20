# Quality Report Tool

Desktop app that assesses another software system's quality against ISO/IEC 25010, weighted by the type of system, using evidence referenced from enterprise sources. It produces white-labelled PDF reports and presentations.

## Read first
- `context.md` — original goal, vision and security concerns (source of truth for intent)
- `docs/plan.md` — approved solution plan: architecture, domain model, scoring, security, phases

## Confirmed decisions
- **Stack**: Tauri 2 + Rust core (cargo workspace of `qrt-*` crates), React + TypeScript frontend
- **Storage**: one encrypted SQLite (SQLCipher) DB per workspace, behind repository traits so Postgres / Jira / Azure DevOps can be added later
- **Users**: single user first; share by exporting/importing workspaces; team sync later
- **AI**: AI-first assessment. The AI proposes measure values and practice maturity with verified citations, a human reviews them, and grades are always calculated deterministically
- **Sources**: read-only and copied first. Every source (local folder, git repo, pipeline artefact, doc) is copied into a temporary per-run sandbox and analysed only there. Originals are never modified, copies are purged after the run, and only references and hashes are stored
- **Reports**: Typst for PDF; PPTX generated from a white-label master template, plus a PDF deck

## Non-negotiables
- Customer IP is pseudonymised and secrets are stripped before any AI call; per-workspace provider policy (incl. local-only)
- All evidence is untrusted: the dual-LLM pattern (the model reading raw evidence has no tools and must return schema-checked output), verified citations, and no side-effecting AI tools
- Least-privilege Tauri capabilities, strict CSP, secrets only in the OS keychain, no execution of source/repo code

## Current status
- **Phase 0 (foundations) complete** (2026-11-20): cargo workspace (`qrt-domain`, `qrt-secrets`, `qrt-store`), Tauri 2 shell + React UI (workspaces & systems CRUD only), SQLCipher store with keychain keys, capabilities/CSP/isolation baseline, CI + cargo-deny + SBOM + Dependabot, ADRs 0001–0004, threat model v1.
- CI workflow is written but has not run yet: there is no GitHub remote.
- **Next: Phase 1** per `docs/plan.md` §11, starting with step 1 (ISO 25010:2023 catalog + profile rules + profile editor).

## Conventions
- Run the checks in `README.md` before committing; CI runs the same.
- Adding an IPC command needs four edits: handler in `src-tauri/src/commands.rs`, `COMMANDS` in `src-tauri/build.rs`, `capabilities/main.json`, and the allowlist in `isolation/index.js` (ADR 0003).
- Storage changes go through repository traits in `qrt-store`; schema changes are new numbered files in `crates/qrt-store/src/migrations/`.
- Record significant decisions as a new ADR in `docs/adr/`; update `docs/security/threat-model.md` when adding a trust boundary.
- No `unwrap`/`expect` outside tests; `unsafe` is forbidden.

Keep this "Current status" section updated as work progresses so new sessions know where to pick up.
