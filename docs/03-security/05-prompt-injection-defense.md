# Prompt Injection Defense

Spec-Version: 2.0.0

**Owner:** Security + Agent Runtime · **Lifecycle:** approved design contract

## Threat

Untrusted content in workspace files, web pages, MCP responses, or pasted chat may instruct the model to ignore policies, exfiltrate secrets, or run dangerous tools.

## Untrusted content sources

| Source | Trust level |
|--------|-------------|
| Current user prompt | Trusted intent, not authority to bypass policy |
| `EURY.md`, user rules, reviewed plan/memory | Semi-trusted context |
| Repository files | **Untrusted** |
| `web_fetch` body | **Untrusted** |
| MCP tool results | **Untrusted** |
| MCP tool names/descriptions/schemas | **Untrusted** |
| Tool/terminal/preview output | **Untrusted** |
| Attachments, OCR, image alt text, citations | **Untrusted** |
| Summaries/compaction/sub-agent output | Inherits the lowest trust of its inputs |
| Other users' messages (shared project) | **Untrusted** |

## Defenses

### 1. Structural separation and provenance

Policy/tool authority exists only in compiled product instructions and the
verified effective policy. Dynamic content uses this canonical envelope:

```typescript
interface ContextBlock {
  id: string;
  source: "user" | "rule" | "memory" | "plan" | "workspace" | "tool" |
          "web" | "mcp" | "attachment" | "other_user" | "summary";
  sourceId: string;
  trust: "trusted" | "semi_trusted" | "untrusted";
  mediaType: string;
  byteLength: number;
  sha256: string;
  transformChain: string[];
  dataBase64: string;
}
```

The provider adapter decodes `dataBase64` into a data segment; it never
concatenates raw content into XML, Markdown fences, or control delimiters.
Lengths/hashes are checked before and after decode. Unknown source/trust values
are rejected. Transformations and summaries follow monotonic trust: they retain
or lower trust, never raise it. Provenance metadata adheres to `ContextProvenance`
defined in `security-types.schema.json`.

### 2. Provider framing

Adapters emit a fixed trusted instruction that semi-trusted/untrusted blocks are
evidence, never policy. Raw bytes are provider data parts preceded by generated
metadata, so hostile closing tags, Unicode controls, nested fences, base64
payloads, or role-like text cannot escape a delimiter.

### 3. Tool argument validation

Before execution, scan tool args for:

- Paths outside workspace (sandbox rejects)
- Commands matching Tier 3 forbidden list
- URLs to internal/metadata addresses (SSRF)

### 4. Secret redaction

Before sending context to model:

- Redact patterns: AWS keys, GitHub tokens, `sk-...`, JWT-shaped strings
- Log redaction count in audit metadata

### 5. No secret tools

Agent MUST NOT have a tool to read keychain or env vars wholesale.

### 6. Output monitoring

Hooks on `PostToolUse`: flag if tool output contains high-entropy secrets → warn user.

### 7. Grant containment

Injection detection is defense in depth, not a claim of perfect classification.
When a block is flagged, that block remains available as marked evidence, but
any causally derived write/execute/network/MCP action:

1. cannot use a standing `session` or `always` grant;
2. requires a fresh `once` approval showing the flagged source;
3. receives no secret-bearing context or child-process environment;
4. is denied if the source/transform provenance is missing.

Non-approvable policy denials remain denied. The finding and invalidated grant
ids are audited without recording content.

## What we do NOT rely on

- Model "refusal" alone (unreliable under injection).
- Regex output deletion (`code-old` anti-pattern).

## Enterprise

Canonical policy field: `network.blockWebFetch: true` for air-gapped profiles.

## Phase 2 attack corpus

Fixtures in `tests/fixtures/security/` cover repository/rule poisoning, tool and
terminal output, MCP descriptions/results, web citations, shared messages,
memory and summary poisoning, OCR/alt text, encoded delimiters, Unicode
controls, nested formats, and instructions to exfiltrate through tool arguments.
Every case links T-001 or T-016, C-005, and an expected decision.

## Related documents

- [01-threat-model.md](01-threat-model.md)
- [02-sandbox-model.md](02-sandbox-model.md)
