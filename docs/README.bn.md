# Eury Agent — সংক্ষিপ্ত বিবরণ (বাংলা)

Spec-Version: 1.2.0

## Eury Agent কী?

Eury Agent হলো Eury প্ল্যাটফর্মের জন্য একটি **ডেস্কটপ কোডিং এজেন্ট** — Cursor বা Claude Code-এর মতো, কিন্তু আমাদের নিজস্ব প্রোডাক্ট। পুরনো PySide6 অ্যাপ (`code-old/`) বাতিল; নতুন করে **Tauri + React + Rust** দিয়ে বানানো হচ্ছে।

## মূল সিদ্ধান্ত

| বিষয় | সিদ্ধান্ত |
|---|---|
| এজেন্ট ইঞ্জিন | [Cersei](https://cersei.tryatlas.cc/docs) — Rust SDK, ডেস্কটপ অ্যাপের **ভিতরেই** embedded |
| আলাদা সার্ভার | না — latency কম রাখতে এজেন্ট লুপ পুরোটাই local |
| UI | Tauri 2 + React 19 + TypeScript + Tailwind 4 |
| লোকাল ডেটা | Encrypted SQLite; API key শুধু OS keychain-এ |
| ক্লাউড | NestJS (`backend/`) — শুধু login, billing, model gateway, policy, audit, sync |
| মডেল | BYOK (নিজের key, সবচেয়ে দ্রুত) অথবা managed gateway (org quota + audit) |
| নিরাপত্তা | প্রতিটি write / shell / network টুলে আগে থেকে অনুমতি লাগবে (deny by default) |

## কী করবে

- প্রজেক্ট ওপেন করে AI দিয়ে কোড পড়া, লেখা, রিফ্যাক্টর, টেস্ট ফিক্স
- পাঁচটি মোড: Chat / Agent / Plan / Ask / Build — প্রতিটির আলাদা permission
- এডিটরে **live write preview** — এজেন্ট কী বদলাবে, apply করার আগেই দেখা যাবে
- Hunk ধরে ধরে apply বা skip; যেকোনো turn এক ক্লিকে rollback (checkpoint)
- টার্মিনাল (PTY), Git, ব্রাউজার প্রিভিউ
- প্রজেক্ট মেমোরি (`EURY.md`) ও স্মার্ট কনটেক্সট (লোকাল index)
- MCP সার্ভার সাপোর্ট (অনুমোদিত সার্ভার কেবল)
- টিম ও এন্টারপ্রাইজ: SSO, SCIM, policy, audit log, quota ও budget

## পুরনো `code` স্ট্যাক থেকে সম্পূর্ণ আলাদা

নতুন অ্যাপের কোনো কিছুই পুরনোটার সাথে শেয়ার করা হবে না — কোনো conflict হবে না।

| জিনিস | পুরনো | নতুন |
|---|---|---|
| API | `/code/*`, `/auth/ide/*` | `/agent/v1/*` (নিজের auth সহ) |
| ব্যাকএন্ড কোড | `modules/code/` | `modules/agent/` — অন্য কোনো module import করবে না |
| ডেটাবেস | `DesktopRelease`, `IdeAuthSession` | শুধু `Agent`-prefix করা টেবিল |
| লোকাল ফোল্ডার | `~/.eury-code/` | `com.eury.agent` app-data ফোল্ডার |
| ইনস্টলার | `Eury-Code-*` | `Eury-Agent-*` |
| Env var | `CODE_*`, `EURY_CODE_*` | `AGENT_*`, `EURY_AGENT_*` |

ব্যাকএন্ডে শুধু **একটি module** যোগ হবে — কোনো পুরনো service বা controller ছোঁয়া হবে না। পুরো তালিকা: [naming and migration map](00-overview/05-naming-and-migration-map.md)।

## ডকুমেন্টেশন কোথায়

ইংরেজি টেকনিক্যাল ডক: **[00-index.md](00-index.md)** — সম্পূর্ণ ডকুমেন্টেশন ম্যাপ, `00-overview` থেকে `09-roadmap` পর্যন্ত।

| ফোল্ডার | কী আছে |
|---|---|
| `00-overview` | ভিশন, শব্দকোষ, নামকরণের নিয়ম |
| `01-product` | ফিচার তালিকা, মোড, প্রাইসিং |
| `02-architecture` | সিস্টেম ডিজাইন + ১০টি ADR |
| `03-security` | threat model, sandbox, permission, privacy |
| `04-specs` | ১৭টি implementation-ready contract |
| `05-ui` | ডিজাইন সিস্টেম ও UX |
| `06-enterprise` | SSO, RBAC, policy, audit, quota |
| `07-ops` | config, CI/CD, packaging, update, runbook |
| `08-quality` | টেস্ট, eval, benchmark, definition of done |
| `09-roadmap` | ৩০টি ফেজের বিস্তারিত পরিকল্পনা |

## বর্তমান অবস্থা

**Phase 0 foundation, Phase 1 product contract, Phase 2 security foundation, Phase 3 desktop shell, Phase 4 agent core, Phase 5 workspace and sandbox, Phase 6 tool layer v1, Phase 7 policy and approval system, Phase 8 chat experience, Phase 9 local persistence, এবং Phase 10 identity সম্পূর্ণ ও যাচাইকৃত।** এখন এজেন্ট নিজস্ব অথেন্টিকেশন স্ট্যাক এবং সিকিউর কেইচেইন স্টোরেজ ব্যবহার করে। পরের কাজ Phase 11 network gateway। প্রতিটি ফেজে নির্দিষ্ট exit criteria আছে।

| মাইলস্টোন | ফেজ | কী পাওয়া যাবে |
|---|---|---|
| M1 Alpha | 9 | চ্যাট + এজেন্ট + টুল + persistence (ইন্টারনাল) |
| M2 Beta | 17 | প্ল্যান, মেমোরি, এডিটর, Git |
| M3 RC | 23 | sync, preview, MCP |
| M4 GA | 29 | এন্টারপ্রাইজ ফিচার + signed installer |

## পুরনো অ্যাপ থেকে কী রাখা হবে, কী বাদ

**রাখা হবে:** live write preview, tool activity timeline, plan-as-markdown, PKCE browser login, PTY terminal bridge।

**বাদ দেওয়া হবে:** markdown fence থেকে tool call parse করা, regex দিয়ে মডেলের আউটপুট মুছে ফেলা, plaintext-এ টোকেন রাখা, আর `CODE_API_TOKEN` না থাকলে API খুলে দেওয়ার বাগ।
