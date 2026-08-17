# Checkpoint and Rollback Specification

Spec-Version: 2.0.0

Checkpoints are the reason users can let the agent write code without fear. The promise is narrow and absolute: **anything the agent changed, the user can put back**, without git, without knowing what happened, in one action.

## Guarantees

| # | Guarantee |
|---|---|
| C1 | No mutating tool executes until its checkpoint is durably written |
| C2 | Restore returns every captured path to its exact prior bytes, mode, and line endings |
| C3 | Restore is itself checkpointed, so an accidental restore is also undoable |
| C4 | Restore is atomic per operation: either every path is restored or none are |
| C5 | Restore never touches a file the run did not modify |
| C6 | Checkpoints are independent of git; they work in a non-repo directory and with a dirty tree |
| C7 | A checkpoint that cannot be written fails the tool call — correctness beats convenience |
| C8 | Restore always shows exactly what will change before it changes anything |

C1 and C7 together mean a full disk stops the agent from writing rather than letting it write unrecoverably.

## What is captured

| Trigger | Captured |
|---|---|
| Before `write_file`, `edit_file`, `multi_edit` | The prior file content, or a "did not exist" marker |
| Before `delete_file` | Full content (delete is a move into the checkpoint store) |
| Before `move_file` | Both paths and the source content |
| Before a build step | All files the step declares, plus the git HEAD and dirty state |
| Before `checkpoint_restore` | Current content of every path about to be overwritten (C3) |
| Manual | Any user-selected set, via "Create checkpoint" |

Not captured, deliberately: files outside the workspace (they need a separate grant and are never auto-mutated), files above 100 MB (a warning is shown instead), and side effects of `run_command`. The last one is the honest limitation: a checkpoint restores files, not databases, containers, or pushed commits. The UI says so at the point where `run_command` is approved.

## Storage layout

```
$EURY_AGENT_DATA_DIR/checkpoints/
  objects/<sha256[0:2]>/<sha256>          # content-addressed, zstd-compressed
  runs/<runId>/<checkpointId>.json        # manifest
```

Content addressing means ten edits to one file cost one object per distinct version, and two files with identical content share one object. Manifests are small JSON; the bytes live in `objects/` with a reference count in SQLite ([local data model](05-local-data-model.md)).

```json
{
  "id": "01J...",
  "runId": "01J...",
  "workspaceId": "01J...",
  "seq": 7,
  "kind": "pre_tool",
  "toolCallId": "call_abc",
  "label": "before edit_file src/auth.ts",
  "createdAt": "2026-08-16T09:00:00.123Z",
  "git": { "head": "9f2c1a…", "branch": "main", "dirty": true },
  "files": [
    {
      "path": "src/auth.ts",
      "existed": true,
      "sha256": "4d9f…",
      "sizeBytes": 4211,
      "mode": 420,
      "eol": "lf",
      "hadTrailingNewline": true
    }
  ]
}
```

The manifest is written and `fsync`ed before the tool runs (C1). Writes use temp-file-and-rename so a crash never leaves a half-written manifest.

## Granularity

Three nested levels, all restorable:

| Level | Scope | Typical UI entry point |
|---|---|---|
| Tool | One tool call | "Undo this change" on a tool card |
| Turn | Every tool call in one assistant turn | "Restore to before this message" |
| Run | Every change in the run | "Revert everything from this run" |
| Step | One build step | "Revert this step" on the plan card |

Restoring a turn or run replays the individual checkpoints in reverse order, coalesced so each path is written exactly once with its oldest captured version.

## Restore

Two-phase, always (C8):

```
Phase 1 — checkpoint_preview_restore { checkpointId }
  returns RestorePlan {
    files: [{ path, action: "restore"|"delete"|"recreate", currentSha, targetSha,
              conflict: bool, diffPreview }],
    conflicts: [{ path, reason: "modified_externally"|"missing_parent"|"permission" }],
    warnings: string[],       // e.g. "3 files were changed by you after this checkpoint"
    confirmToken: string      // valid 5 minutes, bound to this exact plan
  }

Phase 2 — checkpoint_restore { checkpointId, paths?, confirmToken }
  1. re-verify every current hash against the preview; any drift invalidates the token
  2. create a reverse checkpoint of the current state (C3)
  3. write all files to a staging area
  4. verify every staged file's hash
  5. atomically rename each into place; on any failure, roll back the renames
  6. record restored_at, emit audit + fileChanged events
```

| Conflict | Default handling |
|---|---|
| The user edited the file after the checkpoint | Flagged in the preview; the user chooses per file, and nothing is silently overwritten |
| The file was deleted after the checkpoint | Offered as "recreate" |
| The parent directory is gone | Recreated, with the directory listed in the preview |
| The file is open with unsaved editor changes | Blocked until the user saves or discards |
| Permission denied | Listed as a conflict; the restore proceeds for the other files only if the user opts in, otherwise nothing changes (C4) |

## Git integration

Git is used where it adds value and never depended on:

| Situation | Behavior |
|---|---|
| Repo, clean tree | The checkpoint records HEAD; restore offers "use `git checkout` instead" as an equivalent, cheaper path |
| Repo, dirty tree | File snapshots are authoritative; git state is recorded only for context |
| Build step in a repo | An optional `git stash` is offered before the step, on top of file snapshots, never instead of them |
| Not a repo | Fully functional; this is the case that justifies the whole subsystem |
| Agent-created commits | Never rewritten by restore. The UI explains that a commit needs a git-level undo, and offers the exact command |

## Retention

| Rule | Default |
|---|---|
| Age | 14 days |
| Size | 2 GB per workspace |
| Count | 100 runs per workspace |
| Eviction order | Oldest run first; a run with any restorable change referenced by an active plan is kept longer |
| Protected | The current run, the last 3 runs, and manual checkpoints are never auto-evicted |
| Garbage collection | Reference-counted objects are deleted when no manifest references them; runs on idle, at most hourly |
| User actions | "Clear checkpoints" per workspace or globally, with the reclaimed size shown |
| Pressure | Below 1 GB free disk, retention drops to 3 days and the user is warned; below 200 MB, mutating tools stop with `EURY_STORE_DISK_FULL` (C7) |

## Crash recovery

On startup, for each run left in a non-terminal state: read its checkpoint manifests, compare the recorded hashes against the files on disk, and classify. The user is shown one clear choice.

| Finding | Offer |
|---|---|
| No file differs from its pre-checkpoint state | Resume the run |
| Some files were modified | Resume, or revert to the run's first checkpoint, with the file list shown |
| A manifest is unreadable | That checkpoint is quarantined and excluded; others still work, and the user is told which change is unrecoverable |
| Objects are missing | The affected paths are marked unrestorable, explicitly and by name — never a silent partial restore |

## Performance and limits

| Operation | Target |
|---|---|
| Checkpoint a single file under 1 MB | < 10 ms p95 |
| Checkpoint 50 files | < 200 ms p95 |
| Preview a restore of 50 files | < 150 ms |
| Restore 50 files | < 500 ms |
| Compression ratio on source code | ≈ 3–4× with zstd level 3 |
| Storage for a typical 20-file run | < 5 MB |
| Max files per checkpoint | 5 000 (above this, the operation is refused with an explanation) |
| Max single file | 100 MB |

## Audit and events

Every checkpoint creation and restore is audited with the checkpoint id, run id, file count, byte count, and workspace hash — never file contents. Events emitted: `toolCallEnded.checkpointId`, `fileChanged` per restored path, and a `notice` summarizing the restore ([audit](../06-enterprise/04-audit-and-retention.md)).

## Conformance tests

| ID | Test |
|---|---|
| T1 | No mutating tool runs when checkpoint writing fails (disk full, permission denied) |
| T2 | Round-trip fidelity: content, mode, EOL style, trailing newline, and BOM all preserved |
| T3 | Binary files, empty files, 100 MB files, and files with hostile unicode names all restore correctly |
| T4 | Restore is atomic: an injected failure at each stage leaves the tree fully old or fully new |
| T5 | Restore creates a reverse checkpoint that restores the restore (C3) |
| T6 | Restore never modifies an untouched file (C5) |
| T7 | Externally modified files surface as conflicts and are never silently overwritten |
| T8 | An invalid, expired, or mismatched `confirmToken` is refused |
| T9 | Turn-level and run-level restore coalesce correctly to the oldest version per path |
| T10 | Content addressing deduplicates: 10 edits to one file store 10 objects, and 2 identical files store 1 |
| T11 | Garbage collection never deletes a referenced object (property test) |
| T12 | Kill -9 between manifest write and tool execution leaves a valid, restorable checkpoint |
| T13 | Works identically in a non-git directory |
| T14 | Retention eviction never removes the current or last three runs |

## Related documents

- [Local data model](05-local-data-model.md)
- [Agent runtime](01-agent-runtime-spec.md)
- [Tool catalog](02-tool-catalog-spec.md)
- [Tool activity and diff UX](../05-ui/04-tool-activity-and-diff-ux.md)
