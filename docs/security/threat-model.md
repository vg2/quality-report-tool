# Threat Model — v1 (Phase 0)

Date: 2026-11-20 · Method: STRIDE per trust boundary · Review: each phase,
and whenever a new source connector, AI provider or export path is added.

## Assets
| Asset | Why it matters |
|---|---|
| Customer IP (code, docs, metrics, client identity) | Contractual confidentiality; core requirement in `context.md` |
| Assessment results & findings | Reputational and commercial sensitivity for the client |
| Credentials (PATs, API keys) and database keys | Give access to enterprise systems and to all stored data |
| Report integrity | Stakeholders make decisions on the grades |

## Trust boundaries
1. **Webview ↔ Rust core (IPC)** — the UI renders untrusted text; treat it as potentially compromised.
2. **Rust core ↔ local disk** — databases, temp copies of sources.
3. **Rust core ↔ OS keychain.**
4. **Rust core ↔ sources** (git, CI, wiki, local folders) — *Phase 1.*
5. **Rust core ↔ inference providers** — *Phase 1.*
6. **Evidence content ↔ AI model context** (prompt injection) — *Phase 1.*
7. **Generated reports ↔ recipients** — *Phase 1.*

## Threats and mitigations

Status: ✅ implemented in Phase 0 · 🔜 planned (phase) · ⚠️ accepted risk

### 1. Webview ↔ IPC
| STRIDE | Threat | Mitigation | Status |
|---|---|---|---|
| E | XSS in the UI calls powerful native APIs | Command capabilities (only app commands granted); no fs/shell/http plugins; isolation-hook allowlist | ✅ |
| T/E | Injected script loaded from remote origin | Strict CSP (`script-src 'self'`, no `eval`, `connect-src` IPC only); `freezePrototype` | ✅ |
| S | Webview navigated to attacker page that then uses IPC | `on_navigation` allows app origins only | ✅ |
| T | Malformed/hostile IPC arguments (path traversal, oversized input) | Validated domain types; UUID-only identifiers; file paths never derived from user text | ✅ |
| I | Error messages leak paths/SQL/secrets to UI | Generic `CommandError`; details logged in Rust only | ✅ |
| T | Untrusted Markdown/HTML from evidence or AI rendered as DOM | DOMPurify-sanitised rendering, no `dangerouslySetInnerHTML` of raw content | 🔜 P1 |

### 2. Local disk
| STRIDE | Threat | Mitigation | Status |
|---|---|---|---|
| I | Another local user / stolen laptop reads databases | SQLCipher AES-256, per-database random keys in OS keychain; owner-only file permissions | ✅ |
| I | Deleted data recoverable from DB pages | `secure_delete=ON` | ✅ |
| T | Downgrade: older app opens newer schema and corrupts it | Migrations refuse `user_version` newer than supported | ✅ |
| T/I | Crafted DB + `-wal` pair leaks process memory (RUSTSEC-2026-0459) | Patched SQLite 3.54.1 via `sqlcipher-src` 420; DBs only created by the app in its own data dir | ✅ |
| I | Temp copies of sources linger on disk | Per-run sandbox, purge after run and on startup | 🔜 P1 |
| I | OS backups / sync (Time Machine, OneDrive) copy data | Data is encrypted at rest; keys are not in the backup unless the keychain is synced | ⚠️ document for users |

### 3. Keychain
| STRIDE | Threat | Mitigation | Status |
|---|---|---|---|
| I | Secrets in DB, config or logs | Secrets only in keychain; `DbKey` zeroised and redacted in `Debug` | ✅ |
| I | Other apps of the same user read keychain items | macOS: item ACL bound to the signed app (requires code signing). Linux Secret Service: any app in the session can read | ⚠️ until signing (P1); Linux limitation documented |
| D | Keychain entry lost → database unrecoverable | Passphrase-protected export/backup | 🔜 P2 |

### 4–7. Later boundaries (summary)
| Boundary | Key threats | Planned mitigations (`docs/plan.md` §6–§9) |
|---|---|---|
| Sources | Malicious repo content (git hooks, symlinks, zip bombs, huge files); modifying originals | Read-only, copy-first sandbox; no hooks/LFS/submodules; never execute repo code; symlink containment; size limits; fuzzed parsers |
| Inference providers | Customer IP used for training or retained; data sent to unapproved provider | Per-workspace provider policy (incl. local-only); pseudonymisation; secret scanning; minimisation; egress preview + audit log |
| Prompt injection | Evidence text steers the AI to change scores or exfiltrate data | Dual-LLM pattern; no side-effecting tools; schema-validated output; verified citations; injection heuristics; human review gate; red-team corpus in CI |
| Reports | Tampered or mis-attributed reports | Methodology appendix with catalog/prompt/model versions; signed exports (P2) |

### Supply chain & distribution
| Threat | Mitigation | Status |
|---|---|---|
| Vulnerable / malicious dependency | cargo-deny, pnpm audit, lockfiles, Dependabot, SBOM | ✅ |
| Tampered binary or update | Code signing, notarisation, signed updater | 🔜 P1 |
| Compromised CI action | Actions pinned by tag; pin to SHAs before first release | ⚠️ → 🔜 P1 |

## Open questions
- Should workspace import (P2) accept only signed bundles from this app?
- Do any target clients require FIPS-validated crypto (affects SQLCipher/OpenSSL build)?
