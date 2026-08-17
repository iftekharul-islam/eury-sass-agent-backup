# Supply Chain and Signing

Spec-Version: 2.0.0

**Owner:** Platform + Security · **Lifecycle:** approved design contract

## Dependencies

| Ecosystem/input | Required control |
|---|---|
| Rust/Cersei | Committed `Cargo.lock`; `--locked`; exact Cersei version; `cargo audit` high/critical gate; `cargo deny` advisory/yank/license/source/duplicate policy |
| Node/pnpm | Committed lockfile; frozen install; high/critical audit gate; lifecycle scripts disabled or explicitly reviewed/allowlisted |
| GitHub Actions | Full immutable commit SHA, least permissions, reviewed update PR; no tag-only action reference |
| Downloaded tools/assets | HTTPS plus pinned version and SHA-256/signature; no pipe-to-shell |
| MCP | Artifact/executable plus normalized tool-manifest fingerprint; organization allowlist where required |

Dependency updates are isolated PRs with changelog, transitive diff, new
build/lifecycle scripts, license/advisory result, owner, and rollback notes.

### MCP servers

- User-installed; not bundled. Display server hash/command at enable time.
- Org allowlist required for enterprise.

## Build pipeline

| Stage | Control |
|-------|---------|
| Source | Protected branch, CODEOWNERS, required checks; verified identity per organization policy |
| CI | Ephemeral least-privilege runners, pinned actions/tools, locked inputs, artifact hashes |
| Build | Hermetic/reproducible target where practical; network/input manifest; no long-lived secret in build process |
| Release | SLSA provenance/attestation, SPDX 2.3 SBOM, checksums, platform and update signatures |

## Code signing

| Platform | Requirement |
|----------|-------------|
| macOS | Apple Developer ID + notarization |
| Windows | Authenticode EV cert |
| Linux | Signed `.deb`/AppImage metadata/artifacts with published key and checksum |

All distributed stable/beta artifacts are signed. Unsigned local developer
builds are clearly labeled, never update-capable, and never uploaded to a
release channel.

## Auto-update

- Manifest from `GET /agent/v1/releases/latest` includes artifact hash, size,
  version/channel, minimum version, provenance and SBOM digests.
- Verify root-authorized channel key and artifact signature/hash before apply.
- A small offline root public-key set authorizes versioned channel keys and a
  signed revocation list. Emergency channel-key compromise recovery does not
  depend on a release signed only by that compromised key.
- Support rollback to the previous compatible signed version (Phase 27).

## SBOM

Generate SPDX 2.3 JSON per release, validate it, bind its digest into
provenance/manifest, and attach it to the release.

## Exceptions

No security job is disabled inline. A time-bounded exception record contains
id, advisory/rule/license, exact package/path, justification, compensating
control, owner, approver, created/expiry dates, and removal target. Critical
advisories, secret findings, unpinned actions, signature failures, and
prohibited licenses are non-waivable for release. CI fails expired exceptions.

## Phase 2 gates

PR/main/weekly workflows run `cargo audit`, `cargo deny` (via `deny.toml`), `pnpm audit`,
gitleaks (via `.gitleaks.toml`), Semgrep tests/scan, corpus/security-contract checks, lockfile checks,
and PR dependency review. Positive/negative rule fixtures prove detection
without treating later SBOM generation, signing, sandbox tests, or fuzzing as
implemented.

## Related documents

- [../07-ops/03-packaging-signing-notarization.md](../07-ops/03-packaging-signing-notarization.md)
- [../07-ops/02-ci-cd-pipelines.md](../07-ops/02-ci-cd-pipelines.md)
