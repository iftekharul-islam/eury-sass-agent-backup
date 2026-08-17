# Cloud Data Model (Prisma Additions)

Spec-Version: 1.1.0

Additions to `backend/prisma/schema.prisma`.

## Ownership rules

| # | Rule |
|---|---|
| D1 | Every Agent-owned model is prefixed `Agent` — no name can collide with an existing or future platform model |
| D2 | Existing models are modified **only** by adding back-relation fields (no column changes, renames, or drops) |
| D3 | Only the Agent module reads/writes `Agent*` tables |
| D4 | Reads of `User`, `Organization`, `OrganizationMember`, `Subscription`, `Plan` are read-only and by id |
| D5 | Conventions per workspace rule: `@default(cuid())`, `createdAt`/`updatedAt`, both relation sides, `@@index` on every query path |

## AgentDevice

```prisma
model AgentDevice {
  id             String    @id @default(cuid())
  userId         String
  user           User      @relation(fields: [userId], references: [id], onDelete: Cascade)
  organizationId String?
  organization   Organization? @relation(fields: [organizationId], references: [id], onDelete: SetNull)
  name           String
  platform       String    // darwin | win32 | linux
  arch           String    // aarch64 | x86_64
  appVersion     String
  osVersion      String?
  publicKey      String?   // ed25519, base64 — verifies audit batches
  lastSeenAt     DateTime?
  revokedAt      DateTime?
  createdAt      DateTime  @default(now())
  updatedAt      DateTime  @updatedAt

  authSessions   AgentAuthSession[]
  refreshTokens  AgentRefreshToken[]
  auditEvents    AgentAuditEvent[]

  @@index([userId])
  @@index([organizationId])
  @@index([lastSeenAt])
}
```

## AgentAuthSession

Replaces the legacy `IdeAuthSession` for Agent logins. The legacy table is untouched.

```prisma
model AgentAuthSession {
  id                  String    @id @default(cuid())
  deviceCodeHash      String    @unique          // sha256 of device_code
  userCode            String    @unique          // human-typed, e.g. BQDX-7T2M
  codeChallenge       String                     // PKCE S256 challenge
  codeChallengeMethod String    @default("S256")
  deviceName          String
  platform            String
  appVersion          String
  userId              String?
  user                User?     @relation(fields: [userId], references: [id], onDelete: Cascade)
  deviceId            String?
  device              AgentDevice? @relation(fields: [deviceId], references: [id], onDelete: SetNull)
  status              String    @default("pending")  // pending | approved | denied | consumed
  approvedAt          DateTime?
  consumedAt          DateTime?
  pollCount           Int       @default(0)
  expiresAt           DateTime
  createdAt           DateTime  @default(now())
  updatedAt           DateTime  @updatedAt

  @@index([expiresAt])
  @@index([status, expiresAt])
  @@index([userId])
}
```

## AgentRefreshToken

```prisma
model AgentRefreshToken {
  id             String    @id @default(cuid())
  userId         String
  user           User      @relation(fields: [userId], references: [id], onDelete: Cascade)
  deviceId       String
  device         AgentDevice @relation(fields: [deviceId], references: [id], onDelete: Cascade)
  tokenHash      String    @unique         // sha256, never the raw token
  chainId        String                    // shared across rotations of one login
  replacedById   String?   @unique
  replacedBy     AgentRefreshToken?  @relation("AgentRefreshRotation", fields: [replacedById], references: [id])
  replaces       AgentRefreshToken?  @relation("AgentRefreshRotation")
  expiresAt      DateTime
  revokedAt      DateTime?
  revokedReason  String?                   // rotated | reuse_detected | logout | device_revoked | admin
  lastUsedAt     DateTime?
  createdAt      DateTime  @default(now())
  updatedAt      DateTime  @updatedAt

  @@index([userId])
  @@index([deviceId])
  @@index([chainId])
  @@index([expiresAt])
}
```

Reuse of a token whose `replacedById` is set revokes every token sharing `chainId` and emits an `auth.refresh_reuse` audit event.

## AgentRelease

Own table — the legacy `DesktopRelease` is not extended, aliased, or read.

```prisma
model AgentRelease {
  id            String    @id @default(cuid())
  version       String                          // semver, e.g. 1.2.0
  channel       String    @default("stable")    // stable | beta | canary
  minSupported  String
  mandatory     Boolean   @default(false)
  releaseNotes  String?   @db.Text
  artifactsJson Json                            // { "darwin-aarch64": { url, sizeBytes, sha256 }, ... }
  manifestSig   String?                         // minisign over canonical manifest
  isActive      Boolean   @default(false)
  publishedAt   DateTime?
  publishedById String?
  publishedBy   User?     @relation(fields: [publishedById], references: [id], onDelete: SetNull)
  rolledBackAt  DateTime?
  downloadCount Int       @default(0)
  createdAt     DateTime  @default(now())
  updatedAt     DateTime  @updatedAt

  @@unique([version, channel])
  @@index([channel, isActive])
  @@index([publishedAt])
}
```

Exactly one `isActive = true` row per channel; enforced in `AgentReleaseService` inside a transaction.

## AgentPolicy

```prisma
model AgentPolicy {
  id             String   @id @default(cuid())
  organizationId String
  organization   Organization @relation(fields: [organizationId], references: [id], onDelete: Cascade)
  scope          String   @default("org")   // org | team | user
  scopeRef       String?                    // teamId or userId when scope != org
  name           String
  version        Int      @default(1)
  rulesJson      Json                       // WorkspacePolicy schema
  isActive       Boolean  @default(false)
  createdById    String?
  createdBy      User?    @relation(fields: [createdById], references: [id], onDelete: SetNull)
  createdAt      DateTime @default(now())
  updatedAt      DateTime @updatedAt

  @@unique([organizationId, name])
  @@index([organizationId, scope, isActive])
}
```

## AgentAuditEvent

```prisma
model AgentAuditEvent {
  id             String   @id             // client-supplied uuid → idempotent ingest
  organizationId String?
  organization   Organization? @relation(fields: [organizationId], references: [id], onDelete: SetNull)
  userId         String
  user           User     @relation(fields: [userId], references: [id], onDelete: Cascade)
  deviceId       String?
  device         AgentDevice? @relation(fields: [deviceId], references: [id], onDelete: SetNull)
  runId          String?
  eventType      String                    // tool_start | tool_end | policy_denied | auth.* | run_*
  severity       String   @default("info") // info | warn | critical
  payloadJson    Json                      // redacted per privacy doc
  batchId        String?
  occurredAt     DateTime
  ingestedAt     DateTime @default(now())

  @@index([organizationId, occurredAt])
  @@index([userId, occurredAt])
  @@index([runId])
  @@index([eventType, occurredAt])
  @@index([batchId])
}
```

Append-only: no update or delete paths in code; retention handled by a scheduled purge job ([audit and retention](../06-enterprise/04-audit-and-retention.md)).

## AgentUsageCounter

```prisma
model AgentUsageCounter {
  id               String   @id @default(cuid())
  organizationId   String?
  organization     Organization? @relation(fields: [organizationId], references: [id], onDelete: Cascade)
  userId           String
  user             User     @relation(fields: [userId], references: [id], onDelete: Cascade)
  period           String                     // YYYY-MM
  promptTokens     BigInt   @default(0)
  completionTokens BigInt   @default(0)
  requestCount     Int      @default(0)
  costUsdMicros    BigInt   @default(0)       // integer money, no floats
  createdAt        DateTime @default(now())
  updatedAt        DateTime @updatedAt

  @@unique([userId, period])
  @@index([organizationId, period])
}
```

Redis holds the hot counter; this table is the durable rollup flushed every 60 s and on process shutdown.

## AgentSyncCursor

```prisma
model AgentSyncCursor {
  id         String   @id @default(cuid())
  userId     String
  user       User     @relation(fields: [userId], references: [id], onDelete: Cascade)
  deviceId   String
  device     AgentDevice @relation(fields: [deviceId], references: [id], onDelete: Cascade)
  resource   String                      // conversations | memory
  cursor     String
  syncedAt   DateTime @default(now())
  createdAt  DateTime @default(now())
  updatedAt  DateTime @updatedAt

  @@unique([deviceId, resource])
  @@index([userId])
}
```

## Additive relations on existing models

Only these lines are added. Nothing existing is renamed or removed.

```prisma
// model User
agentDevices        AgentDevice[]
agentAuthSessions   AgentAuthSession[]
agentRefreshTokens  AgentRefreshToken[]
agentAuditEvents    AgentAuditEvent[]
agentPolicies       AgentPolicy[]        // created-by
agentReleases       AgentRelease[]       // published-by
agentUsageCounters  AgentUsageCounter[]
agentSyncCursors    AgentSyncCursor[]

// model Organization
agentDevices        AgentDevice[]
agentPolicies       AgentPolicy[]
agentAuditEvents    AgentAuditEvent[]
agentUsageCounters  AgentUsageCounter[]
```

Note the two relations from `AgentDevice` and `AgentPolicy` to `User` need explicit relation names when a model references `User` more than once — e.g. `@relation("AgentPolicyCreatedBy", …)`. Run `prisma validate` before committing.

## Migrations

| Order | Name | Contents |
|---|---|---|
| 1 | `agent_identity` | `AgentDevice`, `AgentAuthSession`, `AgentRefreshToken` + `User`/`Organization` back-relations |
| 2 | `agent_control_plane` | `AgentPolicy`, `AgentAuditEvent`, `AgentUsageCounter`, `AgentSyncCursor` |
| 3 | `agent_releases` | `AgentRelease` |

All three are additive (`CREATE TABLE` / `CREATE INDEX` only), so they are safe to deploy before the Agent code ships and safe to run on a live database without locking existing tables. Migration discipline: review the generated SQL before committing, never edit an applied migration, verify `prisma migrate status` is clean in staging first, and run `prisma validate` after adding the relation fields below.

Rollback: the tables are unreferenced by other modules, so `DROP TABLE` in reverse order is safe if the Agent module is removed.

## Data classification

| Table | Contains user content? | Retention |
|---|---|---|
| `AgentDevice`, `AgentAuthSession`, `AgentRefreshToken` | No | Until revoked + 90 d |
| `AgentPolicy` | No | Indefinite (versioned) |
| `AgentAuditEvent` | Metadata only, redacted payload | Org-configured, default 90 d |
| `AgentUsageCounter` | No | 25 months |
| `AgentSyncCursor` | No | Until device revoked |

Conversation content is only stored server-side when sync is explicitly enabled, and then in the existing `Conversation`/`Message` tables via `/agent/v1/sync` — no new content tables ([privacy](../03-security/07-privacy-and-data-residency.md)).

## Conformance tests

| ID | Test |
|---|---|
| T1 | Every model the Agent module owns is `Agent`-prefixed (schema lint) |
| T2 | Every migration is additive: `prisma migrate diff` shows no dropped or renamed column on pre-existing models |
| T3 | Changes to existing models are limited to new optional fields and back-relations |
| T4 | Every foreign key has the documented `onDelete` behavior, verified by cascade tests |
| T5 | Every documented index exists, and every hot query uses one (`EXPLAIN` assertions) |
| T6 | Token tables store only hashes; a seeded plaintext token never appears in any column |
| T7 | Refresh-token rotation revokes the entire family on reuse |
| T8 | `AgentAuditEvent` rows are insert-only: update and delete are rejected at the database level |
| T9 | The audit hash chain verifies across a seeded 10 000-event device sequence, and an injected gap is detected |
| T10 | Usage counters are exact under 100 concurrent writers (no lost updates) |
| T11 | Retention jobs delete exactly the rows past their tier boundary and nothing else |
| T12 | Deleting an organization removes all its Agent-owned rows with no orphans |

## Related documents

- [Cloud API contract](06-cloud-api-contract.md)
- [Backend module structure](16-backend-module-structure.md)
- [Naming and migration map](../00-overview/05-naming-and-migration-map.md)
