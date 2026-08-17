# Secrets and Key Management

Spec-Version: 2.0.0

**Owner:** Security + Platform · **Lifecycle:** approved design contract

## Principles

1. Never embed, log, export, or place a secret in ordinary persistent storage.
2. Desktop long-lived secrets use the OS keychain; cloud secrets use a managed
   secret service or KMS/HSM.
3. Keychain/secret-manager failure never falls back to a file, database,
   environment variable, command argument, or clipboard (strict no-plaintext-fallback).
4. Managed model routing never exposes Eury's provider credentials to desktop.
5. Automated secret redaction and scrubbing apply to all logging, telemetry, and export paths.

## Canonical secret inventory

Desktop keychain service is `eury-agent`; the account field below is the stable
logical locator. SQLite stores only opaque id, provider/server id, timestamps,
and keychain availability state.

| ID | Secret | Store | Account / locator | Rotate or revoke |
|---|---|---|---|---|
| SEC-001 | BYOK provider key | OS keychain | `provider/<provider>/<profileId>` | Replace; delete on disconnect |
| SEC-002 | Access token | OS keychain + bounded process memory | `auth/access/<accountId>` | 15 min expiry; delete on logout |
| SEC-003 | Refresh token | OS keychain | `auth/refresh/<accountId>` | Rotate every use; reuse revokes chain |
| SEC-004 | Local database key | OS keychain | `db/key/<profileId>` | Transactional re-key |
| SEC-005 | Device audit private key | Non-exportable keychain when supported | `audit/device/<deviceId>` | Re-enroll and revoke public key |
| SEC-006 | MCP credential | OS keychain | `mcp/<serverId>/<credentialId>` | Replace/delete with server |
| SEC-007 | Managed provider secret | Cloud secret manager | `agent/provider/<region>/<provider>` | Platform rotation runbook |
| SEC-008 | JWT/policy/audit/update signing private key | KMS/HSM | Versioned key id | Dual-control rotate/revoke |
| SEC-009 | CI signing/notarization/publishing credential | CI OIDC or protected HSM secret | Environment scoped | Short-lived; release runbook |
| SEC-010 | SCIM/SSO/webhook/registry credential | Cloud secret manager | Tenant/integration scoped | Admin rotation plus audit |

## Platform failure behavior

| State | Required behavior |
|---|---|
| Keychain locked | Invoke OS unlock UI; auth/BYOK/database operations remain unavailable |
| Keychain unavailable | Show platform remedy; never create a fallback secret file |
| Linux Secret Service absent/headless | BYOK and persistent login disabled; history/export remains available |
| Access denied or corrupt record | Treat as absent and require re-auth/re-entry |
| Logout/device revoke | Delete local auth records, revoke cloud chain/device, clear process buffers |
| Uninstall | Offer explicit local/keychain cleanup; server device revoke remains independent |

Supported backends are macOS Keychain, Windows Credential Manager, and Linux
Secret Service through a reviewed Rust keyring abstraction.

## Token lifecycle

| Token | TTL | Rule |
|---|---|---|
| Access JWT | 15 min | Audience/device bound; refresh through SEC-003 |
| Refresh token | 30 d maximum | Rotated on every use; reuse revokes full chain |
| Agent device code | 10 min | PKCE S256, single use, rate limited |

Refresh failure deletes invalid access state and starts Agent-owned login. No
token appears in URL query, log, crash breadcrumb, analytics, support bundle, or
child process.

## Process-memory and boundary handling

- Materialize secrets only for the smallest request; use zeroizing buffers and
  avoid clone/format/debug. Best-effort lock pages where supported.
- Child environments start from an allowlist and contain no Eury/provider/MCP
  credential.
- IPC/UI receive only configured state, labels, and timestamps.
- Redaction runs before log, audit, telemetry, provider context, crash,
  clipboard, and support-export boundaries. Failure drops the payload.
- Crash reports exclude heap/register dumps, environment, prompts, file
  contents, command arguments, and secret values.

## Cloud, MCP, and release secrets

Workload identity fetches minimum scoped secret versions. Secrets are absent
from source, database columns, container layers, reusable developer `.env`
files, and build output. MCP config references SEC-006 ids and rejects
`${env:...}`. Signing keys follow the root-keyset rotation and incident process
in [supply chain and signing](06-supply-chain-and-signing.md).

## Migration from `code-old`

Never import `~/.eury-code/auth.json`; prompt for Agent login and new BYOK/MCP
credential entry.

## Compromise response

1. Record secret id/version, tenant/region, exposure window, and affected route
   without copying the value.
2. Revoke first, rotate in the owning store, and invalidate dependent
   sessions/policies/manifests.
3. Force re-auth/re-enrollment and verify old-version rejection.
4. Search provider/CI/store audit records, preserve evidence, notify under the
   incident matrix, and record closure tests.

## Contract verification

- TEST-006 scans SQLite, logs, telemetry, audit, crash/support exports, child
  environments, and model requests with the secret corpus; no value survives.
- Keychain-unavailable fixtures prove no plaintext fallback.
- Rotation/reuse fixtures prove revoked versions fail.

## Related documents

- [Cloud architecture](../02-architecture/03-cloud-architecture.md)
- [Supply chain and signing](06-supply-chain-and-signing.md)
- [Incident response](09-incident-response.md)
