# Quality Report Tool — Solution Plan

## Context
Solutions architects and tech leads need an evidence-based way to report the state of software quality to stakeholders: what is good (with proof), what is poor (with proof), and where there are gaps that make quality unknowable. They also need prioritised improvement recommendations. Today this is manual and inconsistent. We will build a self-hosted, cross-platform desktop app that assesses a target system against **ISO/IEC 25010**, weighted by the type of system, using evidence referenced from enterprise sources (repos, pipelines, wikis), and produces white-labelled PDF reports and presentations.

The project folder is greenfield (only `context.md`). Decisions confirmed:
- **Tauri 2 + Rust core, React + TypeScript frontend**
- **SQLite per workspace** (encrypted), behind a storage trait for Postgres / Jira / Azure DevOps later
- **Single user first**; export/import to share, team sync later
- **AI-first assessment**: AI analyses evidence and proposes scores; human reviews and approves

---

## 1. Architecture

```
┌──────────────── React/TS UI (Tauri webview, strict CSP, no remote content) ───────────────┐
│ Workspaces · System Profile · Sources · Evidence · Assessment Review · Heatmap · Reports  │
└───────────────────────────── typed IPC (tauri commands, allowlisted) ──────────────────────┘
┌───────────────────────────────────── Rust core (workspace crates) ─────────────────────────┐
│ qrt-domain     entities, IDs, value types (no IO)                                          │
│ qrt-catalog    versioned ISO 25010 model + measures/practices catalog (YAML, signed)       │
│ qrt-profile    system profile dimensions + weighting rules                                 │
│ qrt-scoring    deterministic aggregation, grades, confidence, heatmap, prioritisation      │
│ qrt-store      Repository traits + SQLCipher impl (rusqlite; ADR 0002)                    │
│ qrt-sources    connector trait + git, CI pipelines, test/coverage/perf parsers, docs/wiki  │
│ qrt-redact     classification, pseudonymisation, secret scanning, generalisation           │
│ qrt-ai         provider trait, agent orchestration, prompt-injection defences, validation  │
│ qrt-report     Typst PDF report, PPTX + PDF deck, white-label theming                      │
│ qrt-audit      append-only audit/egress log                                                │
│ app (src-tauri) commands, capabilities, keychain, updater                                  │
└────────────────────────────────────────────────────────────────────────────────────────────┘
```

### Proposed repo layout
```
quality-report-tool/
  Cargo.toml                 # workspace
  crates/qrt-*/              # crates above
  src-tauri/                 # Tauri app shell, capabilities/*.json, tauri.conf.json
  ui/                        # React + TS (Vite, shadcn/ui, TanStack Query, ECharts for heatmap)
  catalog/iso25010-2023/     # characteristics, sub-chars, measures, practices (YAML)
  catalog/profiles/          # profile dimension → weight rules
  templates/report/          # Typst report templates
  templates/deck/            # PPTX master + Typst deck template
  docs/adr/                  # architecture decision records
```

---

## 2. Domain model (core entities)

| Entity | Purpose |
|---|---|
| **Workspace** | One encrypted SQLite DB; holds branding + one or more assessed systems |
| **Branding** | Assessor identity, client identity, or single identity (self-assessment): logos, colours, fonts, disclaimer |
| **System** | The software being assessed |
| **SystemProfile** | Structured characteristics (below); source = manual / AI-inferred (confirmed) |
| **Source** | Reference to external system (repo URL, pipeline, wiki page) + credentials ref (keychain) |
| **Evidence** | Pointer to source artefact (URL, commit SHA, run ID, path, hash, retrieved-at) + extracted facts. Raw content only cached temporarily |
| **Assessment** | Point-in-time snapshot (catalog version, profile, evidence set) — enables trends |
| **MeasureResult / PracticeResult** | Value or maturity level, rationale, citations → Evidence, proposed-by (AI/human), review status |
| **Finding / Recommendation** | Strength, weakness, gap; recommendation with impact/effort/priority |

Principle from vision: **store quality information + references, not copies**. Evidence stores pointers, hashes and extracted facts; cloned repos and copies of local folders live in a temp sandbox and are purged after analysis — sources themselves are never worked on directly. Gaps are filled with manually entered evidence only when no source exists.

---

## 3. ISO/IEC 25010 catalog

Versioned, data-driven catalog (YAML, bundled and signature-verified; user overrides layered on top). Default **25010:2023**, with 2011 mapping retained for clients still using it.

Characteristics (2023): Functional Suitability · Performance Efficiency · Compatibility · Interaction Capability · Reliability · Security · Maintainability · Flexibility · Safety — each with its sub-characteristics (e.g. Performance → Time behaviour, Resource utilisation, Capacity; Reliability → Faultlessness, Availability, Fault tolerance, Recoverability).

Note: ISO text is copyrighted — catalog uses characteristic names with our own descriptions.

Each sub-characteristic has:
- **Measures** — quantitative, with unit, direction, thresholds per profile (e.g. p95 response time, availability %, MTTR, change failure rate, line/branch coverage, critical vulns open, cyclomatic complexity, WCAG violations), accepted evidence types and extractor.
- **Practices** — qualitative with maturity levels 0–4 (e.g. "load tests run in CI with SLO gates", "threat modelling per release", "DR tested", "a11y testing automated").
- **Applicability rules** — when the item applies based on profile (e.g. inclusivity weight high for customer-facing; safety N/A for internal reporting unless safety-critical).

---

## 4. System profile & weighting

Profile dimensions (multi-select where sensible):
1. **Audience** — customer-facing / employee-facing / partner / machine-to-machine
2. **Interaction style** — transactional / reporting & analytics / batch-backend data processing / streaming / API platform
3. **Business criticality** — low / medium / high / mission-critical
4. **Data sensitivity** — public / internal / PII / financial / health
5. **Regulatory** — none / GDPR / PCI-DSS / HIPAA / SOX / sector-specific
6. **Deployment** — SaaS-cloud / on-prem / hybrid / mobile / desktop / embedded
7. **Scale & lifecycle** — users/volume tier; greenfield / active / maintenance / legacy
8. **Safety relevance** — none / indirect / safety-critical

Capture: form, or natural-language description → AI infers profile with per-field rationale → user confirms/edits (never auto-applied).

Weighting: rules map profile → importance per sub-characteristic (0 N/A, 1 low, 2 medium, 3 high, 4 critical), additive/max rules in `catalog/profiles/*.yaml`, user override with justification recorded.

---

## 5. Scoring model (deterministic & explainable)

AI proposes measure values and practice maturity; **aggregation is always deterministic** so grades are reproducible and defensible.

Per sub-characteristic:
- **Quality score Q (0–100)** — measures scored against profile thresholds (linear between "poor" and "target"), practices by maturity; weighted mean.
- **Confidence C (0–100)** — evidence coverage (applicable items evidenced), freshness decay, source reliability (pipeline artefact > doc > assertion), human-reviewed vs AI-only.
- **Importance I** — from profile.

Three-state classification surfaced everywhere:
- **Evidenced good** (high Q, high C) · **Evidenced poor** (low Q, high C) · **Gap** (low C — quality not assessable)

Roll-up: characteristic and overall scores = importance-weighted means; **Grade A–E** on Q with **confidence band** (e.g. "B, low confidence"). A characteristic with critical importance and grade E caps overall grade (configurable).

**Heatmap**: rows = characteristics, columns = sub-characteristics; cell colour = risk `I × (1 − Q)` with hatching/opacity for low confidence; toggle to "gap view" `I × (1 − C)`. Drill-down to measures → evidence → source link.

**Recommendation prioritisation**: `priority = I × (expected ΔQ + expected ΔC) / effort`, where catalog practices declare typical uplift and effort (S/M/L). AI tailors wording and specifics; ranking stays formulaic. Output a top-N roadmap (quick wins vs strategic).

---

## 6. Sources & evidence collection

`Connector` trait: `discover() → artefacts`, `fetch(ref) → temp content`, `extract() → facts`, `link(ref) → URL`.

**Read-only, copy-first rule (all sources):** the app never analyses or modifies a source in place. Every source — remote or local — is first materialised as a **temporary working copy** in the per-assessment sandbox (`<app-temp>/runs/<assessment-id>/<source-id>/`), and all analysers/AI work against that copy only. Original sources are opened read-only; nothing is written back. Working copies are purged after the run (and on startup if a run crashed); evidence keeps only references + hashes.

Phase 1 connectors:
- **Local folder / file** — user picks a folder (or files) on this machine via the native dialog; the path is granted to the app's filesystem scope read-only. Snapshot-copied into the sandbox with: symlinks not followed outside the selected root, size/file-count limits, include/exclude globs (default excludes `node_modules`, `target`, `.venv`, build output), and a content-hash manifest so evidence can cite exact file versions and later re-runs can detect changes. If the folder is a git repo, the git analysers below also apply to the copy.
- **Git** (local path or HTTPS/SSH clone via `gix`) — shallow clone to sandbox temp dir; hooks disabled, no LFS smudge, no submodule recursion by default, size/file-count limits; **never executes repo code**. Deterministic analysers: languages/LOC, test file ratio, dependency manifests → OSV vulnerability lookup, presence of CI config, Dockerfiles, IaC, lint configs, ADRs, README/CONTRIBUTING, CODEOWNERS, commit/PR cadence.
- **CI pipelines** — GitHub Actions, Azure DevOps, GitLab: run history (success rate, duration, DORA proxies), test artefacts.
- **Artefact parsers** — JUnit/xUnit/TRX, Cobertura/LCOV/JaCoCo, SARIF (static analysis/security), k6/JMeter/Gatling/Locust (performance), Lighthouse/axe (a11y), OpenAPI specs.
- **Docs/wiki** — URL (Confluence, ADO wiki, Markdown, PDF upload): text extraction for AI review.
- **Manual evidence** — form entry + file attachment when no source exists.

All extracted facts carry provenance (source, ref, hash, timestamp). Phase 2 adds scheduled/webhook live feeds.

---

## 7. AI-first assessment engine (`qrt-ai`)

### Providers (pluggable, company-chosen)
`InferenceProvider` trait with: Anthropic API, AWS Bedrock, Azure OpenAI, OpenAI-compatible (incl. local Ollama/vLLM), and **CLI providers** (e.g. Claude Code non-interactive mode) run with tools disabled in an empty isolated working dir. Per-workspace provider policy; credentials in OS keychain.

### Pipeline
1. **Collect** — connectors + deterministic analysers produce facts and documents.
2. **Redact** — `qrt-redact` (section 8) before anything leaves the machine.
3. **Quarantined extraction** (per characteristic agent) — LLM reads untrusted evidence and returns **schema-constrained JSON only**: candidate measure values, practice maturity, findings, each with citations (evidence ID + quoted span).
4. **Validation** — JSON schema check; every citation verified to exist in the evidence (verbatim span match); values range-checked; uncited claims dropped and flagged.
5. **Synthesis** (privileged LLM) — sees only validated structured outputs (never raw evidence) to draft narrative: executive summary, characteristic summaries, tailored recommendations.
6. **Score** — deterministic engine (section 5).
7. **Human review** — review queue: accept / edit / reject each AI proposal; diff vs previous assessment; nothing reaches a report as "reviewed" without approval (reports can be issued as draft with AI-only items labelled).

Prompts and model IDs versioned and stored with each assessment for reproducibility. Evaluation harness with golden sample systems to regression-test prompt/model changes.

---

## 8. Security design

### Desktop app hardening
- Tauri 2 **capabilities** least-privilege; only explicitly allowlisted commands; no shell plugin exposed to UI; filesystem scoped to workspace + temp dirs, plus read-only access to user-selected source folders.
- Strict **CSP**, no remote scripts, no `eval`, Tauri isolation pattern; all AI/evidence text rendered as sanitised Markdown (DOMPurify), external links open in system browser after confirmation.
- IPC inputs validated with typed structs (serde + validator); errors don't leak paths/secrets.
- **Encryption at rest**: SQLCipher DB; key held in OS keychain (`keyring` crate: macOS Keychain, Windows Credential Manager, Secret Service). Attachments encrypted. Temp clones purged after run and at startup.
- Secrets (PATs, API keys) only in keychain, never in DB/logs; logs redact via tracing layer.
- Network: rustls, cert validation, optional corporate proxy & custom CA; egress allowlist per workspace.
- Untrusted file handling: size limits, parser fuzzing (`cargo-fuzz`) for artefact parsers, zip-bomb/path-traversal protection, PDF parsing in isolated process with timeout.
- Supply chain: `cargo-deny`, `cargo-audit`, `npm audit`/lockfiles, SBOM (CycloneDX) per release, pinned toolchains, reproducible CI.
- Signed & notarised builds (macOS notarisation, Windows Authenticode), signed auto-updater; catalog files signature-verified.
- Append-only **audit log** (who/what/when, including every AI egress).
- Threat model (STRIDE) maintained in `docs/security/`.

### Protecting customer IP from inference providers
- **Data classification per workspace** → policy: `local-only` (local model only), `approved-cloud` (zero-retention / no-training providers: Bedrock, Anthropic commercial API, Azure OpenAI), etc. Policy enforced in `qrt-ai`, not UI.
- **Pseudonymisation**: client/company names, people, emails, hostnames, IPs, URLs, repo/project names, account IDs replaced with stable tokens (`ORG_1`, `HOST_3`); mapping stays local and is re-hydrated on responses.
- **Secret scanning** (gitleaks-style rules + entropy) — secrets removed, never sent.
- **Generalisation / minimisation**: prefer sending derived metrics and summaries over raw code; code snippets capped and only where needed; file paths generalised.
- **Egress preview**: user can inspect exactly what will be sent; full egress logged.

### Prompt-injection defence
- All evidence treated as untrusted; wrapped with random-nonce delimiters + datamarking ("spotlighting"); system prompt states content is data only.
- **Dual-LLM pattern**: quarantined model touches untrusted content but has **no tools** and can only emit schema-valid JSON; privileged model never sees raw evidence.
- No side-effecting tools anywhere in the AI layer; agents' read tools scoped to the evidence store of the current assessment.
- Output validation: schema, citation verification, bounds checks, heuristic injection detector (flags phrases like instructions to the model, unusual score swings) → flagged for human review.
- AI output never executed or used to build URLs/commands; rendered sanitised.
- Red-team test corpus of injected evidence files in CI.

---

## 9. Reporting & presentation (`qrt-report`)

- **PDF report**: **Typst** (embedded Rust crate — fast, offline, high-quality typography). Templates: cover, executive summary + overall grade, heatmap, per-characteristic sections (score, confidence, findings, evidence references with links), gaps, prioritised roadmap, methodology appendix (ISO mapping, scoring formula, profile, catalog version).
- **Presentation**: **PPTX** (editable, stakeholder-friendly) generated in Rust by filling a white-label master template (OOXML via `zip` + XML templating; charts as native or SVG→PNG), plus **PDF deck** via Typst slide template.
- **White labelling**: Branding entity drives both outputs — assessor + client logos/colours/fonts, or single identity for self-assessment; custom template upload (validated).
- Charts rendered once (server-side SVG via `plotters`/Typst) to keep UI, PDF and deck consistent.

---

## 10. Extensibility

- `Repository` traits in `qrt-store` (Systems, Assessments, Evidence, …) — SQLite now; Postgres next; **Jira / Azure DevOps** backends later mapping findings/recommendations to work items (sync, not source of truth initially).
- `Connector`, `InferenceProvider`, `ReportRenderer` traits; registration via config. (WASM plugin sandbox considered for third-party connectors in a later phase.)
- Export/import of a workspace as an encrypted, signed bundle for sharing (single-user model).

---

## 11. Delivery phases

**Phase 0 — Foundations (≈2–3 wks)**: cargo workspace, Tauri 2 + React shell, CI (lint, test, audit, SBOM), SQLCipher store + migrations, keychain integration, capabilities/CSP baseline, ADRs, threat model v1.

**Phase 1 — MVP (≈10–12 wks)**
1. Catalog 25010:2023 + profile rules + manual profile editor
2. Scoring engine + heatmap + drill-down UI
3. Git, GitHub Actions/ADO pipeline connectors; JUnit, coverage, SARIF, k6/JMeter parsers; manual evidence
4. `qrt-redact` + provider abstraction (Anthropic, Bedrock, OpenAI-compatible/Ollama, CLI)
5. AI pipeline: profile inference, quarantined extraction, validation, synthesis, review queue
6. Typst PDF report + PPTX deck with white labelling
7. Audit log, signed builds, injection red-team suite

**Phase 2**: live feeds (scheduled pulls, webhooks), assessment trends over time, more connectors (GitLab, SonarQube, Snyk, Lighthouse, APM like Datadog/App Insights for real availability/latency), Postgres backend, team sharing/sync.

**Phase 3**: Jira / Azure DevOps as storage/tracking backends, multi-user roles, plugin SDK.

---

## 12. Verification

- **Unit**: scoring formulas (property tests with `proptest` — monotonicity, bounds, N/A handling), profile weighting rules, redaction (golden inputs → no PII/secret leakage), parsers (fixture files + fuzzing).
- **Integration**: end-to-end assessment of 2–3 bundled sample systems (e.g. a small open-source web app with tests/CI) against a mock inference provider returning recorded responses; assert grades, findings and citations.
- **AI eval harness**: golden systems with expected measure ranges; run per prompt/model change; injection corpus must yield zero unflagged score changes.
- **Security**: `cargo-audit`/`cargo-deny`/`npm audit` gates, CSP & capability review, egress test asserting no unredacted tokens leave via a recording proxy, SQLCipher DB unreadable without key.
- **UI/E2E**: Playwright/WebDriver (tauri-driver) flows — create workspace → profile → add sources → run assessment → review → export PDF/PPTX; visual snapshot checks on generated report pages.
- **Manual**: generate a white-labelled report and deck for a sample system and open in Acrobat/PowerPoint/Keynote.
