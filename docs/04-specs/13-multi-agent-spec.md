# Multi-Agent Specification

Spec-Version: 2.0.0

Sub-agents exist for one reason: some tasks are better done by a fresh context with a narrow job than by one long conversation that has to hold everything at once. They are not a way to look sophisticated, and they are not a default ([ADR-0010](../02-architecture/adr/0010-multi-agent-orchestration-model.md)).

## When a sub-agent is the right tool

| Use a sub-agent when | Do not use one when |
|---|---|
| The subtask needs a large, disposable context (searching a big codebase, reading many files) | The task is a few tool calls the main agent can do directly |
| The subtask needs different permissions (read-only review of code the main agent wrote) | It only adds indirection and latency |
| The result is a small summary from a lot of input (context compression by delegation) | The main agent needs the full intermediate detail anyway |
| Independent work can genuinely proceed in parallel | The subtasks share mutable state |
| A second, uncontaminated opinion has value (review) | You are trying to fix a prompt problem with architecture |

The last row is the honest failure mode of every multi-agent system: spawning agents to compensate for a weak prompt makes runs slower, costlier, and harder to debug.

## Roles

| Role | Tool classes | Writes | Output | Purpose |
|---|---|---|---|---|
| `explorer` | read | no | Findings summary with file references | Search and comprehension over large codebases |
| `planner` | read, network | plan artifacts only | A [plan](11-plan-format-spec.md) | Turn a goal into steps |
| `implementer` | read, write, execute | yes, in a worktree | Changed files + summary | Execute one plan step or scoped change |
| `tester` | read, execute | test files only | Test report with pass/fail and output | Write and run tests |
| `reviewer` | read | no | Findings with severity and locations | Critique a diff without the author's context |
| `documenter` | read, write | docs paths only | Changed docs | Write documentation for a change |

Roles are **permission profiles first**. A `reviewer` cannot write; that is enforced by the registry, not by asking it nicely in a prompt.

## Permission inheritance

The rule that makes delegation safe:

> A sub-agent's effective permissions are the **intersection** of its role profile, its parent's effective permissions, and the workspace policy. A sub-agent can never hold a permission its parent lacks, and a grant given to a sub-agent never propagates upward or sideways.

| Rule | Detail |
|---|---|
| Grants | `session` and `always` grants apply; a sub-agent's own approvals are scoped to `run` at most |
| Approvals | Surface in the parent's approval queue, labeled with the sub-agent's role and goal, and count against the parent's context |
| Denials | A denial in a sub-agent is returned as a structured result; the sub-agent may not re-ask |
| Escalation | A sub-agent cannot spawn a sub-agent with broader permissions than itself |
| Depth | Maximum nesting depth 2; the third level is refused with `EURY_RUN_SUBAGENT_DEPTH` |

Without intersection, delegation would be the easiest privilege-escalation path in the product: an `ask`-mode conversation could spawn an `implementer` and write files.

## Orchestration patterns

Three supported shapes. Anything else is a composition of these.

### Delegate (one shot)

The main agent calls `spawn_subagent`, waits, and receives a summary. Used by `explorer`, `reviewer`, and `documenter`.

```
main -> spawn_subagent { role: "explorer", goal, contextRefs }
     <- { summary, findings[], filesRead, usage }
```

### Fan-out (parallel, read-only)

Multiple independent sub-agents run concurrently. Restricted to non-writing roles, because concurrent writers to one worktree are a correctness disaster and the complexity is not worth it.

| Limit | Value |
|---|---|
| Max concurrent sub-agents | 3 |
| Allowed roles | `explorer`, `reviewer` |
| Result handling | Results are merged in spawn order for determinism, not completion order |

### Pipeline (plan → implement → test → review)

The full workflow, used by Build mode and the review workflow.

```
planner  -> plan artifact                    [user approves the plan]
implementer (in a worktree, per step)        [normal approval flow per tool]
tester   -> test report
reviewer -> findings
main     -> presents the diff + report + findings   [user approves the merge]
```

Every arrow is a checkpoint boundary. The user can stop, inspect, and revert at each one, and the two bracketed gates are mandatory: the user approves the plan before implementation, and the user approves the merge before anything leaves the worktree.

## Worktree isolation

Writing sub-agents work in a git worktree when the workspace is a repo:

| Aspect | Behavior |
|---|---|
| Creation | `git worktree add $EURY_AGENT_DATA_DIR/worktrees/<runId> -b eury/<slug>` |
| Path boundary | The worktree becomes the sub-agent's workspace root; all path guards apply to it |
| Untracked files | Copied in when the parent tree is dirty, so the sub-agent sees the user's real state |
| Merge | Never automatic. The user reviews a unified diff and approves; the merge is a normal checkpointed apply into the main tree |
| Cleanup | The worktree and branch are removed after merge or discard; orphans are reaped on startup |
| Non-repo workspaces | Writing sub-agents are disabled; only read-only roles are available, and the UI explains why |
| Conflicts | If the main tree changed under a touched path, the conflict is shown per file with a choice; nothing is auto-resolved |

## Context passing

Sub-agents start with a **fresh** context. They receive only what is passed explicitly:

```typescript
interface SubagentRequest {
  role: SubagentRole;
  goal: string;                       // 20..2000 chars, self-contained
  contextRefs?: {                     // references, never inlined content
    files?: string[];
    planId?: string; stepId?: string;
    diffFromRunId?: string;
    memoryIds?: string[];
  };
  constraints?: string[];             // "do not modify the public API"
  limits?: { maxTurns?: number; maxCostUsdMicros?: number; maxWallMs?: number };
  model?: ModelConfig;                // may differ; explorers can use a cheaper model
}
```

The sub-agent does **not** inherit conversation history, and returns only a bounded summary:

```typescript
interface SubagentResult {
  agentRunId: string;
  ok: boolean;
  summary: string;                    // <= 4000 tokens, hard cap
  findings?: { severity: string; path?: string; line?: number; text: string }[];
  filesChanged?: FileChange[];
  testReport?: { passed: number; failed: number; output: string };
  usage: TokenUsage; costUsdMicros: number;
  stopReason: string;
  worktree?: string;
}
```

That asymmetry — references in, bounded summary out — is the entire point. A sub-agent that returned its full transcript would consume more parent context than doing the work inline.

## Limits and budgets

| Limit | Default | Enforced by |
|---|---|---|
| Concurrent sub-agents per run | 3 | Runtime |
| Total sub-agents per run | 10 | Runtime |
| Nesting depth | 2 | Runtime |
| Turns per sub-agent | 20 | Sub-agent limits |
| Wall clock per sub-agent | 10 min | Sub-agent limits |
| Cost per orchestrated run | $10, org-configurable | [Cost guard](../06-enterprise/05-usage-quotas-and-budgets.md) |
| Summary size | 4 000 tokens | Truncated with a notice |
| Combined budget | The parent's remaining budget is the ceiling for all children | Cost guard |

A sub-agent hitting a limit returns a partial result with `stopReason` set. The parent decides what to do; it is never handed a silent empty answer.

## Failure handling

| Failure | Behavior |
|---|---|
| Sub-agent fails | Structured failure returned; the parent may retry once with a narrowed goal, then must proceed or stop |
| Sub-agent hangs | Wall-clock limit terminates it; the worktree is preserved for inspection |
| Parent cancelled | All children cancelled recursively; the parent waits 5 s then force-terminates ([runtime](01-agent-runtime-spec.md)) |
| Child crashes the app | On restart, orphan worktrees are detected and the user is offered inspect or discard |
| Approval times out in a child | The child pauses; the parent shows it as blocked rather than failing the whole run |
| Two children want the same file | Impossible by design — only one writer exists at a time |
| Merge conflict | Surfaced per file; the user resolves |

## Observability

Sub-agents are first-class in the UI, not hidden machinery. Each appears as a nested, collapsible run block showing role, goal, live turn count, cost, and status, expandable into its own full activity list. The Runs surface shows the tree with per-node cost, and the total cost of an orchestrated run always shows the breakdown by role. Events: `subagentSpawned`, `subagentProgress`, `subagentCompleted` ([event protocol](03-event-protocol-spec.md)). Audit records every spawn with role, goal hash, and inherited permission set.

## Cost transparency

Because orchestration multiplies spend, the UI must state the cost model before the fact: the plan card shows an estimate, the spawn shows the child's budget ceiling, and the run summary breaks cost down by role. An orchestrated run that costs 5× a single-agent run is acceptable only if the user knew that before starting it.

## Conformance tests

| ID | Test |
|---|---|
| T1 | A sub-agent never holds a permission its parent lacks (property test over role × parent-grant combinations) |
| T2 | An `ask`-mode parent cannot obtain writes through any sub-agent |
| T3 | Nesting depth 3 is refused |
| T4 | Sub-agent approvals appear in the parent queue, labeled, and `Esc` denies |
| T5 | A `reviewer` cannot write, even when the model tries repeatedly |
| T6 | Worktree path guards prevent escape into the main tree |
| T7 | Nothing reaches the main tree without an explicit merge approval |
| T8 | Cancelling the parent terminates every child, worktree, and process within 5 s |
| T9 | Fan-out results merge in spawn order, deterministically |
| T10 | Combined child cost never exceeds the parent's remaining budget |
| T11 | Summary truncation at 4 000 tokens is enforced and flagged |
| T12 | Orphan worktrees from a simulated crash are detected and cleaned up |
| T13 | Non-repo workspaces expose only read-only roles |
| T14 | An eval task confirms the pipeline beats a single agent on a multi-file feature, and the result is recorded in the [eval harness](../08-quality/02-agent-eval-harness.md) |

T14 is the honesty check on this entire document. If orchestration does not measurably beat a single agent, it should not ship.

## Related documents

- [ADR-0010](../02-architecture/adr/0010-multi-agent-orchestration-model.md)
- [Agent runtime](01-agent-runtime-spec.md)
- [Plan format](11-plan-format-spec.md)
- [Permission and policy engine](../03-security/03-permission-and-policy-engine.md)
- [Modes and workflows](../01-product/03-modes-and-workflows.md)
