## User

Work on onde repo feature/decouple-gresiq-from-onde-inference branch. Use /gh-stack continue working on **M4 geography**: the public `/pulse` globe still reads IP-geolocated regions from the platform. That's the largest Onde-owned piece left and spec 0007 deliberately excludes it pending a separate geography spec, mostly because it's a privacy and retention decision before it's an engineering one.

## Assistant

I'll start by getting oriented in the onde repo and loading the gh-stack skill.

**Tool Call: cd /Users/setoelkahfi/Repositories/onde && git status --short | head -30 && echo "--- BRANCH ---" && git branch --show-current && echo "--- LOG ---" && git log --oneline -15**
Status: Completed

Terminal:
```
--- BRANCH ---
feature/deps
--- LOG ---
019cd894 v1.2.3
a9f41644 Merge pull request #11 from ondeinference/feature/sdk-dart-android-v2
cb342599 Fix NDK version comparison and native lib task ordering
ebf77159 Build the Dart SDK's Android libs with cargo-ndk
84c6e528 Merge branch 'development' of gh-keypair34:ondeinference/onde into development
5cc7b9e9 Merge pull request #10 from ondeinference/feature/add-UQFF-support
6051fd1a Fix sysctl-rs deps
14a1d113 Fix ci
5a6615af Add UQFF model format
c8972c02 Merge branch 'development' into feature/add-UQFF-support
5d0f032c Merge branch 'feature/deps' into development
3d7eca58 Update deps
a495aab0 Merge branch 'development' into feature/add-UQFF-support
30272293 Merge branch 'feature/release-improvement' into development
2ae59c59 Fix dart example
```


**Tool Call: Load skill: gh-stack**
Status: Completed

Launching skill: gh-stack

**Tool Call: git branch -a | head -50 && echo "=== REMOTES ===" && git remote -v && echo "=== SPECS ===" && ls -d specs* .specify 2>/dev/null; find . -maxdepth 3 -type d -name "*spec*" -not -path "./target/*" -not -path "./node_modules/*" 2>/dev/null | head**
Status: Completed

Terminal:
```
  development
+ feature/decouple-gresiq-from-onde-inference
* feature/deps
  feature/diffusion-vision
  feature/v1.0.0-chat-transcribe-demucs-rs
  feature/v1.0.0-chat-transcribe-demucs-rs-DeepSeek-Coder-V2-Lite-gguf
  main
  remotes/gh-keypair34/HEAD -> gh-keypair34/main
  remotes/gh-keypair34/claude/zen-feynman-0u78dk
  remotes/gh-keypair34/copilot/assess-inference-engine-complexity
  remotes/gh-keypair34/copilot/fix-onde-npm-sdk-v1-1-2-release
  remotes/gh-keypair34/development
  remotes/gh-keypair34/feature/ci
  remotes/gh-keypair34/feature/deps
  remotes/gh-keypair34/feature/diffusion-vision
  remotes/gh-keypair34/feature/release-v0.1.8
  remotes/gh-keypair34/feature/release-v1
  remotes/gh-keypair34/feature/sdk-dart
  remotes/gh-keypair34/feature/sdk-dart-android-v2
  remotes/gh-keypair34/feature/sdk-kotlin
  remotes/gh-keypair34/feature/sdk-kotlin-kmm-ios
  remotes/gh-keypair34/feature/sdk-rust
  remotes/gh-keypair34/feature/sdk-swift
  remotes/gh-keypair34/feature/swift-release-1.1.1-recovery
  remotes/gh-keypair34/main
  remotes/gh-setoelkahfi/claude/qwen-3-support-d1sxiw
  remotes/gh-setoelkahfi/claude/zen-feynman-0u78dk
  remotes/gh-setoelkahfi/copilot/assess-inference-engine-complexity
  remotes/gh-setoelkahfi/copilot/fix-onde-npm-sdk-v1-1-2-release
  remotes/gh-setoelkahfi/development
  remotes/gh-setoelkahfi/feature/add-UQFF-support
  remotes/gh-setoelkahfi/feature/ci
  remotes/gh-setoelkahfi/feature/decouple-gresiq-from-onde-inference
  remotes/gh-setoelkahfi/feature/deps
  remotes/gh-setoelkahfi/feature/diffusion-vision
  remotes/gh-setoelkahfi/feature/release-v0.1.8
  remotes/gh-setoelkahfi/feature/release-v1
  remotes/gh-setoelkahfi/feature/sdk-dart
  remotes/gh-setoelkahfi/feature/sdk-dart-android-v2
  remotes/gh-setoelkahfi/feature/sdk-kotlin
  remotes/gh-setoelkahfi/feature/sdk-kotlin-kmm-ios
  remotes/gh-setoelkahfi/feature/sdk-rust
  remotes/gh-setoelkahfi/feature/sdk-swift
  remotes/gh-setoelkahfi/feature/swift-release-1.1.1-recovery
  remotes/gh-setoelkahfi/fix/visionos-onde-tqdm
  remotes/gh-setoelkahfi/main
=== REMOTES ===
gh-keypair34	gh-keypair34:ondeinference/onde.git (fetch)
gh-keypair34	gh-keypair34:ondeinference/onde.git (push)
gh-setoelkahfi	git@github.com:ondeinference/onde.git (fetch)
gh-setoelkahfi	git@github.com:ondeinference/onde.git (push)
=== SPECS ===
(eval):1: no matches found: specs*
```


**Tool Call: git log --oneline development..feature/decouple-gresiq-from-onde-inference | head -30; echo "=== diff stat ==="; git diff --stat development...feature/decouple-gresiq-from-onde-inference | tail -30**
Status: Completed

Terminal:
```
69f2e767 feat(pulse): measure TTFT and stamp edge liveness in dual-write
952dbde0 fix(pulse): harden GresIQ document dual-write rollout
f59e9fc8 feat(pulse): dual-write telemetry to GresIQ document gateway (ADR-0003 M2)
=== diff stat ===
 Cargo.lock              |  12 +--
 Cargo.toml              |   4 +-
 src/inference/engine.rs |  66 +++++++++++-
 src/pulse/client.rs     | 260 +++++++++++++++++++++++++++++++++++++++++++++++-
 src/pulse/events.rs     |  69 +++++++++++++
 5 files changed, 398 insertions(+), 13 deletions(-)
```


**Tool Call: for r in onde-cloud onde-web onde-klepon onde-swift; do echo "=== $r ==="; ls /Users/setoelkahfi/Repositories/$r/specs 2>/dev/null | head -20; done**
Status: Completed

Terminal:
```
=== onde-cloud ===
=== onde-web ===
=== onde-klepon ===
=== onde-swift ===


```


**Tool Call: cd /Users/setoelkahfi/Repositories && for r in onde onde-cloud onde-web onde-klepon; do echo "=== $r ==="; find $r -iname "*0007*" -o -iname "*ADR-0003*" -o -iname "*adr*0003*" 2>/dev/null | grep -v node_modules | grep -v "/target/" | head; done**
Status: Completed

Terminal:
```
=== onde ===
=== onde-cloud ===
=== onde-web ===
onde-web/.agents/specs/0007-pulse-document-read-migration.md
=== onde-klepon ===

Shell cwd was reset to /Users/setoelkahfi/Repositories/onde-swift
```


**Tool Call: ls /Users/setoelkahfi/Repositories/onde-web/.agents/specs/ /Users/setoelkahfi/Repositories/onde-web/.agents/ 2>/dev/null; echo "=== git ==="; git -C /Users/setoelkahfi/Repositories/onde-web log --oneline -5; git -C /Users/setoelkahfi/Repositories/onde-web branch --show-current; git -C /Users/setoelkahfi/Repositories/onde-web status --short | head**
Status: Completed

Terminal:
```
/Users/setoelkahfi/Repositories/onde-web/.agents/:
AGENTS.md
memory
skills
specs

/Users/setoelkahfi/Repositories/onde-web/.agents/specs/:
0001-onde-cloud-platform.md
0002-onde-app-id-pulse-events.md
0003-offline-first-on-device-model-loading.md
0004-sdk-example-apple-signing-strategy.md
0005-session-expired-on-root-models.md
0006-cloud-model-availability-controls.md
0007-pulse-document-read-migration.md
README.md
TEMPLATE.md
=== git ===
0097920 feat(pulse): page document reads by cursor
076aac6 feat(pulse): derive edge liveness from recency, not a stale status
68bbd05 feat(pulse): read app telemetry from GresIQ documents
10512e7 docs(pulse): specify document read migration
960d465 Merge branch 'development' into feature/pulse-migration-gresiq-document
feature/pulse-migration-gresiq-document
```


**Tool Call: cat /Users/setoelkahfi/Repositories/onde-web/.agents/specs/0007-pulse-document-read-migration.md**
Status: Completed

Terminal:
```
# Onde Web Pulse Document Read Migration

| Field | Value |
| --- | --- |
| Spec | 0007 |
| Status | Read boundary implemented; document mode not enabled |
| Owner | Onde Web / Forward-deployed engineering |
| Created | 2026-09-04 |
| Updated | 2026-09-04 |
| Repos | onde-web, onde, smbcloud-api |
| Related | [[0002-onde-app-id-pulse-events]], smbcloud-api `docs/adr-0003.md` (M1–M5) |

## Summary

Move Onde product-specific Pulse reads and aggregation into Onde Web while
keeping GresIQ a tenant-agnostic document service. The first cutover is the
authenticated, per-app dashboard summary; the public geographic Pulse view is
not part of this migration until Onde owns a location-enrichment design.

## Background / Problem

The legacy dashboard reads Onde-specific `pulse_*` relational resources from
GresIQ. That makes a shared smbCloud service understand Onde's telemetry
schema, query model, and product identity.

ADR-0003 M1 supplies a generic document gateway, and the SDK M2 branch can
mirror the following opaque documents when both `ONDE_PULSE_DUAL_WRITE` is
explicitly enabled and an Onde app ID is supplied:

| Collection | Key | Required product fields |
| --- | --- | --- |
| `pulse_edges` | `<onde_app_id>:<edge_id>` | `onde_app_id`, `edge_id`, `status` |
| `pulse_models` | `<onde_app_id>:<model_id>` | `onde_app_id`, `model_id`, `model_name` |
| `pulse_deployments` | `<onde_app_id>:<edge_id>:<model_id>` | app, edge, model, load duration, residency state |
| `pulse_inference_events` | `<onde_app_id>:<request_id>` | app, edge, model, request ID, duration, status |

The documents are intentionally opaque to GresIQ. Onde Web must therefore
authenticate the user, scope reads to the selected Onde app, decode the
documents, and construct the existing UI data shape. The existing
`normalizePulseData` boundary in `app/pulse/pulse-data.ts` remains the only
camel-case rendering boundary.

## Goals

- Add a server-only, app-scoped document reader for the authenticated Onde app
  dashboard.
- Introduce shadow-read parity checks before a user-visible source switch.
- Make the source selection reversible and fail closed to legacy Pulse data.
- Keep GresIQ generic: no Onde-specific routes, tables, document interpretation,
  or browser-exposed GresIQ credentials.

## Non-Goals

- Replacing the public `/pulse` globe, country map, or regional rollups.
- Deriving geography from IP addresses in GresIQ or storing raw IP addresses in
  the generic document gateway.
- Deleting legacy `pulse_*` resources, backfilling un-attributed events, or M5
  retirement.
- Changing cloud-model availability controls or billing behavior.

## Requirements

- Onde Web must query document collections only from server code using the
  selected app's GresIQ data-plane credentials; those credentials must never be
  returned to a browser.
- Every query and every assembled record must be constrained by an exact
  `onde_app_id`. A document missing that field is ineligible, even if its key
  happens to share an app-ID prefix.
- The reader must preserve the existing rendered `PulseData` contract through
  `normalizePulseData` or a tested adapter immediately before it.
- `ONDE_PULSE_DOCUMENT_READ` must be server-only, disabled by default, and
  select document data only after the app passes parity gates. A read failure,
  malformed document, missing credentials, or incomplete document set must use
  legacy data rather than render invented zeroes.
- The reader must page the generic list API by cursor until it is exhausted,
  and must treat a walk that stops early — at its own safety cap — as an
  incomplete read rather than a whole window.
- Shadow comparison telemetry must contain counts and bounded latency/error
  categories only—never data-plane secrets, request text, raw IPs, or event
  payloads.

## Design / Approach

### Read boundary

Add a server-side product reader, for example
`app/pulse/document-pulse-data.ts`, and an app-scoped route such as
`app/api/gresiq/apps/[appId]/pulse/data/route.ts`. The route authorizes access
to `appId` using the same app-dashboard policy as existing protected endpoints,
then passes the resolved GresIQ application credentials to the reader. It must
not reuse a global unscoped `/api/gresiq/pulse/data` response for an app page.

The reader lists each collection with an exact top-level document filter for
`onde_app_id`, validates collection-specific fields, and converts valid
documents into an Onde-owned intermediate model. A final adapter produces the
legacy `PulseData` shape and calls `normalizePulseData` before rendering.

### Source modes

| Mode | User response | Secondary work |
| --- | --- | --- |
| `legacy` (default) | Existing relational response | None |
| `shadow` | Existing relational response | Fetch/assemble document data; emit parity metrics |
| `document` | Document response for an allowlisted app | Fall back to legacy on any completeness/error gate |

Feature configuration must be evaluated only on the server. `document` is
per-app allowlisted, not a global flip, so a bad tenant dataset has a contained
rollback.

### Parity and completeness

For a fixed app and measurement window, compare legacy and document results:

- distinct edges and models;
- deployments by `(edge_id, model_id)` and residency state;
- inference-event counts by `(model_id, status)`;
- oldest/newest source timestamps and the number of rejected documents.

The expected M2 document set has no geographic fields. Geographic dimensions
are excluded from comparison and block only the public Pulse migration, not the
per-app summary cutover.

TTFT is excluded for a different reason: the relational route stores a
hardcoded `ttft_ms: 0`, so legacy latency figures are aggregates over zeros
rather than measurements. The SDK now measures TTFT on the streaming path only
and omits the field elsewhere, so the two sources are not comparable on latency
and the document side is the more truthful of the two.

Missing app attribution, a truncated list, or a document decode failure makes
the comparison inconclusive—not equal.

### Cross-repository dependency

Before enabling the first app, deploy the SDK M2 dual-write for that app with a
stable per-installation edge ID and `ChatEngine::with_app_id` (or `ONDE_APP_ID`).
The gateway provides cursor pagination as of the ADR-0003 M1 extension, so the
reader can walk an entire window. It stayed a generic platform capability: no
Onde-specific interpretation entered GresIQ.

## Implementation Plan

- [ ] Confirm one non-production Onde app emits all four M2 collections with
  explicit app IDs and stable edge IDs. Blocked on shipping, not on code: the
  dual-write lives on `onde`'s `feature/decouple-gresiq-from-onde-inference`
  and no released SDK carries it yet.
- [x] Add typed document-envelope and collection decoders in Onde Web, including
  rejection accounting for malformed or unscoped documents.
  (`types/pulse-documents.ts`, `app/pulse/pulse-documents.ts`)
- [x] Implement the authorized app-scoped server reader and adapter to the
  existing `PulseData` rendering contract.
  (`app/pulse/document-pulse-data.ts`,
  `app/api/gresiq/apps/[appId]/pulse/data/route.ts`; the app dashboard now reads
  that route instead of the global proxy)
- [x] Add a server-only `legacy` / `shadow` / per-app `document` source policy;
  retain the legacy endpoint as the fallback. (`app/pulse/pulse-source.ts`,
  `ONDE_PULSE_DOCUMENT_READ` + `ONDE_PULSE_DOCUMENT_APPS`)
- [x] Add cursor pagination or a versioned product-owned aggregate capability
  before asserting full-window parity or enabling `document` in production.
  The gateway now returns `next_cursor` (keyset on `(order column, id)`) and the
  reader pages until it is null. A per-collection safety cap still reports
  `result_truncated` so an unbounded walk degrades to legacy instead of hanging
  on a request path.
- [ ] Run shadow mode for a bounded pilot, record parity/error/latency metrics,
  and define an app-specific pass threshold.
- [ ] Enable document reads for the pilot app, then expand only after the same
  gates pass for each app.

## Acceptance Criteria

- [x] Browser network traffic contains no GresIQ data-plane credential.
- [x] An authenticated app dashboard cannot receive documents belonging to a
  different `onde_app_id`. Enforced twice: the gateway query filters on
  `onde_app_id`, and every decoder re-checks the field per document.
- [x] Decoder and adapter tests cover valid documents, app/key mismatch,
  malformed payloads, duplicate inference requests, and legacy fallback.
  (`pnpm test`, vitest)
- [x] Shadow mode cannot change the user-visible response.
- [x] A document read error or incomplete result returns the current legacy
  response and emits a bounded diagnostic category. A walk that stops before
  the gateway runs out of rows counts as incomplete.
- [ ] Production document mode is enabled only for app allowlists that satisfy
  the agreed completeness and parity thresholds.
- [ ] `/pulse` geographic views still use their current source until a separate
  product-owned geography spec is accepted.

## Open Questions

- ~~Which component should expose cursor pagination: the generic document
  gateway or an Onde-owned aggregate publisher?~~ **Resolved: the gateway.**
  Paging is a storage concern, not a product one, and every future tenant needs
  it; an Onde-owned aggregate publisher would have put a product-shaped
  workaround on top of a generic gap.
- What historical window and numerical tolerance define parity for a pilot?
- Where should Onde retain coarse geographic aggregation, and what privacy and
  retention policy applies before migrating the public globe?
- Should document-mode configuration live in deployment environment variables,
  an authenticated admin setting, or both?

## Out of Scope

M5 legacy retirement and all destructive cleanup are deferred until document
mode has operated successfully through a full retention period and a backfill
decision exists.

## References

- `smbcloud/services/smbcloud-api/docs/adr-0003.md`
- `onde/src/pulse/client.rs` on `feature/decouple-gresiq-from-onde-inference`
- `onde-web/app/pulse/pulse-data.ts`
- `onde-web/app/api/gresiq/pulse/data/route.ts`
```


**Tool Call: sed -n '1,120p' /Users/setoelkahfi/Repositories/smbcloud/services/smbcloud-api/docs/adr-0003.md**
Status: Completed

Terminal:
```
# ADR-0003: Make GresIQ an Application-Agnostic Managed-Data Service

## Status

**M0 + M1 shipped, M1 extended with cursor pagination. M2 implemented in the
product tier (unreleased). M3–M5 parked.**

Direction accepted; M0 resolved (Project-owned). **M1 document core is now
SHIPPED and live** (schema version `2026_06_28_130000`): `gresiq_documents` table
migrated, `GresiqDocument` model, and the generic gateway
`Gresiq::V1::DocumentsController` (CRUD + containment filter + aggregate) wired at
`/gresiq/v1/collections/:collection`, authenticated by the Project app's
api_key/secret (`GresiqGatewayAuthenticatable`). Fully additive — it coexists with
the live `gresiq_ondeinference_*` pulse tables, which still serve Pulse until the
M5 cut-over. The FK cascade gap from the initial migration was corrected in
`20260628130000_add_cascade_to_gresiq_documents_app_fk`.

The document list API now returns `next_cursor` and accepts `cursor`, using
keyset pagination on `(<order column>, id)`. Offset paging was not an option: a
product reading a whole window while documents are still being written would
have rows shift under it between pages. A `nil` cursor is the platform's
statement that the caller has seen every matching document, which is what lets
a product distinguish a complete read from a truncated one.

**M2 is implemented but unreleased:** `onde` dual-writes the four `pulse_*`
collections behind `ONDE_PULSE_DUAL_WRITE` on
`feature/decouple-gresiq-from-onde-inference`, and `onde-web` has an app-scoped
document reader behind `ONDE_PULSE_DOCUMENT_READ`. Neither is enabled for any
app, and no SDK release carries the dual-write yet.

**Still parked (M3–M5):** promotion + soft-ref validation, externalizing Onde's
rollups/dashboards, and the backfill/cut-over/retire of the
`gresiq_ondeinference_*` tables. The remainder is a multi-quarter, 4-repo effort
(see Complexity assessment).

> **Note:** the *product* layers on top of this document core — realtime, RLS +
> BYO-issuer identity, client SDKs, vector/RAG — are tracked separately in
> **ADR-0005** (BaaS surface, modularity) and **ADR-0006** (pgvector), with the
> auth seam specced in `docs/gresiq-identity-contract.md`. None of those are
> started yet.

Why parked, not dropped: it is the agreed top-priority direction for making GresIQ
application-agnostic; the cost/sequencing just makes it a planned program, not an
in-flight change.

**Resolved (M0, principal model):** A GresIQ app is owned by a **Project**. It may
*store* `AuthApp` / `AuthUser` rows as application data; because GresIQ and Auth
live in the same service and database, those references are **real foreign keys**,
not opaque external IDs. There is therefore exactly one app-ownership concept
(`gresiq_apps` → `Project`); the former Onde end-user registry
(`gresiq_ondeinference_apps`) collapses into app-defined *data* inside the
Project's GresIQ schema. See "Resolved principal model" below.

## Context

GresIQ is described (in the Rust `smbcloud-gresiq-sdk` crate) as:

> a managed-database layer inside smbCloud. It sits in front of a PostgreSQL
> database, adds API-key auth, and exposes a simple REST interface for
> inserting and querying rows. **Schema knowledge (which tables exist, what the
> rows look like) lives in the caller.**

That is the intended contract: GresIQ is the *platform data layer*; downstream
products (Onde Inference, FatApp, siGit) are *tenants* of it and own their own
schemas. The smbCloud → Onde Inference → siGit stack is vertically integrated,
but the layers are supposed to be **modular**: GresIQ must not know what an
"edge", a "model", or a "TTFT" is.

The current Rails implementation violates that contract. Onde Inference's entire
telemetry domain — schema, ingest semantics, geolocation, rollups, and dashboard
aggregation — is baked into the platform. This ADR records the deep-research
findings and assesses the complexity of making GresIQ genuinely agnostic.

## Findings: the current GresIQ subsystem

GresIQ today is **three subsystems wearing one name**, and Onde is welded into
all three.

### Subsystem 1 — Generic gateway registry (`gresiq_apps`) — *partly agnostic*

`gresiq_apps` (`tenant_id`, `project_id`, `name`, `table_prefix`, `api_key`,
`api_secret_digest`, `status`) is the one genuinely generic piece. It models
"an application that stores data in GresIQ", scoped to a Project, authenticated
by `X-Gresiq-Api-Key` + `X-Gresiq-Api-Secret` (BCrypt). `GresiqApp#table_name_for`
composes physical names as `gresiq_<table_prefix>_<short_name>`, and the
controllers reach those tables through anonymous `Class.new(ApplicationRecord)`
subclasses. **This is the right primitive** and should survive.

Two gaps:

- **No provisioning.** Nothing creates the per-app physical tables. The design
  *implies* dynamic per-app schemas, but in practice the Onde tables were
  hand-written in a migration (`20260601000001_create_ondeinference_pulse_tables.rb`).
  The generic prefix mechanism is half-built — routing/auth is prefix-aware, DDL
  is not.
- **It is bypassed by Subsystem 2.** The user-facing dashboard and the SDK do
  *not* use `gresiq_apps` as the app identity; they use a second, parallel
  registry (below). The telemetry data even foreign-keys to that second one.

### Subsystem 2 — Onde-specific app registry (`gresiq_ondeinference_apps`) — *fully coupled*

A second "app" concept, scoped to a **different principal**:

| | `gresiq_apps` | `gresiq_ondeinference_apps` |
|---|---|---|
| Owner principal | Tenant + Project (platform operator) | AuthUser + AuthApp (Onde end user) |
| Identity | `api_key` | UUID PK = `app_id` |
| Secret | BCrypt digest | UUID PK of `gresiq_ondeinference_app_secrets` row |
| Consumed by | ingest gateway, M2M validate | dashboard, SDK `model_config` |

So "a GresIQ application" is modelled **twice**, incompatibly, and the name
`ondeinference` is hardcoded into the table, model, controller, and route names
(`GresiqOndeinferenceApp`, `/v1/client/gresiq/...`, `OndeSdkController`,
`OndeinferenceValidateAppController`).

### Subsystem 3 — Onde's telemetry domain, living in the platform — *fully coupled*

The actual data tables are Onde's application schema, physically named
`gresiq_ondeinference_*` and created by platform migrations:
```


**Tool Call: cd /Users/setoelkahfi/Repositories/onde-web && ls app/pulse/ && echo "=== api gresiq ===" && find app/api/gresiq -type f | head -20 && echo "=== geo grep ===" && grep -rniE "country|region|latitude|longitude|geo|ip_address|city" --include=*.ts --include=*.tsx -l app types lib 2>/dev/null | head -30**
Status: Completed

Terminal:
```
document-pulse-data.test.ts
document-pulse-data.ts
globe-view.tsx
page.tsx
pulse-data.ts
pulse-documents.test.ts
pulse-documents.ts
pulse-scene.tsx
pulse-source.test.ts
pulse-source.ts
span-selector.tsx
=== api gresiq ===
app/api/gresiq/models/route.ts
app/api/gresiq/apps/route.ts
app/api/gresiq/pulse/data/route.ts
app/api/gresiq/apps/[appId]/route.ts
app/api/gresiq/apps/[appId]/cloud-models/route.ts
app/api/gresiq/apps/[appId]/model/route.ts
app/api/gresiq/apps/[appId]/pulse/data/route.ts
=== geo grep ===
(eval):1: no matches found: --include=*.ts
```


**Tool Call: grep -rniE "country|region|latitude|longitude|geo|city" -l app types lib 2>/dev/null | head -30; echo "=== pulse-data.ts ==="; wc -l app/pulse/*.ts app/pulse/*.tsx app/api/gresiq/pulse/data/route.ts**
Status: Completed

Terminal:
```
app/page.tsx
app/enterprise/page.tsx
app/pulse/pulse-data.ts
app/pulse/pulse-scene.tsx
app/pulse/pulse-source.ts
app/pulse/document-pulse-data.ts
app/about/page.tsx
app/pulse/globe-view.tsx
app/pulse/page.tsx
app/cloud/page.tsx
app/root/models/[modelId]/page.tsx
app/root/models/page.tsx
app/pulse/document-pulse-data.test.ts
app/globals.css
app/root/apps/[appId]/page.tsx
app/pulse-demo/page.tsx
app/billing/page.tsx
types/pulse.ts
=== pulse-data.ts ===
     307 app/pulse/document-pulse-data.test.ts
     361 app/pulse/document-pulse-data.ts
     398 app/pulse/pulse-data.ts
     140 app/pulse/pulse-documents.test.ts
     172 app/pulse/pulse-documents.ts
      55 app/pulse/pulse-source.test.ts
     107 app/pulse/pulse-source.ts
     294 app/pulse/globe-view.tsx
     267 app/pulse/page.tsx
     219 app/pulse/pulse-scene.tsx
      39 app/pulse/span-selector.tsx
      59 app/api/gresiq/pulse/data/route.ts
    2418 total
```


**Tool Call: cat app/pulse/pulse-data.ts**
Status: Completed

Terminal:
```
import type {
  PulseData,
  PulseModel,
  PulseRegion,
  PulseSummary,
  RawPulseData,
  RawPulseRecord,
} from "@/types/pulse";

export const placeholderPulseData: PulseData = {
  updatedAt: "2026-03-29T15:42:00.000Z",
  summary: {
    liveRegions: 12,
    activeEdges: 384,
    inferencesPerMinute: 18240,
    tokensPerSecond: 92844,
    medianTtftMs: 118,
    p95TtftMs: 204,
  },
  regions: [
    {
      id: "arn1",
      label: "Stockholm",
      code: "ARN1",
      lat: 59.3293,
      lon: 18.0686,
      activeEdges: 28,
      activationsPerMinute: 1180,
      inferencesPerMinute: 1760,
      ttftMs: 93,
      dominantModel: "Qwen 2.5 7B",
    },
    {
      id: "fra1",
      label: "Frankfurt",
      code: "FRA1",
      lat: 50.1109,
      lon: 8.6821,
      activeEdges: 34,
      activationsPerMinute: 1490,
      inferencesPerMinute: 2240,
      ttftMs: 102,
      dominantModel: "Llama 3.1 8B",
    },
    {
      id: "yyz1",
      label: "Toronto",
      code: "YYZ1",
      lat: 43.6532,
      lon: -79.3832,
      activeEdges: 26,
      activationsPerMinute: 1075,
      inferencesPerMinute: 1615,
      ttftMs: 108,
      dominantModel: "DeepSeek R1 Distill",
    },
    {
      id: "iad1",
      label: "Virginia",
      code: "IAD1",
      lat: 38.9072,
      lon: -77.0369,
      activeEdges: 48,
      activationsPerMinute: 1960,
      inferencesPerMinute: 2920,
      ttftMs: 121,
      dominantModel: "Llama 3.1 8B",
    },
    {
      id: "sjc1",
      label: "San Jose",
      code: "SJC1",
      lat: 37.3382,
      lon: -121.8863,
      activeEdges: 42,
      activationsPerMinute: 1820,
      inferencesPerMinute: 2680,
      ttftMs: 115,
      dominantModel: "Qwen 2.5 7B",
    },
    {
      id: "hnd1",
      label: "Tokyo",
      code: "HND1",
      lat: 35.6762,
      lon: 139.6503,
      activeEdges: 39,
      activationsPerMinute: 1715,
      inferencesPerMinute: 2540,
      ttftMs: 98,
      dominantModel: "Llama 3.2 3B",
    },
    {
      id: "mex1",
      label: "Mexico City",
      code: "MEX1",
      lat: 19.4326,
      lon: -99.1332,
      activeEdges: 24,
      activationsPerMinute: 955,
      inferencesPerMinute: 1420,
      ttftMs: 129,
      dominantModel: "Phi 4 Mini",
    },
    {
      id: "cgk1",
      label: "Jakarta",
      code: "CGK1",
      lat: -6.2088,
      lon: 106.8456,
      activeEdges: 33,
      activationsPerMinute: 1425,
      inferencesPerMinute: 2140,
      ttftMs: 113,
      dominantModel: "Qwen 2.5 7B",
    },
    {
      id: "jog1",
      label: "Yogyakarta",
      code: "JOG1",
      lat: -7.7971,
      lon: 110.3708,
      activeEdges: 22,
      activationsPerMinute: 930,
      inferencesPerMinute: 1410,
      ttftMs: 124,
      dominantModel: "DeepSeek R1 Distill",
    },
    {
      id: "ikn1",
      label: "Nusantara",
      code: "IKN1",
      lat: -0.9667,
      lon: 116.7089,
      activeEdges: 18,
      activationsPerMinute: 755,
      inferencesPerMinute: 1155,
      ttftMs: 139,
      dominantModel: "Phi 4 Mini",
    },
    {
      id: "kno1",
      label: "Medan",
      code: "KNO1",
      lat: 3.5952,
      lon: 98.6722,
      activeEdges: 17,
      activationsPerMinute: 690,
      inferencesPerMinute: 1025,
      ttftMs: 136,
      dominantModel: "DeepSeek R1 Distill",
    },
    {
      id: "sub1",
      label: "Surabaya",
      code: "SUB1",
      lat: -7.2575,
      lon: 112.7521,
      activeEdges: 20,
      activationsPerMinute: 812,
      inferencesPerMinute: 1225,
      ttftMs: 132,
      dominantModel: "Phi 4 Mini",
    },
  ],
  models: [
    {
      id: "qwen-2_5-7b",
      name: "Qwen 2.5 7B",
      modality: "Text",
      activeRegions: 8,
      activationsPerMinute: 5810,
      inferencesPerMinute: 8910,
      share: 0.31,
      ttftMs: 96,
    },
    {
      id: "llama-3_1-8b",
      name: "Llama 3.1 8B",
      modality: "Text",
      activeRegions: 6,
      activationsPerMinute: 4204,
      inferencesPerMinute: 6402,
      share: 0.24,
      ttftMs: 118,
    },
    {
      id: "deepseek-r1-distill",
      name: "DeepSeek R1 Distill",
      modality: "Reasoning",
      activeRegions: 5,
      activationsPerMinute: 3014,
      inferencesPerMinute: 4115,
      share: 0.18,
      ttftMs: 141,
    },
    {
      id: "phi-4-mini",
      name: "Phi 4 Mini",
      modality: "Compact",
      activeRegions: 4,
      activationsPerMinute: 2190,
      inferencesPerMinute: 3350,
      share: 0.13,
      ttftMs: 88,
    },
    {
      id: "llama-3_2-3b",
      name: "Llama 3.2 3B",
      modality: "Text",
      activeRegions: 3,
      activationsPerMinute: 1540,
      inferencesPerMinute: 2425,
      share: 0.09,
      ttftMs: 81,
    },
  ],
  timeseries: [
    { minute: "00", activations: 11210, inferences: 17022, ttftMs: 127 },
    { minute: "05", activations: 11884, inferences: 17620, ttftMs: 124 },
    { minute: "10", activations: 12490, inferences: 18344, ttftMs: 121 },
    { minute: "15", activations: 12920, inferences: 18810, ttftMs: 119 },
    { minute: "20", activations: 13110, inferences: 19042, ttftMs: 118 },
    { minute: "25", activations: 13380, inferences: 19324, ttftMs: 115 },
    { minute: "30", activations: 13620, inferences: 19750, ttftMs: 111 },
    { minute: "35", activations: 13888, inferences: 20142, ttftMs: 110 },
    { minute: "40", activations: 14100, inferences: 20494, ttftMs: 114 },
    { minute: "45", activations: 13955, inferences: 20180, ttftMs: 120 },
    { minute: "50", activations: 13714, inferences: 19860, ttftMs: 123 },
    { minute: "55", activations: 13402, inferences: 19484, ttftMs: 118 },
  ],
  events: [
    { time: "15:41:58", region: "Tokyo", model: "Llama 3.2 3B", message: "burst activation wave detected", tone: "hot" },
    { time: "15:41:36", region: "Virginia", model: "Llama 3.1 8B", message: "p95 ttft recovered under 220ms", tone: "stable" },
    { time: "15:41:22", region: "Frankfurt", model: "DeepSeek R1 Distill", message: "new edge pool attached", tone: "cool" },
    { time: "15:40:59", region: "Singapore", model: "Qwen 2.5 7B", message: "inference throughput peaked at 2.8k/min", tone: "hot" },
  ],
};

export const pulseQuerySketch = {
  overview: `
select
  count(distinct region_id) as live_regions,
  count(*) filter (where status = 'online') as active_edges,
  coalesce(sum(inferences_per_minute), 0) as inferences_per_minute,
  coalesce(sum(tokens_per_second), 0) as tokens_per_second,
  percentile_cont(0.5) within group (order by ttft_ms) as median_ttft_ms,
  percentile_cont(0.95) within group (order by ttft_ms) as p95_ttft_ms
from pulse_region_minute
where bucket_at = date_trunc('minute', now());
`,
  regions: `
select
  r.id,
  r.code,
  r.city_label as label,
  r.latitude as lat,
  r.longitude as lon,
  m.active_edges,
  m.activations_per_minute,
  m.inferences_per_minute,
  m.ttft_ms,
  dm.name as dominant_model
from pulse_regions r
join pulse_region_minute m on m.region_id = r.id
left join pulse_models dm on dm.id = m.dominant_model_id
where m.bucket_at = date_trunc('minute', now())
order by m.inferences_per_minute desc;
`,
  models: `
select
  pm.id,
  pm.slug,
  pm.name,
  pm.modality,
  count(distinct rmm.region_id) as active_regions,
  sum(rmm.model_activations_per_minute) as activations_per_minute,
  sum(rmm.model_inferences_per_minute) as inferences_per_minute,
  avg(rmm.ttft_ms) as ttft_ms
from pulse_models pm
join pulse_region_model_minute rmm on rmm.model_id = pm.id
where rmm.bucket_at >= now() - interval '1 minute'
group by pm.id, pm.slug, pm.name, pm.modality
order by inferences_per_minute desc;
`,
};

// The smbCloud GresIQ backend is Rails, which serializes JSON in snake_case by
// default (active_edges, inferences_per_minute, ttft_ms, ...). Every consumer in
// this app reads camelCase. Normalize once, right after the fetch, so the render
// path is correct regardless of what casing the backend sends. camelCase input
// passes through unchanged because each lookup prefers the camelCase key first.

function pick<T = unknown>(
  obj: RawPulseRecord | null | undefined,
  camel: string,
  snake: string,
): T | undefined {
  if (obj == null) return undefined;
  return (obj[camel] ?? obj[snake]) as T | undefined;
}

function normalizeSummary(
  summary: RawPulseRecord | null | undefined,
): PulseSummary | undefined {
  if (!summary) return undefined;
  return {
    liveRegions: pick(summary, "liveRegions", "live_regions"),
    activeEdges: pick(summary, "activeEdges", "active_edges"),
    inferencesPerMinute: pick(
      summary,
      "inferencesPerMinute",
      "inferences_per_minute",
    ),
    tokensPerSecond: pick(summary, "tokensPerSecond", "tokens_per_second"),
    medianTtftMs: pick(summary, "medianTtftMs", "median_ttft_ms"),
    p95TtftMs: pick(summary, "p95TtftMs", "p95_ttft_ms"),
    totalInferences: pick(summary, "totalInferences", "total_inferences"),
  };
}

function normalizeRegion(
  region: RawPulseRecord | null | undefined,
): PulseRegion | null | undefined {
  if (!region) return region;
  return {
    id: region.id as string | number | undefined,
    code: region.code as string | undefined,
    label: pick(region, "label", "city_label"),
    lat: pick(region, "lat", "latitude"),
    lon: pick(region, "lon", "longitude"),
    activeEdges: pick(region, "activeEdges", "active_edges"),
    activationsPerMinute: pick(
      region,
      "activationsPerMinute",
      "activations_per_minute",
    ),
    inferencesPerMinute: pick(
      region,
      "inferencesPerMinute",
      "inferences_per_minute",
    ),
    ttftMs: pick(region, "ttftMs", "ttft_ms"),
    dominantModel: pick(region, "dominantModel", "dominant_model"),
  };
}

function normalizeModel(
  model: RawPulseRecord | null | undefined,
): PulseModel | null | undefined {
  if (!model) return model;
  return {
    id: model.id as string | number | undefined,
    slug: model.slug as string | undefined,
    name: model.name as string | undefined,
    modality: model.modality as string | undefined,
    activeRegions: pick(model, "activeRegions", "active_regions"),
    activationsPerMinute: pick(
      model,
      "activationsPerMinute",
      "activations_per_minute",
    ),
    inferencesPerMinute: pick(
      model,
      "inferencesPerMinute",
      "inferences_per_minute",
    ),
    ttftMs: pick(model, "ttftMs", "ttft_ms"),
    share: model.share as number | undefined,
  };
}

export function normalizePulseData(
  data: RawPulseData | null | undefined,
): PulseData | null | undefined {
  if (!data || typeof data !== "object") return data;
  return {
    ...data,
    updatedAt: pick(data, "updatedAt", "updated_at"),
    summary: normalizeSummary(data.summary),
    regions: Array.isArray(data.regions)
      ? (data.regions.map(normalizeRegion).filter(Boolean) as PulseRegion[])
      : undefined,
    models: Array.isArray(data.models)
      ? (data.models.map(normalizeModel).filter(Boolean) as PulseModel[])
      : undefined,
  };
}

export function formatPulseTimestamp(timestamp: string | number | Date): string {
  return new Intl.DateTimeFormat("en", {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
    timeZone: "UTC",
  }).format(new Date(timestamp));
}
```


**Tool Call: cd /Users/setoelkahfi/Repositories/smbcloud/services/smbcloud-api && grep -rniE "geocoder|maxmind|geoip|geolocat|remote_ip|latitude" --include="*.rb" app lib config db/schema.rb 2>/dev/null | grep -v spec | head -40**
Status: Completed

Terminal:
```
app/controllers/gresiq/v1/pulse_controller.rb:209:            lat:                  region.latitude,
app/controllers/gresiq/v1/pulse_controller.rb:270:            lat:                  region.latitude,
app/controllers/gresiq/v1/pulse_controller.rb:348:      # ── IP geolocation helpers ───────────────────────────────────────────────
app/controllers/gresiq/v1/pulse_controller.rb:356:        ip = request.remote_ip
app/controllers/gresiq/v1/pulse_controller.rb:359:        result = Geocoder.search(ip).first
app/controllers/gresiq/v1/pulse_controller.rb:370:          latitude:   result.latitude.to_f,
app/controllers/gresiq/v1/pulse_controller.rb:380:      def find_or_create_region(id:, code:, city_label:, latitude:, longitude:)
app/controllers/gresiq/v1/pulse_controller.rb:385:          r.latitude   = latitude
config/initializers/geocoder.rb:3:# IP geolocation for pulse edge region detection.
config/initializers/geocoder.rb:6:Geocoder.configure(
db/schema.rb:526:    t.float "latitude", null: false
```


**Tool Call: sed -n '330,400p' app/controllers/gresiq/v1/pulse_controller.rb && echo "=== geocoder init ===" && cat config/initializers/geocoder.rb && echo "=== schema regions ===" && sed -n '515,545p' db/schema.rb**
Status: Completed

Terminal:
```
      # owned by the GresIQ app's prefixed schema.
      def ar_class_raw(table_name)
        @ar_classes_raw ||= {}
        @ar_classes_raw[table_name] ||= Class.new(ApplicationRecord) do
          self.table_name  = table_name
          self.primary_key = 'id'
        end
      end

      # Compute a percentile from a pre-sorted array of integers.
      # Returns 0 for empty input.
      def percentile_value(sorted_values, p)
        return 0 if sorted_values.empty?

        idx = [ (p * (sorted_values.length - 1)).round, sorted_values.length - 1 ].min
        sorted_values[idx]
      end

      # ── IP geolocation helpers ───────────────────────────────────────────────

      # Resolve the pulse_region for the current request IP at city level.
      # The first device from a city creates the region row; subsequent devices
      # from the same city reuse it.
      # Region ID is a stable slug e.g. "stockholm-se", "jakarta-id".
      # Returns "unknown" for private/loopback IPs or when geocoding fails.
      def resolve_region
        ip = request.remote_ip
        return 'unknown' if private_ip?(ip)

        result = Geocoder.search(ip).first
        return 'unknown' unless result&.city.present? && result&.country_code.present?

        city         = result.city
        country_code = result.country_code.upcase
        region_id    = "#{city.downcase.gsub(/[^a-z0-9]+/, '-')}-#{country_code.downcase}"

        find_or_create_region(
          id:         region_id,
          code:       "#{country_code}-#{city.first(3).upcase}",
          city_label: "#{city}, #{country_code}",
          latitude:   result.latitude.to_f,
          longitude:  result.longitude.to_f,
        )
      rescue StandardError => error
        Rails.logger.warn("pulse: geocode failed for #{ip.inspect}: #{error.message}")
        'unknown'
      end

      # Find or auto-create a pulse_regions row for this city.
      # Returns the region id on success, "unknown" on any DB error.
      def find_or_create_region(id:, code:, city_label:, latitude:, longitude:)
        region_class = ar_class('pulse_regions')
        region_class.find_or_create_by!(id: id) do |r|
          r.code       = code
          r.city_label = city_label
          r.latitude   = latitude
          r.longitude  = longitude
        end
        id
      rescue ActiveRecord::RecordInvalid, ActiveRecord::RecordNotUnique => error
        Rails.logger.warn("pulse: region upsert failed for #{id.inspect}: #{error.message}")
        'unknown'
      end

      # True for loopback (127.x, ::1) and RFC-1918 private addresses.
      def private_ip?(ip)
        addr = IPAddr.new(ip.to_s)
        addr.loopback? || addr.private?
      rescue IPAddr::InvalidAddressError
        true
      end
=== geocoder init ===
# frozen_string_literal: true

# IP geolocation for pulse edge region detection.
# Uses ipinfo.io (free tier — 50k req/month, no API key required).
# Caches results in Rails.cache to avoid redundant lookups for the same IP.
Geocoder.configure(
  # Use ipinfo.io for IP → lat/lon lookups.
  ip_lookup: :ipinfo_io,

  # Fail fast — pulse telemetry is non-critical.
  timeout: 3,

  # Cache geocode results so repeated model-loads from the same device
  # don't hammer the lookup service.
  cache: Rails.cache,

  # Suppress exceptions — errors are logged, not raised.
  always_raise: [],
)
=== schema regions ===
    t.string "region_id", null: false
    t.uuid "model_id", null: false
    t.integer "model_activations_per_minute", null: false
    t.integer "model_inferences_per_minute", null: false
    t.integer "ttft_ms", null: false
    t.index ["bucket_at"], name: "gresiq_ondeinference_pulse_rmm_bucket_idx", order: :desc
  end

  create_table "gresiq_ondeinference_pulse_regions", id: :string, force: :cascade do |t|
    t.string "code", null: false
    t.string "city_label", null: false
    t.float "latitude", null: false
    t.float "longitude", null: false
    t.string "provider"
    t.datetime "created_at", default: -> { "now()" }, null: false
    t.index ["code"], name: "index_gresiq_ondeinference_pulse_regions_on_code", unique: true
  end

  create_table "jwt_denylists", force: :cascade do |t|
    t.string "jti", null: false
    t.datetime "exp", null: false
    t.index ["jti"], name: "index_jwt_denylists_on_jti"
  end

  create_table "mail_api_keys", id: :uuid, default: -> { "gen_random_uuid()" }, force: :cascade do |t|
    t.uuid "mail_app_id", null: false
    t.bigint "tenant_id", null: false
    t.bigint "created_by_id", null: false
    t.string "name", null: false
    t.string "token_digest", null: false
    t.string "token_prefix", null: false
```


**Tool Call: cd /Users/setoelkahfi/Repositories/onde && git show feature/decouple-gresiq-from-onde-inference:src/pulse/events.rs | head -120**
Status: Completed

Terminal:
```
use serde::Serialize;

/// What gets sent to GresIQ when a model finishes loading.
#[derive(Debug, Clone, Serialize)]
pub struct ModelLoadedEvent {
    /// Which machine this is. Keep it stable across restarts — change it
    /// and you'll get a duplicate edge row in the dashboard.
    pub edge_id: String,
    /// The HuggingFace repo ID, e.g. `bartowski/Qwen2.5-3B-Instruct-GGUF`.
    pub model_id: String,
    /// What shows up in the UI, e.g. `Qwen 2.5 3B`.
    pub model_name: String,
    /// How long the load took. This is `elapsed.as_millis() as u64`,
    /// where `elapsed` is what `load_gguf_model` returns.
    pub load_duration_ms: u64,
    /// The Onde app that loaded this model (UUID string from the
    /// ondeinference.com dashboard). `None` for SDK builds without app
    /// credentials or for direct Rust consumers. In the document store this
    /// is a plain field on the `pulse_models` doc, not a foreign key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onde_app_id: Option<String>,
}

/// What gets sent to GresIQ after each `send_message` or `generate`.
#[derive(Debug, Clone, Serialize)]
pub struct InferenceEvent {
    /// Same edge as in `ModelLoadedEvent`.
    pub edge_id: String,
    /// Same model as in `ModelLoadedEvent`.
    pub model_id: String,
    /// Auto-generated per request. Looks like `onde-1720000000000-42`.
    pub request_id: String,
    /// Wall-clock time for the whole response, in milliseconds.
    pub duration_ms: u64,
    /// Time to first token, in milliseconds. Only the streaming path can
    /// observe this; a blocking `send_message` gets the whole response at
    /// once and reports `None` rather than a stand-in value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttft_ms: Option<u64>,
    /// `"success"`, `"cancelled"`, or `"error"`. Currently always
    /// `"success"` here — inference errors throw before we reach this point.
    pub status: String,
    /// Same Onde app as the corresponding `ModelLoadedEvent`. `None` for SDK
    /// builds without app credentials or direct Rust consumers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onde_app_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_loaded_serializes_onde_app_id_when_present() {
        let event = ModelLoadedEvent {
            edge_id: "edge-1".into(),
            model_id: "org/repo".into(),
            model_name: "Repo 3B".into(),
            load_duration_ms: 1234,
            onde_app_id: Some("app-uuid".into()),
        };
        let value = serde_json::to_value(&event).expect("serialize");
        assert_eq!(value["onde_app_id"], "app-uuid");
        assert_eq!(value["load_duration_ms"], 1234);
    }

    #[test]
    fn onde_app_id_is_omitted_when_absent() {
        let event = InferenceEvent {
            edge_id: "edge-1".into(),
            model_id: "org/repo".into(),
            request_id: "onde-1-0".into(),
            duration_ms: 42,
            ttft_ms: None,
            status: "success".into(),
            onde_app_id: None,
        };
        let value = serde_json::to_value(&event).expect("serialize");
        assert!(value.get("onde_app_id").is_none());
    }

    #[test]
    fn ttft_is_omitted_when_the_path_cannot_measure_it() {
        let mut event = InferenceEvent {
            edge_id: "edge-1".into(),
            model_id: "org/repo".into(),
            request_id: "onde-1-0".into(),
            duration_ms: 42,
            ttft_ms: None,
            status: "success".into(),
            onde_app_id: None,
        };
        assert!(serde_json::to_value(&event)
            .expect("serialize")
            .get("ttft_ms")
            .is_none());

        event.ttft_ms = Some(17);
        assert_eq!(serde_json::to_value(&event).expect("serialize")["ttft_ms"], 17);
    }
}
```


**Tool Call: cd /Users/setoelkahfi/Repositories/onde-web && cat app/api/gresiq/pulse/data/route.ts && echo "=== pulse-source.ts ===" && cat app/pulse/pulse-source.ts**
Status: Completed

Terminal:
```
import { NextResponse } from "next/server";
import {
  resolvedEnv,
  smbcloudApiUrl,
  gresiqCredentials,
} from "../../../_lib/credentials";
import { normalizePulseData } from "../../../../pulse/pulse-data";

// GET /api/gresiq/pulse/data?env=development|production
//
// Proxy the smbCloud GresIQ pulse endpoint.
// GresIQ credentials come from server-side env vars and never reach the browser.
export async function GET(request: Request) {
  const { searchParams } = new URL(request.url);
  const env = resolvedEnv(searchParams);
  const { apiKey, apiSecret } = gresiqCredentials(env);

  if (!apiKey || !apiSecret) {
    return NextResponse.json(
      { error: "GresIQ pulse credentials not configured." },
      { status: 503 },
    );
  }

  const span = searchParams.get("span");
  const windowMinutes = searchParams.get("window_minutes");

  let queryStr = "";
  if (span === "alltime") {
    queryStr = "?span=alltime";
  } else if (windowMinutes) {
    queryStr = `?window_minutes=${windowMinutes}`;
  }

  const url = `${smbcloudApiUrl(env)}/gresiq/v1/pulse/data${queryStr}`;

  try {
    const upstream = await fetch(url, {
      headers: {
        "Content-Type": "application/json",
        "X-Gresiq-Api-Key": apiKey,
        "X-Gresiq-Api-Secret": apiSecret,
      },
      // Revalidate every 60 seconds. The backend aggregates this data per minute.
      next: { revalidate: 60 },
    });

    const body = await upstream.json().catch(() => ({}));
    // Normalize snake_case (Rails) → camelCase so the per-app dashboard, which
    // reads camelCase fields, renders correctly regardless of upstream casing.
    const normalized = upstream.ok ? normalizePulseData(body) : body;
    return NextResponse.json(normalized, { status: upstream.status });
  } catch (error) {
    return NextResponse.json(
      { error: (error as Error).message },
      { status: 502 },
    );
  }
}
=== pulse-source.ts ===
// Server-only source policy for per-app pulse reads.
//
// `document` is per-app allowlisted rather than a global flip so one bad
// tenant dataset rolls back on its own. Nothing here is NEXT_PUBLIC_-prefixed:
// the browser never learns or chooses which source served a response.

import type { PulseData } from "@/types/pulse";
import type {
  PulseDocumentDiagnostics,
  PulseDocumentIncompleteReason,
} from "@/types/pulse-documents";

export type PulseSourceMode = "legacy" | "shadow" | "document";

type EnvRecord = Record<string, string | undefined>;

export function pulseSourceMode(
  appId: string,
  env: EnvRecord = process.env,
): PulseSourceMode {
  const configured = (env.ONDE_PULSE_DOCUMENT_READ ?? "").trim().toLowerCase();
  if (configured === "shadow") return "shadow";
  if (configured !== "document") return "legacy";

  const allowlist = (env.ONDE_PULSE_DOCUMENT_APPS ?? "")
    .split(",")
    .map((entry) => entry.trim())
    .filter(Boolean);

  return allowlist.includes(appId) ? "document" : "legacy";
}

export interface PulseParityReport {
  modelCount: { legacy: number; document: number };
  activeEdges: { legacy?: number; document?: number };
  totalInferences: { legacy?: number; document?: number };
  /** Model ids present in one source only. */
  modelIdMismatches: number;
  /** True only when both sides are comparable and every counted dimension agrees. */
  equal: boolean;
}

function modelIds(data: PulseData | null | undefined): Set<string> {
  return new Set(
    (data?.models ?? [])
      .map((model) => (model.id ?? model.slug)?.toString())
      .filter((id): id is string => Boolean(id)),
  );
}

/**
 * Compare a legacy response against a document response for one app and
 * window. Geographic dimensions are excluded: the M2 document set carries no
 * geography, and that gap gates the public Pulse view, not this summary.
 */
export function comparePulse(
  legacy: PulseData | null | undefined,
  document: PulseData | null | undefined,
): PulseParityReport {
  const legacyIds = modelIds(legacy);
  const documentIds = modelIds(document);
  let mismatches = 0;
  legacyIds.forEach((id) => {
    if (!documentIds.has(id)) mismatches += 1;
  });
  documentIds.forEach((id) => {
    if (!legacyIds.has(id)) mismatches += 1;
  });

  const report: PulseParityReport = {
    modelCount: { legacy: legacyIds.size, document: documentIds.size },
    activeEdges: {
      legacy: legacy?.summary?.activeEdges,
      document: document?.summary?.activeEdges,
    },
    totalInferences: {
      legacy: legacy?.summary?.totalInferences,
      document: document?.summary?.totalInferences,
    },
    modelIdMismatches: mismatches,
    equal: false,
  };

  report.equal =
    mismatches === 0 &&
    report.activeEdges.legacy === report.activeEdges.document &&
    report.totalInferences.legacy === report.totalInferences.document;

  return report;
}

export interface PulseSourceDiagnostic {
  appId: string;
  mode: PulseSourceMode;
  served: "legacy" | "document";
  incompleteReason?: PulseDocumentIncompleteReason;
  documents?: PulseDocumentDiagnostics;
  parity?: PulseParityReport;
}

/**
 * Emit counts and bounded categories only. Never event payloads, request
 * text, raw IPs, or any data-plane credential.
 */
export function emitPulseSourceDiagnostic(diagnostic: PulseSourceDiagnostic) {
  console.info("[pulse.source]", JSON.stringify(diagnostic));
}
```


**Tool Call: cd /Users/setoelkahfi/Repositories/onde && ls -a | head -30 && echo "=== docs ===" && ls docs .agents 2>/dev/null | head -30 && echo "=== pulse dir ===" && ls src/pulse/**
Status: Completed

Terminal:
```
.
..
.DS_Store
.agents
.cargo
.claude
.env
.git
.github
.gitignore
.zed
AGENTS.md
CHANGELOG.md
CLAUDE.md
Cargo.lock
Cargo.toml
LICENSE-APACHE
LICENSE-MIT
Makefile
README.md
assets
build.rs
dist
generated
rust-toolchain.toml
scripts
sdk
src
target
uniffi-bindgen
=== docs ===
.agents:
AGENTS.md
skills
=== pulse dir ===
client.rs
events.rs
mod.rs
```


**Tool Call: git show feature/decouple-gresiq-from-onde-inference:src/pulse/client.rs | sed -n '1,140p'**
Status: Completed

Terminal:
```
use smbcloud_gresiq_sdk::{Environment, GresiqClient, GresiqCredentials};

use super::events::{InferenceEvent, ModelLoadedEvent};

/// GresIQ credentials embedded at SDK build time — one pair per environment.
/// Consumer apps never set these — they're Onde Inference's own credentials.
const EMBEDDED_API_KEY_DEV: Option<&str> = option_env!("GRESIQ_API_KEY_DEV");
const EMBEDDED_API_SECRET_DEV: Option<&str> = option_env!("GRESIQ_API_SECRET_DEV");
const EMBEDDED_API_KEY_PRODUCTION: Option<&str> = option_env!("GRESIQ_API_KEY_PRODUCTION");
const EMBEDDED_API_SECRET_SECRET_PRODUCTION: Option<&str> =
    option_env!("GRESIQ_API_SECRET_PRODUCTION");

/// Onde telemetry client.  Wraps GresiqClient so pulse events land in the
/// right GresIQ-managed tables without consumer apps knowing anything about
/// the GresIQ auth layer underneath.
///
/// GresIQ credentials are embedded in the SDK at build time.
/// Consumer apps only provide an `edge_id` (stable device identifier).
///
/// Cheap to clone: the inner GresiqClient holds an Arc-backed reqwest::Client,
/// so cloning is a pointer bump, not a new TCP connection.
#[derive(Debug, Clone)]
pub struct PulseClient {
    inner: GresiqClient,
    edge_id: String,
    onde_app_id: Option<String>,
}

impl PulseClient {
    /// Returns true when pulse telemetry is disabled explicitly by the host app.
    pub fn disabled_by_env() -> bool {
        matches!(
            std::env::var("ONDE_DISABLE_PULSE")
                .ok()
                .as_deref()
                .map(str::trim)
                .map(str::to_ascii_lowercase)
                .as_deref(),
            Some("1") | Some("true") | Some("yes") | Some("on")
        )
    }

    /// ADR-0003 M2 dual-write toggle. The mirror is deliberately opt-in while
    /// document parity is being proven. When enabled, every attributed pulse
    /// event is *also* written to the GresIQ document gateway
    /// (`/gresiq/v1/collections/:collection`) in addition to the semantic
    /// `pulse/*` routes, which still serve reads until the M5 cut-over. Set
    /// `ONDE_PULSE_DUAL_WRITE=1` (or `true`/`yes`/`on`) to enable it.
    fn dual_write_enabled() -> bool {
        dual_write_enabled_value(std::env::var("ONDE_PULSE_DUAL_WRITE").ok().as_deref())
    }

    /// Build a pulse client using the GresIQ credentials embedded in the SDK.
    ///
    /// Returns `None` if the SDK was compiled without `GRESIQ_API_KEY` /
    /// `GRESIQ_API_SECRET` (e.g. a local dev build of onde without `.env`).
    /// In that case telemetry is silently disabled — no panic, no partial state.
    ///
    /// `edge_id` is a stable device identifier (installation UUID).
    /// Pass an empty string to default to `"onde-unknown"`.
    ///
    /// `onde_app_id` is the Onde app UUID (from the ondeinference.com
    /// dashboard) that owns this telemetry. `None` is fine — events then carry
    /// no app association (open-source/direct Rust consumers).
    pub fn new(
        environment: Environment,
        edge_id: String,
        onde_app_id: Option<String>,
    ) -> Option<Self> {
        if Self::disabled_by_env() {
            return None;
        }

        let (api_key, api_secret) = match environment {
            Environment::Dev => (EMBEDDED_API_KEY_DEV?, EMBEDDED_API_SECRET_DEV?),
            Environment::Production => (
                EMBEDDED_API_KEY_PRODUCTION?,
                EMBEDDED_API_SECRET_SECRET_PRODUCTION?,
            ),
        };

        let edge_id = if edge_id.is_empty() {
            "onde-unknown".to_string()
        } else {
            edge_id
        };

        // reqwest 0.12.x requires a live Tokio runtime (with I/O reactor)
        // to construct a Client.  GresiqClient::from_credentials calls
        // reqwest::Client::new() internally.  Guard against panics when
        // called from a thread/context that lacks a Tokio reactor.
        if tokio::runtime::Handle::try_current().is_err() {
            log::warn!(
                "pulse: no Tokio runtime available — \
                 deferring PulseClient creation"
            );
            return None;
        }

        let credentials = GresiqCredentials {
            api_key,
            api_secret,
        };

        let inner = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            GresiqClient::from_credentials(environment, credentials)
        })) {
            Ok(client) => client,
            Err(_) => {
                log::warn!(
                    "pulse: GresiqClient::from_credentials panicked \
                     (likely missing Tokio reactor) — telemetry disabled"
                );
                return None;
            }
        };

        Some(PulseClient {
            inner,
            edge_id,
            onde_app_id,
        })
    }

    /// Writes the model-load event to the pulse/model_loaded table and
    /// **awaits** the result before returning.
    ///
    /// This must complete before any `record_inference` call for the same
    /// edge + model is made, because the API enforces a foreign-key
    /// constraint: an `inference_event` row is rejected with HTTP 404
    /// ("Call model_loaded first") if no matching `model_loaded` row exists.
    ///
    /// Keeping this synchronous with respect to the model-load path means
    /// the row is always present by the time the first inference event fires.
    /// A failed write emits a warn! log line — no retry, no queue, and no
    /// effect on the caller.
    pub async fn record_model_loaded(
        &self,
        model_id: String,
        model_name: String,
```


**Tool Call: git show feature/decouple-gresiq-from-onde-inference:src/pulse/client.rs | sed -n '140,330p'**
Status: Completed

Terminal:
```
        model_name: String,
        load_duration_ms: u64,
    ) {
        let client = self.clone();
        let send_event = async move {
            let event = ModelLoadedEvent {
                edge_id: client.edge_id.clone(),
                model_id,
                model_name,
                load_duration_ms,
                onde_app_id: client.onde_app_id.clone(),
            };
            if let Err(error) = client.inner.insert("pulse/model_loaded", &event).await {
                log::warn!("pulse: model_loaded failed: {}", error);
            }
            client.spawn_model_loaded_dual_write(event);
        };

        if tokio::runtime::Handle::try_current().is_ok() {
            send_event.await;
            return;
        }

        let join = std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();

            match runtime {
                Ok(runtime) => {
                    runtime.block_on(send_event);
                }
                Err(error) => {
                    log::warn!(
                        "pulse: could not create fallback runtime for model_loaded: {}",
                        error
                    );
                }
            }
        });

        if join.join().is_err() {
            log::warn!("pulse: fallback model_loaded thread panicked");
        }
    }

    /// Same fire-and-forget pattern as record_model_loaded but for inference
    /// completions.  Writes to pulse/inference_event.  Logs on failure, no retry.
    ///
    /// Swift / UniFFI consumers may call into the SDK from contexts where
    /// `tokio::spawn` does not have a current runtime handle even though the
    /// outer API is async.  To avoid panicking on Apple platforms, we prefer
    /// the current Tokio runtime when available and otherwise fall back to a
    /// short-lived current-thread runtime on a native background thread.
    pub fn record_inference(
        &self,
        model_id: String,
        request_id: String,
        duration_ms: u64,
        ttft_ms: Option<u64>,
        status: String,
    ) {
        let client = self.clone();
        let send_event = async move {
            let event = InferenceEvent {
                edge_id: client.edge_id.clone(),
                model_id,
                request_id,
                duration_ms,
                ttft_ms,
                status,
                onde_app_id: client.onde_app_id.clone(),
            };
            if let Err(error) = client.inner.insert("pulse/inference_event", &event).await {
                log::warn!("pulse: inference_event failed: {}", error);
            }
            client.dual_write_inference(&event).await;
        };

        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(send_event);
            return;
        }

        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();

            match runtime {
                Ok(runtime) => {
                    runtime.block_on(send_event);
                }
                Err(error) => {
                    log::warn!(
                        "pulse: could not create fallback runtime for inference_event: {}",
                        error
                    );
                }
            }
        });
    }

    // ── ADR-0003 M2 dual-write to the document gateway ───────────────────────
    //
    // These mirror each semantic pulse event into JSONB collections under the
    // Onde gresiq_app. The relational `pulse/*` routes still own reads until the
    // M5 cut-over, so failures are logged and never affect inference.

    /// Move the three model-load mirror requests off the model-loading critical
    /// path. A dedicated thread is acceptable here because this runs once per
    /// model activation, not once per inference, and also works for callers
    /// whose original async runtime is short-lived.
    fn spawn_model_loaded_dual_write(&self, event: ModelLoadedEvent) {
        if !Self::dual_write_enabled() || event.onde_app_id.is_none() {
            return;
        }

        let client = self.clone();
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            match runtime {
                Ok(runtime) => runtime.block_on(client.dual_write_model_loaded(&event)),
                Err(error) => log::warn!(
                    "pulse: could not create runtime for model-loaded dual-write: {}",
                    error
                ),
            }
        });
    }

    /// Mirror a model-load into `pulse_edges`, `pulse_models`, and
    /// `pulse_deployments`. Each collection is a separate upsert (the
    /// FK-ordered chain of the semantic route becomes client-orchestrated
    /// multi-insert; the document store has no relationships to order).
    async fn dual_write_model_loaded(&self, event: &ModelLoadedEvent) {
        let Some(onde_app_id) = event.onde_app_id.as_deref() else {
            return;
        };

        let edge_key = scoped_document_key(onde_app_id, &[&event.edge_id]);

        let model_key = scoped_document_key(onde_app_id, &[&event.model_id]);
        let deployment_key = scoped_document_key(onde_app_id, &[&event.edge_id, &event.model_id]);
        let edge_document = edge_document(event, onde_app_id);
        let model_document = serde_json::json!({
            "slug": event.model_id,
            "model_id": event.model_id,
            "model_name": event.model_name,
            "onde_app_id": event.onde_app_id,
        });
        let deployment_document = serde_json::json!({
            "edge_id": event.edge_id,
            "model_id": event.model_id,
            "load_duration_ms": event.load_duration_ms,
            "onde_app_id": event.onde_app_id,
            "residency_state": "loaded",
        });

        tokio::join!(
            self.write_doc("pulse_edges", Some(&edge_key), &edge_document),
            self.write_doc("pulse_models", Some(&model_key), &model_document),
            self.write_doc(
                "pulse_deployments",
                Some(&deployment_key),
                &deployment_document,
            ),
        );
    }

    /// Mirror an inference event idempotently. Request IDs are already unique,
    /// so using an app-scoped natural key prevents a retry from double-counting
    /// the same inference during cutover validation.
    async fn dual_write_inference(&self, event: &InferenceEvent) {
        if !Self::dual_write_enabled() {
            return;
        }
        let Some(onde_app_id) = event.onde_app_id.as_deref() else {
            return;
        };
        let event_key = scoped_document_key(onde_app_id, &[&event.request_id]);
        let document = inference_document(event);

        self.write_doc("pulse_inference_events", Some(&event_key), &document)
            .await;
    }

    /// Upsert (or append, when `key` is `None`) one document, logging failures.
    async fn write_doc(&self, collection: &str, key: Option<&str>, doc: &serde_json::Value) {
```


**Tool Call: git show feature/decouple-gresiq-from-onde-inference:src/pulse/client.rs | sed -n '330,420p'**
Status: Completed

Terminal:
```
    async fn write_doc(&self, collection: &str, key: Option<&str>, doc: &serde_json::Value) {
        if let Err(error) = self.inner.upsert_document(collection, key, doc).await {
            log::warn!("pulse: dual-write to {} failed: {}", collection, error);
        }
    }
}

fn dual_write_enabled_value(value: Option<&str>) -> bool {
    matches!(
        value.map(str::trim).map(str::to_ascii_lowercase).as_deref(),
        Some("1") | Some("true") | Some("yes") | Some("on")
    )
}

/// Keys are scoped inside Onde's shared GresIQ app so identical installation
/// ids belonging to different Onde customer apps cannot overwrite each other.
fn scoped_document_key(onde_app_id: &str, parts: &[&str]) -> String {
    std::iter::once(onde_app_id)
        .chain(parts.iter().copied())
        .collect::<Vec<_>>()
        .join(":")
}

/// Build the inference document. `ttft_ms` is omitted entirely when the path
/// could not measure it, so a reader can tell "not measured" from a real zero
/// — the relational route stores a hardcoded `0` and cannot make that
/// distinction.
fn inference_document(event: &InferenceEvent) -> serde_json::Value {
    let mut document = serde_json::json!({
        "edge_id": event.edge_id,
        "model_id": event.model_id,
        "request_id": event.request_id,
        "duration_ms": event.duration_ms,
        "status": event.status,
        "onde_app_id": event.onde_app_id,
    });

    if let (Some(map), Some(ttft_ms)) = (document.as_object_mut(), event.ttft_ms) {
        map.insert("ttft_ms".to_string(), serde_json::json!(ttft_ms));
    }

    document
}

/// `status` is always `"online"`: this document is only written when the edge
/// just loaded a model, and nothing reports a shutdown. `last_seen_ms` is what
/// makes that honest — a reader counts an edge as active by recency, the way
/// the relational dashboard windows on `last_seen_at`, rather than trusting a
/// status that is never revoked.
fn edge_document(event: &ModelLoadedEvent, onde_app_id: &str) -> serde_json::Value {
    serde_json::json!({
        "edge_id": event.edge_id,
        "onde_app_id": onde_app_id,
        "status": "online",
        "last_seen_ms": epoch_millis(),
    })
}

/// Milliseconds since the Unix epoch. The crate has no date dependency and
/// this is only ever read as a recency window, so an integer beats pulling in
/// a formatting library for an RFC 3339 string.
fn epoch_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dual_write_requires_an_explicit_positive_flag() {
        for disabled in [
            None,
            Some(""),
            Some("0"),
            Some("false"),
            Some("no"),
            Some("off"),
        ] {
            assert!(!dual_write_enabled_value(disabled));
        }
        for enabled in [Some("1"), Some("true"), Some("YES"), Some(" on ")] {
            assert!(dual_write_enabled_value(enabled));
        }
    }

    #[test]
    fn document_keys_include_the_onde_app_boundary() {
```


## Assistant

I've got the full picture. Here's what M4 geography actually is today:

**smbcloud-api** (`gresiq/v1/pulse_controller.rb:348-395`) geolocates `request.remote_ip` through **ipinfo.io** at **city** level on every model-load, and auto-creates a permanent `gresiq_ondeinference_pulse_regions` row (`stockholm-se`, lat/lon, no retention, no k-anonymity — "the first device from a city creates the region row"). The public `/pulse` globe renders those rows. That's three ADR-0003 violations at once: product geography in the generic platform, device IPs going to a third party, and city-level pinpointing on a public page.

The M2 document set (`pulse_edges`) carries no geography at all, which is exactly why spec 0007 excluded the globe from parity.

Before I write spec 0008 and the stack, these are your calls, not mine:

**Tool Call: Asking for your input**
Status: Completed

Your questions have been answered: "Where should the public globe's geography come from once GresIQ stops geolocating IPs?"="Device-declared, from OS locale/timezone (Recommended)", "How coarse should a region be before it can appear on the public /pulse globe?"="Country, with a k-anonymity floor (Recommended)", "What retention applies to geography-bearing Pulse telemetry?"="Raw docs 30d, aggregates indefinite (Recommended)". You can now continue with these answers in mind.

## Input Requested

Please answer the following questions.

## Assistant

Recorded. Now checking tooling prerequisites before I start committing — this repo has two account remotes and no `origin`, which both gh-stack and commit signing care about.

**Tool Call: gh extension list 2>&1 | head; echo "--- config ---"; git config --get remote.pushDefault; git config --get rerere.enabled; git config --get user.name; git config --get user.email; git config --get user.signingkey; git config --get commit.gpgsign; echo "--- gh auth ---"; gh auth status 2>&1 | head -20**
Status: Completed

Terminal:
```
gh stack	github/gh-stack	v0.1.0
--- config ---
Seto Elkahfi
1797197+setoelkahfi@users.noreply.github.com
B58A4C0C7E61728364A29FC82B76DF0F04014E36
true
--- gh auth ---
github.com
  ✓ Logged in to github.com account setoelkahfi (keyring)
  - Active account: true
  - Git operations protocol: ssh
  - Token: gho_************************************
  - Token scopes: 'gist', 'read:org', 'repo'

  ✓ Logged in to github.com account keypair34 (keyring)
  - Active account: false
  - Git operations protocol: ssh
  - Token: gho_************************************
  - Token scopes: 'gist', 'read:org', 'repo'

  ✓ Logged in to github.com account paydii (keyring)
  - Active account: false
  - Git operations protocol: ssh
  - Token: gho_************************************
  - Token scopes: 'gist', 'read:org', 'repo'
```


**Tool Call: Load skill: git-commit-signing**
Status: Completed

Launching skill: git-commit-signing

## Assistant

You've hit your session limit · resets 11:10pm (Europe/Stockholm)
