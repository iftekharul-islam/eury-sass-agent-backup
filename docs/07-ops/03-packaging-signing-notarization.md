# Packaging, Signing, and Notarization

Spec-Version: 1.1.0

## Artifacts

| OS / arch | Format | Signing | Notes |
|---|---|---|---|
| macOS arm64 | `.dmg` (+ `.app.tar.gz` for updater) | Developer ID Application + notarization + staple | Hardened runtime, no `com.apple.security.get-task-allow` |
| macOS x86_64 | `.dmg` | same | Universal binary considered; two artifacts chosen for size |
| Windows x86_64 | `.msi` (WiX) | Authenticode, EV certificate, RFC 3161 timestamp | Per-user install by default |
| Linux x86_64 | `.AppImage`, `.deb` | Detached GPG signature + SHA-256 | `.rpm` deferred |

Naming, exactly:

```
Eury-Agent-1.2.0-aarch64.dmg
Eury-Agent-1.2.0-x64.dmg
Eury-Agent-1.2.0-x64.msi
Eury-Agent-1.2.0-x86_64.AppImage
Eury-Agent_1.2.0_amd64.deb
```

Bundle identifier `com.eury.agent`; product name "Eury Agent". Nothing reuses the legacy `Eury-Code-*` names or `com.eury.code` identifier ([naming and migration map](../00-overview/05-naming-and-migration-map.md)).

## Version synchronization

A single `agent/version.json` is the source of truth. A CI step asserts equality across:

- `agent/apps/desktop/package.json`
- `agent/apps/desktop/src-tauri/tauri.conf.json`
- `agent/Cargo.toml` (workspace `version`)
- the git tag `agent-vX.Y.Z`
- the `AgentRelease.version` created by the pipeline

Any mismatch fails the release before signing.

## macOS

| Step | Detail |
|---|---|
| Entitlements | Minimal set: JIT is not requested; network client; user-selected files read/write |
| Hardened runtime | Required; library validation on |
| Sandbox | App Store sandbox not used (the app must execute user build tools); Seatbelt is applied per tool process instead ([sandbox model](../03-security/02-sandbox-model.md)) |
| Signing order | Sign embedded binaries and frameworks inner-to-outer, then the `.app`, then the `.dmg` |
| Notarization | `notarytool submit --wait`, then `stapler staple` on both `.app` and `.dmg` |
| Verification | `codesign --verify --deep --strict`, `spctl --assess --type execute` |
| First-run | Quarantine attribute cleared by staple; no "damaged app" dialog |

Keychain access on macOS is tied to the code signature. Re-signing with a different identity invalidates saved keychain items, which is why the identity is pinned per channel and rotated only with a documented migration.

## Windows

| Step | Detail |
|---|---|
| Certificate | EV code-signing certificate on a hardware token / cloud HSM |
| Timestamping | RFC 3161, mandatory (signatures must outlive the certificate) |
| Signed objects | `agent.exe`, bundled sidecars, the `.msi`, and the updater |
| Install scope | Per-user (`%LOCALAPPDATA%\Programs\Eury Agent`), no admin prompt |
| SmartScreen | EV certificate avoids the reputation ramp; reputation still monitored post-release |
| Verification | `signtool verify /pa /all` |
| Uninstall | Removes program files; leaves user data unless "remove my data" is checked |

## Linux

| Step | Detail |
|---|---|
| Build base | Oldest supported glibc (Ubuntu 22.04) for portability |
| Dependencies | `libwebkit2gtk-4.1`, `libayatana-appindicator3` declared in the `.deb` |
| Signature | Detached GPG `.asc` per artifact plus a signed `SHA256SUMS` file |
| Sandbox | Landlock when available; documented degradation when not |
| Desktop entry | MIME handler for the `eury-agent://` scheme |

## Update manifest signing

The update manifest is signed independently of the installers (minisign/Ed25519). The desktop verifies the manifest signature **and** each artifact's SHA-256 before applying an update, so compromising the CDN alone is not sufficient to push a malicious update ([auto-update](04-auto-update-and-rollback.md)).

Key hierarchy: an offline root signing key held in an HSM signs channel keys; channel keys sign manifests. Public keys are embedded in the binary; rotation requires a release that ships the new public key, so rotations are planned one version ahead.

## SBOM and provenance

| Output | Detail |
|---|---|
| SBOM | SPDX 2.3 JSON, generated from `cargo-sbom` + `pnpm licenses`, one per artifact |
| Provenance | SLSA-style attestation naming the workflow, commit SHA, and builder |
| Checksums | `SHA256SUMS` covering every artifact, signed |
| Publication | Attached to the GitHub Release and uploaded next to the artifacts in object storage |

## Reproducibility

Builds pin the Rust toolchain, Node version, and lockfiles, and set `SOURCE_DATE_EPOCH`. Binaries are not byte-for-byte reproducible today (signing and notarization embed timestamps); the goal is reproducible *pre-signing* artifacts, verified by a nightly job that builds twice and diffs. Divergence is tracked as a bug.

## Release verification checklist

Automated in `agent-release.yml`, all must pass before a draft release is created:

- [ ] Version consistency across all five locations
- [ ] Every expected artifact present for every platform
- [ ] macOS: `codesign --verify --deep --strict` and `spctl --assess` pass; staple verified
- [ ] Windows: `signtool verify /pa /all` passes; timestamp present
- [ ] Linux: GPG signature verifies against the published key
- [ ] Checksums match the values registered with the admin API
- [ ] Manifest signature verifies with the embedded public key
- [ ] SBOM generated and non-empty for each artifact
- [ ] `--verify-install` exits 0 on each platform
- [ ] Fresh-install smoke: launch, sign in against staging, run one read-only tool
- [ ] Upgrade smoke: install N−1, update to N, settings and conversations intact

## Key management

| Key | Storage | Rotation |
|---|---|---|
| Apple Developer ID | Apple-issued cert in CI secret store, HSM-backed where possible | On expiry or compromise |
| Windows EV | Cloud HSM, no exportable private key | On expiry or compromise |
| GPG (Linux) | Offline primary, CI-held signing subkey | Subkey yearly |
| Update manifest root | Offline HSM | 3 years |
| Update manifest channel | CI secret store | Yearly |

Compromise response, including emergency re-sign and forced-update path: [incident response](../03-security/09-incident-response.md).

## Related documents

- [CI/CD pipelines](02-ci-cd-pipelines.md)
- [Auto-update and rollback](04-auto-update-and-rollback.md)
- [Supply chain and signing](../03-security/06-supply-chain-and-signing.md)
