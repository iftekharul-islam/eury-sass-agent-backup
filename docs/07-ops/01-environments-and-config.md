# Environments and Configuration

Spec-Version: 1.1.0

Every Agent-specific variable is namespaced `EURY_AGENT_*` (desktop) or `AGENT_*` (cloud) so it can never collide with the legacy `code` stack ([naming and migration map](../00-overview/05-naming-and-migration-map.md)).

## Desktop configuration

Precedence, lowest to highest: compiled defaults → machine config file → user settings (SQLite) → environment variables → command-line flags. Policy is not part of this chain: policy can only **restrict** the result ([workspace policies](../06-enterprise/03-workspace-policies.md)).

### Environment variables

| Variable | Default | Description |
|---|---|---|
| `EURY_AGENT_DATA_DIR` | OS app-data dir for `com.eury.agent` | Override the data directory |
| `EURY_AGENT_CLOUD_URL` | `https://api.eury.app` (`http://localhost:4001` in dev) | Control-plane base URL |
| `EURY_AGENT_LOG_LEVEL` | `info` | `trace\|debug\|info\|warn\|error` |
| `EURY_AGENT_LOG_FORMAT` | `text` | `text\|json` |
| `EURY_AGENT_DEV` | unset | Enable devtools and dev-only commands |
| `EURY_AGENT_OFFLINE` | unset | Fully disable outbound Eury calls ([air-gapped](../06-enterprise/07-air-gapped-and-self-hosted.md)) |
| `EURY_AGENT_CA_BUNDLE` | unset | Extra PEM bundle for private CAs |
| `EURY_AGENT_POLICY_FILE` | unset | Signed local policy file path |
| `EURY_AGENT_DISABLE_SANDBOX` | unset | Dev only; refused in release builds |
| `EURY_AGENT_DISABLE_INDEX` | unset | Skip workspace indexing (debugging) |
| `EURY_AGENT_MODEL_BASE_URL` | unset | OpenAI-compatible endpoint override for BYOK |
| `HTTPS_PROXY`, `NO_PROXY` | OS | Standard proxy handling |

Release builds hard-fail at startup if `EURY_AGENT_DISABLE_SANDBOX` is set, rather than silently ignoring it.

### CLI flags

| Flag | Effect |
|---|---|
| `--workspace <path>` | Open a workspace |
| `--safe-mode` | Read-only tools, no MCP, no index, minimal UI |
| `--reset-settings` | Restore defaults, keep conversations |
| `--export-logs <path>` | Write a redacted diagnostic bundle and exit |
| `--verify-install` | Check signature, keychain access, sandbox availability, then exit |
| `--version`, `--help` | Standard |

### Machine config file (MDM-friendly)

| OS | Path |
|---|---|
| macOS | `/Library/Application Support/Eury/agent.toml` (or MDM plist `com.eury.agent`) |
| Windows | `%ProgramData%\Eury\agent.toml` (or `HKLM\Software\Policies\Eury\Agent`) |
| Linux | `/etc/eury/agent.toml` |

```toml
cloud_url = "https://eury.internal.acme.com"
policy_file = "/etc/eury/agent-policy.json"
policy_public_key = "…"
offline = false
auto_update = false
telemetry = false
```

Machine config can set defaults and can **lock** keys (`locked = ["cloud_url", "policy_file"]`) so users cannot override them.

### Local paths

Inside `EURY_AGENT_DATA_DIR`:

```
db/agent.sqlite         conversations, runs, settings, audit queue
db/agent.sqlite-wal
index/<workspace-hash>/ index shards
checkpoints/<run_id>/   file snapshots for rollback
logs/agent.log          rotated 10 MB × 7
cache/                  model catalog, release manifests, price table
crash/                  pending crash reports (opt-in upload)
```

Secrets are never in these files; they live in the OS keychain ([secrets and key management](../03-security/04-secrets-and-key-management.md)).

## Cloud configuration

### Agent-owned variables

| Variable | Required | Description |
|---|---|---|
| `AGENT_JWT_SECRET` | yes | Signs Agent access tokens (distinct from the platform's `JWT_SECRET`) |
| `AGENT_REFRESH_TOKEN_SECRET` | yes | HMAC pepper for refresh-token hashing |
| `AGENT_UPSTREAM_LLM_URL` | yes (managed) | Upstream inference service base URL |
| `AGENT_RELEASE_BUCKET` | yes | Object storage bucket for installers |
| `AGENT_RELEASE_CDN_BASE` | no | Public CDN base for download URLs |
| `AGENT_RELEASE_SIGNING_KEY_ID` | yes (prod) | Key id used to sign update manifests |
| `AGENT_RELEASE_FALLBACK_VERSION` | no | Manifest fallback when no `AgentRelease` row is active |
| `AGENT_RELEASE_MIN_SUPPORTED` | no | Fallback minimum supported version |
| `AGENT_RELEASE_URL_DARWIN` / `_WIN32` / `_LINUX` | no | Fallback download URLs |
| `AGENT_SCIM_TOKEN` | no | Enables SCIM routes; absent → routes 404 |
| `AGENT_AUDIT_ARCHIVE_BUCKET` | no | Long-term audit archive |
| `AGENT_SIEM_WEBHOOK_URL` | no | SIEM delivery endpoint |
| `AGENT_POLICY_SIGNING_KEY_ID` | no | Signs enterprise policy documents |
| `AGENT_MAX_STREAM_SECONDS` | no (300) | Gateway wall-clock timeout |

### Shared infrastructure variables (read, not owned)

`DATABASE_URL`, `REDIS_URL`, `JWT_SECRET` (only to verify the web session during device exchange), provider keys (`OPENAI_API_KEY`, `CLAUDE_API_KEY`, `GEMINI_API_KEY`, …), object-storage credentials, and `TAVILY_API_KEY`.

The Agent module reads these through its own `AgentConfigService`, which validates them at boot and fails fast. It does not import another module's config service.

### Validation rules

| Rule | Behavior |
|---|---|
| Missing required secret | Process exits non-zero at boot with the variable name |
| Weak secret (< 32 bytes) | Boot failure in staging and production |
| Secret equal to another secret | Boot failure (`AGENT_JWT_SECRET` must differ from `JWT_SECRET`) |
| Unknown `AGENT_*` variable | Warning, listed at boot (catches typos) |
| Production + `mock: true` allowed | Boot failure |

There is deliberately **no** variable that disables Agent authentication. The legacy `CODE_API_TOKEN`-unset-means-open behavior is not reproduced ([backend module structure](../04-specs/16-backend-module-structure.md)).

## Environments

| Name | Cloud URL | Frontend | Release channel | Secrets source |
|---|---|---|---|---|
| `local` | `http://localhost:4001` | `http://localhost:3000` | none (dev build) | `.env` (git-ignored) |
| `ci` | ephemeral container | — | none | CI secret store, synthetic keys |
| `staging` | `https://api-staging.eury.app` | `https://staging.eury.app` | `beta` | platform secret manager |
| `production` | `https://api.eury.app` | `https://eury.app` | `stable` | platform secret manager |

Staging uses a separate database, Redis namespace, release bucket prefix, and signing keys. No production secret is ever present in CI for non-release jobs.

## Build-time configuration

Non-secret values baked into the desktop binary at build time: default `EURY_AGENT_CLOUD_URL`, release channel, telemetry endpoint, build id, git SHA, and the bundled price-table version. Injected as CI variables, verified by `--verify-install`, and displayed in Settings → About.

Secrets are never baked into the desktop binary. There is no client secret in the PKCE flow, by design.

## Configuration change management

| Concern | Rule |
|---|---|
| Adding a variable | Must be added to this table and to `agent-env.schema.ts` in the same PR |
| Rotation | Access-token and refresh secrets support dual-key verification during rotation |
| Rotation cadence | Signing keys annually, service secrets quarterly, immediately on suspected compromise |
| Drift detection | Boot logs a hash of the effective non-secret config; a staging/production diff is reviewed each release |

## Related documents

- [Packaging, signing, notarization](03-packaging-signing-notarization.md)
- [CI/CD pipelines](02-ci-cd-pipelines.md)
- [Secrets and key management](../03-security/04-secrets-and-key-management.md)
- [Naming and migration map](../00-overview/05-naming-and-migration-map.md)
