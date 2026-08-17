# Personas and Jobs

Spec-Version: 1.1.0

Persona IDs are stable product-contract identifiers. Feature rows use `P-SOLO`, `P-LEAD`, `P-SEC`, and `P-PLAT`; a feature may serve more than one persona.

## Persona 1: Solo Developer (Sam)

- **ID:** `P-SOLO`
- **Role:** Full-stack freelancer
- **Goals:** Ship features fast; use own API keys; minimal setup
- **Pain:** Context switching; repetitive boilerplate; debugging unfamiliar codebases
- **Jobs to be done:**
  - "Explain this codebase and fix the bug in under an hour"
  - "Generate tests for my PR"
  - "Refactor without breaking prod"
  - "Attach a screenshot or diagram and turn the visual evidence into an explained, reviewable change"
- **Plan:** Starter or Pro; BYOK preferred
- **Measurable outcomes:** first useful answer in < 60 s; routine bug fixed and verified in one run; no unapproved write or command
- **Primary workflow:** open project → Ask/Plan → Agent → review diff → verify tests
- **Risk constraints:** cost surprise, accidental overwrite, hidden shell action, provider-key leakage

## Persona 2: Team Lead (Taylor)

- **ID:** `P-LEAD`
- **Role:** Engineering manager, 8-person team
- **Goals:** Consistent quality; visibility into AI usage; shared project context
- **Pain:** Junior devs paste secrets; no audit trail; inconsistent agent behavior
- **Jobs to be done:**
  - "Standardize how we use AI on this repo"
  - "See who ran what commands last week"
  - "Onboard new hire with project memory"
- **Plan:** Business
- **Measurable outcomes:** new engineer reaches a productive first change in one day; every privileged action is attributable; shared policy behaves consistently across the team
- **Primary workflow:** configure project policy and memory → assign seats → review runs, changes, and audit evidence
- **Risk constraints:** inconsistent agent behavior, unreviewed policy exceptions, weak audit coverage, uncontrolled spend

## Persona 3: Security / IT (Jordan)

- **ID:** `P-SEC`
- **Role:** Enterprise security architect
- **Goals:** SSO, policy enforcement, no data exfiltration, compliance evidence
- **Pain:** Shadow AI tools; unapproved MCP servers; no central kill switch
- **Jobs to be done:**
  - "Allow Claude for code read, deny shell on production configs"
  - "Export audit for SOC 2 auditor"
  - "Provision users via SCIM"
- **Plan:** Enterprise
- **Measurable outcomes:** 100% managed users provisioned by SSO/SCIM; prohibited egress blocked; audit export reconciles without sequence gaps
- **Primary workflow:** approve providers → publish policy → monitor exceptions/audit → revoke access or models centrally
- **Risk constraints:** cross-tenant access, prompt injection, unapproved provider/MCP use, retention or residency violation

## Persona 4: Platform Engineer (Riley)

- **ID:** `P-PLAT`
- **Role:** Maintains Eury SaaS monorepo
- **Goals:** Reliable releases; observable agent cloud; clear SLOs
- **Jobs to be done:**
  - "Ship desktop update without breaking auth"
  - "Debug gateway 5xx spike"
  - "Roll out feature flag to 10% users"
- **Plan:** Internal operator / Enterprise platform owner
- **Measurable outcomes:** SLOs remain within budget; rollback completes inside the runbook target; every alert has an owner and runbook
- **Primary workflow:** inspect health → isolate provider/version → stage mitigation → verify recovery → record evidence
- **Risk constraints:** incompatible client rollout, provider outage, failed migration, audit backlog, signing-key compromise

## Feature-family ownership

| Persona | Primary feature families | Secondary feature families |
|---|---|---|
| `P-SOLO` | App shell, chat/agent, local tools, IDE surfaces, intelligence | Authentication, cloud sync |
| `P-LEAD` | Chat/agent, memory/plans, cloud sync, usage | Authentication, policies, audit |
| `P-SEC` | Authentication, enterprise governance, audit, policy, air-gapped | Model routing, MCP, update controls |
| `P-PLAT` | Release/update, observability, gateway, compatibility | Identity, quotas, admin console |

## Acceptance rule

Every feature in the [feature catalog](02-feature-catalog.md) MUST identify at least one primary persona. A capability intended only for an operator still needs a persona (`P-PLAT`); “all users” is not a valid substitute for a concrete job and risk owner.

## Anti-personas (not primary v1)

- Non-developers seeking general chat only → use web Eury chat
- Users needing cloud IDE only → out of scope
- Mobile coding → out of scope

## Related documents

- [02-feature-catalog.md](02-feature-catalog.md)
- [04-pricing-and-packaging.md](04-pricing-and-packaging.md)
