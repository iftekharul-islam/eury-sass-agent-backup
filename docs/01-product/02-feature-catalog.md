# Feature Catalog

Spec-Version: 2.0.0

Legend: **P0** = release-blocking core, **P1** = v1 standard, **P2** = advanced, **P3** = enterprise/GA. Priority expresses release criticality, not chronology; `Phase` is the sole delivery schedule.

Size is implementation effort for one cross-functional team: **XS** ≤ 2 days, **S** ≤ 1 week, **M** ≤ 2 weeks, **L** ≤ 4 weeks, **XL** must be split across phase deliverables. Personas are defined in [personas and jobs](01-personas-and-jobs.md). Entitlement keys are defined in [pricing and packaging](04-pricing-and-packaging.md). Every row is `approved`; later lifecycle changes use `draft`, `implemented`, `verified`, or `deprecated`.

## App shell

| ID | Feature | Priority | Size | Phase | Primary persona | Entitlement | Lifecycle |
|---|---|---|---|---|---|---|---|
| F-001 | Persistent Home / Code product areas | P0 | M | 3 | `P-SOLO` | `agent.desktop` | approved |
| F-002 | Light/dark theme + five accents | P0 | S | 3 | `P-SOLO` | `agent.desktop` | approved |
| F-003 | Collapsible sidebar | P0 | S | 3 | `P-SOLO` | `agent.desktop` | approved |
| F-004 | Command palette | P1 | M | 3 | `P-SOLO` | `agent.desktop` | approved |
| F-005 | Native menus (About, Updates) | P0 | S | 3 | `P-SOLO` | `agent.desktop` | approved |
| F-006 | Multi-window | P2 | L | 23 | `P-LEAD` | `agent.desktop` | approved |
| F-007 | Searchable settings modal and preferences | P0 | M | 3 | `P-SOLO` | `agent.desktop` | approved |
| F-008 | Project-grouped conversation history | P0 | M | 3–9 | `P-SOLO` | `agent.desktop` | approved |

## Authentication

| ID | Feature | Priority | Size | Phase | Primary persona | Entitlement | Lifecycle |
|---|---|---|---|---|---|---|---|
| F-010 | Agent-owned PKCE browser device login | P1 | L | 10 | `P-SOLO` | `agent.desktop` | approved |
| F-011 | OS keychain token storage | P1 | M | 10 | `P-SEC` | `agent.desktop` | approved |
| F-012 | Refresh token rotation | P1 | M | 10 | `P-SEC` | `agent.desktop` | approved |
| F-013 | Sign out and session revoke | P1 | S | 10 | `P-SEC` | `agent.desktop` | approved |
| F-014 | 24-hour offline grace | P1 | M | 10 | `P-SOLO` | `agent.offline_grace` | approved |

## Chat and agent

| ID | Feature | Priority | Size | Phase | Primary persona | Entitlement | Lifecycle |
|---|---|---|---|---|---|---|---|
| F-020 | Streaming chat UI | P0 | L | 8 | `P-SOLO` | `agent.desktop` | approved |
| F-021 | Stop/cancel run | P0 | S | 4 | `P-SOLO` | `agent.desktop` | approved |
| F-022 | Policy-aware model picker and catalog | P0 | M | 11 | `P-SOLO` | `models.catalog` | approved |
| F-023 | Ask mode | P0 | M | 17 | `P-SOLO` | `modes.ask` | approved |
| F-024 | Plan mode | P1 | L | 17 | `P-SOLO` | `modes.plan` | approved |
| F-025 | Agent mode | P0 | XL | 4–7 | `P-SOLO` | `modes.agent` | approved |
| F-026 | Build mode | P1 | L | 17 | `P-SOLO` | `modes.build` | approved |
| F-027 | Review multi-agent workflow | P2 | XL | 20 | `P-LEAD` | `workflows.review` | approved |
| F-028 | Structured tool activity in conversation | P0 | M | 8 | `P-SOLO` | `agent.desktop` | approved |
| F-029 | Live write preview | P0 | L | 6 | `P-SOLO` | `tools.workspace_write` | approved |
| F-030 | Inline tool approval card | P0 | M | 7 | `P-SEC` | `tools.approvals` | approved |
| F-031 | Image attachments and vision | P1 | L | 11 | `P-SOLO` | `multimodal.vision` | approved |
| F-032 | Generated image gallery | P1 | L | 11 | `P-SOLO` | `multimodal.generate` | approved |
| F-033 | Web citation chips | P1 | M | 11 | `P-SOLO` | `tools.network` | approved |
| F-034 | Message actions (copy, retry) | P0 | S | 8 | `P-SOLO` | `agent.desktop` | approved |
| F-035 | File, selection, diff, and terminal attachments | P0 | M | 8 | `P-SOLO` | `agent.desktop` | approved |

## Tools (local)

| ID | Feature | Priority | Size | Phase | Primary persona | Entitlement | Lifecycle |
|---|---|---|---|---|---|---|---|
| F-040 | `list_dir` / `read_file` | P0 | M | 6 | `P-SOLO` | `tools.workspace_read` | approved |
| F-041 | `write_file` / `edit_file` | P0 | L | 6 | `P-SOLO` | `tools.workspace_write` | approved |
| F-042 | `delete_file` / `mkdir` | P1 | M | 6 | `P-SOLO` | `tools.workspace_write` | approved |
| F-043 | `grep` / `glob` | P0 | M | 6 | `P-SOLO` | `tools.workspace_read` | approved |
| F-044 | Sandboxed `run_command` | P1 | L | 12 | `P-SOLO` | `tools.execute` | approved |
| F-045 | `web_search` / `web_fetch` | P1 | L | 11 | `P-SOLO` | `tools.network` | approved |
| F-046 | MCP tools | P2 | XL | 19 | `P-SEC` | `tools.mcp` | approved |
| F-047 | Git status, diff, branch, commit | P1 | L | 14 | `P-SOLO` | `tools.git` | approved |
| F-048 | `read_image` | P1 | M | 11 | `P-SOLO` | `multimodal.vision` | approved |
| F-049 | `generate_image` | P1 | L | 11 | `P-SOLO` | `multimodal.generate` | approved |

## IDE surfaces

| ID | Feature | Priority | Size | Phase | Primary persona | Entitlement | Lifecycle |
|---|---|---|---|---|---|---|---|
| F-050 | File explorer | P1 | M | 13 | `P-SOLO` | `ide.explorer` | approved |
| F-051 | Tabbed editor | P1 | L | 13 | `P-SOLO` | `ide.editor` | approved |
| F-052 | Syntax highlighting | P1 | M | 13 | `P-SOLO` | `ide.editor` | approved |
| F-053 | Integrated PTY terminal | P1 | L | 12 | `P-SOLO` | `ide.terminal` | approved |
| F-054 | Local browser preview | P2 | L | 22 | `P-SOLO` | `ide.preview` | approved |
| F-055 | Git panel | P1 | M | 14 | `P-SOLO` | `ide.git` | approved |
| F-056 | Fast file search | P1 | M | 13 | `P-SOLO` | `ide.explorer` | approved |

## Intelligence

| ID | Feature | Priority | Size | Phase | Primary persona | Entitlement | Lifecycle |
|---|---|---|---|---|---|---|---|
| F-060 | Workspace indexer | P1 | XL | 15 | `P-SOLO` | `intelligence.index` | approved |
| F-061 | Semantic search | P2 | L | 15 | `P-SOLO` | `intelligence.semantic` | approved |
| F-062 | Graph memory | P1 | XL | 16 | `P-LEAD` | `intelligence.memory` | approved |
| F-063 | `EURY.md` rules | P1 | M | 16 | `P-LEAD` | `intelligence.memory` | approved |
| F-064 | Plan files in `.eury/plans/` | P1 | L | 17 | `P-SOLO` | `modes.plan` | approved |
| F-065 | Run checkpoints and rollback | P2 | XL | 18 | `P-SOLO` | `agent.checkpoints` | approved |
| F-066 | Skills discovery | P2 | L | 19 | `P-LEAD` | `intelligence.skills` | approved |

## Local state and safety

| ID | Feature | Priority | Size | Phase | Primary persona | Entitlement | Lifecycle |
|---|---|---|---|---|---|---|---|
| F-067 | Workspace open and explicit trust | P0 | L | 5 | `P-SEC` | `agent.desktop` | approved |
| F-068 | Encrypted local conversations and run journal | P0 | XL | 9 | `P-SOLO` | `agent.desktop` | approved |
| F-069 | Persisted drafts and local preferences | P0 | M | 9 | `P-SOLO` | `agent.desktop` | approved |

## Cloud integration

| ID | Feature | Priority | Size | Phase | Primary persona | Entitlement | Lifecycle |
|---|---|---|---|---|---|---|---|
| F-070 | Home conversation sync | P1 | L | 23 | `P-LEAD` | `cloud.conversation_sync` | approved |
| F-071 | Cloud project/file references | P1 | XL | 23 | `P-LEAD` | `cloud.projects` | approved |
| F-072 | Personalization sync | P1 | M | 10 | `P-SOLO` | `cloud.personalization` | approved |
| F-073 | Billing/subscription UI | P1 | M | 10 | `P-SOLO` | `billing.portal` | approved |
| F-074 | Organization members UI | P2 | L | 24 | `P-LEAD` | `enterprise.org_admin` | approved |
| F-075 | Signed auto-update | P1 | XL | 27 | `P-PLAT` | `agent.updates` | approved |

## Enterprise

| ID | Feature | Priority | Size | Phase | Primary persona | Entitlement | Lifecycle |
|---|---|---|---|---|---|---|---|
| F-080 | SAML/OIDC SSO | P3 | XL | 24 | `P-SEC` | `enterprise.sso` | approved |
| F-081 | SCIM provisioning | P3 | XL | 24 | `P-SEC` | `enterprise.scim` | approved |
| F-082 | Workspace policies | P3 | XL | 25 | `P-SEC` | `enterprise.policy` | approved |
| F-083 | Central audit log | P3 | XL | 25 | `P-SEC` | `enterprise.audit` | approved |
| F-084 | Usage quotas and budgets | P3 | L | 25 | `P-LEAD` | `enterprise.budgets` | approved |
| F-085 | Admin console | P3 | XL | 25 | `P-SEC` | `enterprise.org_admin` | approved |
| F-086 | Air-gapped/self-hosted profile | P3 | XL | 24 | `P-SEC` | `enterprise.air_gapped` | approved |

## Explicitly dropped from code-old

| Feature | Reason |
|---------|--------|
| Regex tool-call fabrication | Unreliable |
| Prose censoring in parser | Corrupts output |
| `CODE_API_TOKEN` dev bypass in prod | Security |
| Plaintext `auth.json` | Keychain |
| `HOME=workspace` shell hack | Breaks tools |
| Whole-file JSON session store | SQLite |

## Related documents

- [03-modes-and-workflows.md](03-modes-and-workflows.md)
- [../09-roadmap/00-roadmap-overview.md](../09-roadmap/00-roadmap-overview.md)
