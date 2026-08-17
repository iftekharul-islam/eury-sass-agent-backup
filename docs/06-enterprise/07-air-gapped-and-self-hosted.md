# Air-Gapped and Self-Hosted

Spec-Version: 1.1.0

Two distinct offerings that are often conflated:

- **Air-gapped desktop** — the app runs with no Eury cloud at all.
- **Self-hosted cloud** — the customer runs the Nest control plane themselves.

Both are enabled by the same architecture: the agent loop is local, and every cloud dependency is optional or configurable ([ADR-0001](../02-architecture/adr/0001-embed-cersei-in-desktop.md)).

## Air-gapped desktop

### Requirements

| Requirement | Implementation |
|---|---|
| Zero outbound calls to Eury | `EURY_AGENT_OFFLINE=1` disables update checks, telemetry, audit upload, sync, and license phone-home |
| Model access | BYOK to an on-prem endpoint: OpenAI-compatible base URL, Azure OpenAI, AWS Bedrock via VPC endpoint, or local vLLM/Ollama |
| Custom TLS | `EURY_AGENT_CA_BUNDLE` for private certificate authorities |
| Proxy | Honors `HTTPS_PROXY`/`NO_PROXY`; supports proxy auth from the keychain |
| No telemetry | Compile-time verifiable: the offline build fails CI if any network target other than the configured model endpoint is reachable in the egress test |
| Audit | Written to local NDJSON files with the same hash chain, rotated daily, collectable by the customer's agent |
| Policy | Loaded from a signed local file (`/etc/eury/agent-policy.json` or the Windows equivalent), verified against a customer-provided public key |
| Updates | Manual installer sideload; `minSupported` cannot be enforced remotely, so the admin controls it via the local policy file |
| Licensing | Signed offline license file with expiry and seat count; 30-day grace after expiry, then read-only mode |
| Indexing | Fully local; embeddings from a bundled local model, no hosted embedding calls |
| MCP | Local servers only; registry allowlist by manifest hash |

### Verification

An **egress test** runs in CI for the offline build profile: the app runs a scripted session inside a network namespace that logs every connection attempt. Any destination other than the configured model endpoint fails the build. The resulting report ships in the enterprise deployment package as evidence.

### Deliverables for the customer

Installer bundles per platform, SBOM, signature/checksum list, the local policy schema, the egress report, an offline license, and a deployment guide (generated from these docs, not written separately).

## Self-hosted cloud

### Components

| Component | Requirement |
|---|---|
| Nest API (with `AgentModule`) | Container image, 2 vCPU / 4 GB minimum per replica |
| PostgreSQL | 15+ |
| Redis | 7+ (quotas, rate limits, abort handles) |
| Object storage | S3-compatible (MinIO acceptable) for releases and audit archive |
| Identity | Local users or the customer's IdP via SAML/OIDC |
| TLS termination | Customer-provided ingress |

Because the Agent surface is a self-contained module, a self-hosted deployment can run with other feature modules disabled by configuration — the customer does not have to operate billing or marketing surfaces they do not use ([backend module structure](../04-specs/16-backend-module-structure.md)).

### Configuration

Desktop points at the customer's URL via `EURY_AGENT_CLOUD_URL`, set at install time by a config profile (MDM plist, Windows registry policy, or `/etc/eury/agent.toml`) so users do not type it.

Required server env: `DATABASE_URL`, `REDIS_URL`, `AGENT_JWT_SECRET`, `AGENT_REFRESH_TOKEN_SECRET`, `AGENT_RELEASE_BUCKET`, and either provider keys or an upstream model endpoint ([environments and config](../07-ops/01-environments-and-config.md)).

### Delivery and upgrades

Versioned container images with the Agent module's Prisma migrations bundled and additive-only, so a rollback of the image does not require a database rollback. Customers are supported on N and N−1 minor versions. A `--verify-config` startup mode validates env, DB connectivity, and secret presence, then exits.

### What is not supported self-hosted (v1)

Managed Eury inference (the customer must bring keys or an endpoint), Eury-operated backups and monitoring, and Eury-hosted release distribution. These are documented as customer responsibilities in the deployment guide.

## Licensing

| Aspect | Rule |
|---|---|
| Format | Signed JSON: org name, seat count, features, `notBefore`, `notAfter`, issuer key id |
| Verification | Ed25519 against a bundled public key; tampering is unrecoverable, not a warning |
| Grace | 30 days after `notAfter`, banner from day 1 of grace |
| Post-grace | Read-only mode: no write, execute, or network tools; local data remains accessible and exportable |
| Seat enforcement | Distinct `deviceId` count against seat count, evaluated locally in air-gapped mode |
| Privacy | License checks in connected mode send only license id and seat count — never workspace or user content |

## Threat considerations specific to these modes

| Risk | Mitigation |
|---|---|
| No cloud policy distribution | Signed local policy file with version monotonicity; downgrades rejected |
| No remote revocation | Short-lived local sessions and admin-controlled `minVersion`; MDM removal is the revocation path |
| Audit tampering on the endpoint | Hash chain plus customer-side log shipping; gaps are detectable |
| Stale, vulnerable client | Local policy can pin a minimum version and refuse to run below it |
| Exfiltration via model endpoint | Endpoint allowlist in policy; egress test evidence |

## Delivery

Configuration hooks (`EURY_AGENT_OFFLINE`, CA bundle, proxy, local policy file, local audit sink) land incrementally through Phases 7–25 as each subsystem is built. The offline build profile, the egress verification report, the self-hosted image, licensing, and the deployment guide all land in Phase 27. Nothing here requires an architectural change later.

## Related documents

- [Offline and degraded modes](../02-architecture/06-offline-and-degraded-modes.md)
- [Privacy and data residency](../03-security/07-privacy-and-data-residency.md)
- [Environments and configuration](../07-ops/01-environments-and-config.md)
- [Compliance baseline](../03-security/08-compliance-baseline.md)
