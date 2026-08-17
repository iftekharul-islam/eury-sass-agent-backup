# Multimodal and Attachment Specification

Spec-Version: 1.0.0

This normative specification covers user attachments, workspace images, vision input, generated images, and their lifecycle. It does not change the [tool catalog](02-tool-catalog-spec.md): `read_image` and `generate_image` remain the only visual-media tools.

## Invariants

| # | Requirement |
|---|---|
| M1 | Raw attachment bytes MUST remain local or in the configured Agent attachment service; they MUST NOT be embedded in prompts, events, logs, or audit records. |
| M2 | A provider receives an image only after model capability, effective policy, data residency, and size validation succeed. |
| M3 | Attachments, OCR, web-derived media, and generated images are untrusted content when assembled into a prompt. |
| M4 | A generated image MUST NOT write to the workspace until the user approves a separate `write_file` operation. |
| M5 | Metadata is stripped before provider delivery; original bytes are retained only with explicit user import consent. |
| M6 | Deleting a conversation schedules its non-pinned attachments for encrypted-store deletion according to retention policy. |

## Attachment record

```typescript
interface AttachmentRecord {
  id: string;                       // UUIDv7
  ownerId: string;
  conversationId?: string;
  source: "paste" | "drop" | "workspace" | "web" | "generated";
  mediaType: "image/png" | "image/jpeg" | "image/webp" | "image/gif" | "image/avif";
  bytes: number;                    // original accepted input
  width: number;
  height: number;
  sha256: string;
  encryptedBlobRef: string;
  provider?: string;                // generated or transformed only
  promptHash?: string;              // generated only; no prompt text
  trust: "untrusted";
  createdAt: string;
  expiresAt?: string;
}
```

`encryptedBlobRef` is local encrypted SQLite/blob storage for local and BYOK use. Managed routing uses a short-lived, single-use Agent attachment reference; the cloud service stores encrypted bytes only for the configured retention period and never exposes a general public URL.

## Ingestion and validation

| Concern | Rule |
|---|---|
| Formats | PNG, JPEG, WebP, AVIF, GIF first frame only |
| Size | ≤ 20 MB input, ≤ 8,192 px per side, ≤ 40 megapixels decoded |
| Count | ≤ 4 images per user turn and ≤ 10 per run |
| Processing | Decode and downscale locally when needed; preserve aspect ratio; record the downscale in the tool result |
| Metadata | Strip EXIF, XMP, GPS, comments, and embedded thumbnails before provider delivery |
| Failure | Unsupported media → `EURY_ATTACHMENT_MEDIA_UNSUPPORTED`; malformed/decompression-bomb input → `EURY_ATTACHMENT_INVALID`; over-limit input → `EURY_ATTACHMENT_TOO_LARGE` |
| Accessibility | UI requires editable alt text for a user-exported generated image; generated previews receive provider/model alt text plus dimensions |

An attachment is virus-scanned where an enterprise policy requires it. Scan failure blocks provider delivery but does not destroy the original local import; the user receives a retry/remove choice.

## Provider adaptation

The runtime resolves an attachment ID to provider-native image parts only at request assembly. `supportsVision`, `maxImageBytes`, and `maxImageCount` from `GET /agent/v1/models` are hard limits. A selected model without vision returns `EURY_MODEL_VISION_UNSUPPORTED` before any network call. The UI preserves the attachment and offers a compatible model picker.

For managed routes, the desktop sends an attachment reference and integrity hash, never base64 image data. The gateway verifies ownership, hash, expiry, organization policy, and residency before requesting the provider.

## OCR and visual observations

OCR is a derived visual observation, not trusted file content. It MAY be returned with normalized coordinate regions, but it MUST NOT create a file, command, URL action, policy exception, or tool call without independent model reasoning and the ordinary tool-policy decision.

## Generated images

`generate_image` requires a capability-enabled provider and policy permission. The first request in a run shows provider, number of images, dimensions/aspect ratio, and estimated cost. A stricter organization policy may require approval for every request.

Generated outputs are stored as `source: "generated"` attachment records. The transcript gallery shows provider, dimensions, generated time, alt text, and actions: Copy, Download, and Save to project. Download exports a user-selected copy; Save to project invokes the regular workspace-write flow with target path, diff/metadata, and approval.

## Retention, export, and audit

| Data | Default |
|---|---|
| Local attachment blob | Encrypted; removed with conversation deletion unless pinned |
| Managed temporary provider reference | Single-use; expires after 15 minutes |
| Generated output | Conversation retention; policy may shorten it |
| Audit event | Attachment ID, source, media type, bytes bucket, model/provider, policy decision, content hash; never pixels, OCR text, prompt text, or file path |
| Export | User-visible warning when an export moves content outside the encrypted store |

## Conformance tests

| ID | Test |
|---|---|
| AT-01 | Every accepted format reaches provider adaptation with metadata stripped and integrity hash unchanged. |
| AT-02 | Oversize, malformed, and decompression-bomb images fail before provider/network access. |
| AT-03 | A non-vision model rejects an image request without spending provider credits. |
| AT-04 | Managed requests send a reference rather than raw image bytes. |
| AT-05 | Generated output does not create a workspace file until the ordinary write approval succeeds. |
| AT-06 | Conversation deletion, expiry, and pinning obey encrypted-store retention rules. |
| AT-07 | Audit/log fixtures prove pixels, OCR, prompts, and paths never leave the allowed data stores. |

## Related documents

- [Agent runtime](01-agent-runtime-spec.md)
- [Tool catalog](02-tool-catalog-spec.md)
- [Cloud API contract](06-cloud-api-contract.md)
- [Privacy and data residency](../03-security/07-privacy-and-data-residency.md)
- [Chat UX](../05-ui/03-chat-and-streaming-ux.md)
