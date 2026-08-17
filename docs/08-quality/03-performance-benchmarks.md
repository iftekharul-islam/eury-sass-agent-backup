# Performance Benchmarks

Spec-Version: 1.1.0

## Methodology

| Aspect | Rule |
|---|---|
| Reference hardware | Apple M1 Pro / 16 GB (primary), Windows x86_64 8-core / 16 GB, Ubuntu 22.04 x86_64 8-core / 16 GB |
| Build | Release, LTO on, the exact artifact shipped |
| Warmup | 3 discarded iterations for micro-benchmarks |
| Iterations | 1000 for micro (`criterion`), 20 for macro (app-level), 5 for cold start |
| Reporting | p50 / p95 / p99 plus standard deviation; never a single mean |
| Isolation | No other user processes; CI runners are noisy, so CI results gate only on large regressions and the reference machine produces the published numbers |
| Model calls | Excluded from our numbers, measured separately — provider latency is not ours to optimize |

## Targets

### Desktop

| Benchmark | p95 target | Notes |
|---|---|---|
| Cold start → shell interactive | < 400 ms | Skeleton visible, input focusable |
| Cold start → fully ready | < 2 s | Settings loaded, workspace listed |
| Workspace open (10k files) → chat usable | < 500 ms | Index continues in background |
| IPC round-trip (no-op command) | < 1 ms | Serialization overhead only |
| Event delivery latency (core → DOM) | < 16 ms | One frame |
| `read_file` 10 KB | < 10 ms | Includes path guard |
| `read_file` 1 MB | < 40 ms | |
| `write_file` 10 KB + checkpoint | < 25 ms | Includes snapshot |
| `grep` across 10k files | < 300 ms | Index-backed |
| Context assembly (10k-file repo) | < 30 ms | Retrieval + prompt build |
| Context assembly (50k-file repo) | < 80 ms | |
| Memory recall (graph) | < 1 ms | Validates the vendor claim independently |
| Policy evaluation per tool call | < 0.5 ms | Cached policy |
| Diff computation, 500-line file | < 20 ms | Rust side |
| SQLite conversation load (500 messages) | < 50 ms | |
| Index full build (10k files) | < 60 s | Background, throttled |
| Index incremental update (1 file) | < 200 ms | |
| Terminal throughput | ≥ 10 MB/s | UI stays ≥ 55 fps |
| Streaming frame rate | ≥ 55 fps | While a run streams |
| Idle CPU | < 1% | No run active |
| Idle RSS | < 350 MB | One workspace, index loaded |
| RSS with 50k-file index | < 800 MB | |
| Installer size | < 40 MB per platform | |

### Cloud

| Benchmark | p95 target |
|---|---|
| `/agent/v1/health` | < 10 ms |
| `/agent/v1/auth/device/poll` | < 40 ms |
| `/agent/v1/auth/refresh` | < 50 ms |
| `/agent/v1/policies/effective` (cached) | < 30 ms |
| `/agent/v1/models` | < 30 ms |
| `/agent/v1/audit/batch` (500 events) | < 300 ms |
| Gateway added latency (excl. upstream) | < 400 ms |
| Gateway guard chain (auth + quota + policy) | < 10 ms |

End-to-end user-perceived targets and their derivation: [latency budget](../02-architecture/05-latency-budget.md).

## Benchmark suite layout

```
agent/bench/
  micro/        criterion benches per crate (ipc, store, policy, sandbox, index, diff)
  macro/        scripted app runs measuring cold start, workspace open, run latency
  cloud/        k6 scripts per endpoint
  fixtures/     generated repos: 1k, 10k, 50k, 200k files
  results/      JSON per run, appended to the trend dataset
```

Macro benchmarks drive the real app through the automation harness, so they measure what a user experiences rather than an isolated function.

## Independent verification of vendor claims

Cersei's published numbers (tool dispatch, graph memory recall, throughput) are reproduced in our own harness on our own hardware before any of them is repeated externally. Results, methodology, and variance go in `agent/bench/REPORT.md`. Rule: **never cite a vendor number in marketing or docs without a matching entry in our report.** If our measurement differs materially, our number is the one we use, and the discrepancy is documented.

## Regression policy

| Change vs. baseline | Action |
|---|---|
| < 5% | Accepted, recorded |
| 5–10% | Warning; must be explained in the PR |
| > 10% on any target | CI fails; either fixed or an explicit, approved baseline change |
| > 25% | Blocks release regardless of justification |
| Memory > 10% | Same as latency |
| Installer size > 5% | Requires justification |

Baselines are stored per platform and per hardware profile, updated only by an explicit commit that states why. CI compares against the baseline with a noise allowance derived from the runner's measured variance, so a noisy runner cannot silently pass a real regression or fail a clean build.

## Profiling workflow

When a regression appears: `cargo flamegraph` for Rust hot paths, Chrome DevTools performance profile for the webview, `tracing` spans for run-level attribution, and `heaptrack`/Instruments for memory. Every performance fix lands with the before/after numbers in the PR description.

## Load and soak

| Test | Detail |
|---|---|
| Cloud load | k6 ramp to 2× projected peak on the gateway and auth routes; error rate < 0.1%, p95 within target |
| Cloud spike | 10× for 60 s; degradation must be graceful (429s, not 5xx) |
| Desktop soak | 8-hour session with periodic runs; no memory growth beyond 10%, no fd leaks, no orphaned processes |
| Large repo soak | 200k-file monorepo; index stays responsive, degrades to lexical-only if configured |
| Streaming endurance | 30-minute continuous stream; steady frame rate and bounded memory |

## Schedule

| When | What |
|---|---|
| Every PR | Micro benchmarks for touched crates (fast subset) |
| Weekly on `main` | Full micro + macro on all three platforms |
| Pre-release | Full suite on reference hardware + load tests + 8-hour soak |
| Quarterly | Re-verify vendor claims and refresh `REPORT.md` |

## Related documents

- [Latency budget](../02-architecture/05-latency-budget.md)
- [Test strategy](01-test-strategy.md)
- [Observability and SLOs](../07-ops/05-observability-and-slos.md)
- [ADR-0001: embed Cersei](../02-architecture/adr/0001-embed-cersei-in-desktop.md)
