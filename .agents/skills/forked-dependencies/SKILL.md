---
name: forked-dependencies
description: The complete inventory of forked crates and [patch.crates-io] overrides that the onde stack depends on — mistral.rs, candle, sysctl, mio, tqdm, core2. Explains what each fork is for, where it is consumed, how the git-vs-published two-tier model works, and how to update or add one. Apply whenever a build fails on a missing/incompatible crate for an Apple or Android target, when bumping the mistral.rs or candle fork, or when adding a new patched dependency.
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
user-invocable: true
---

# Skill: Forked Dependencies

The onde stack pulls several crates from forks instead of crates.io. They exist
for three reasons: **platform support** the upstream crate lacks (watchOS,
visionOS), **unreleased APIs** a dependency needs before upstream cuts a release
(candle), and **registry constraints** (`cargo publish` rejects git deps, so the
mistral.rs fork is republished under `onde-*` names).

This skill is the single source of truth for every fork and patch. When a build
breaks on a missing symbol or unsupported target, or you are bumping a fork,
start here.

---

## The two-tier consumption model (read this first)

There are two distinct ways the same dependency tree gets built, and they
resolve forks differently. Almost every confusing bug in this area comes from
mixing them up.

1. **Building the mistral.rs fork from source** (its own CI, or `onde` local dev
   with the git override uncommented). Here the fork's `Cargo.toml` git deps and
   `[patch.crates-io]` entries apply directly.

2. **Publishing to crates.io** (`cargo publish` of the `onde-mistralrs-*`
   crates). **`cargo publish` strips every `git =` field and the entire
   `[patch.crates-io]` table.** Published crates fall back to the `version = "..."`
   registry requirement on each dep. So a git-only dep is invisible to anyone
   installing from crates.io.

3. **onde consuming the published crates.** onde depends on the registry
   `onde-mistralrs-*` crates, so it gets the *registry-resolved* tree — the
   stripped-down one from step 2, not the fork's git tree. onde then re-applies
   whatever patches it still needs in **its own** `[patch.crates-io]`. Those
   apply to onde's from-source CI builds (Swift XCFramework, Android/Kotlin,
   the `cargo check` matrix) but are themselves stripped when onde publishes to
   crates.io.

**Consequence that used to bite (now fixed — kept as the canonical example):**
originally the `tqdm` fork bumped crossterm to 0.29 (which pulls mio 1.x, which
supports visionOS) but was a *git* dep of mistral.rs, so it was stripped on
publish. onde therefore resolved crates.io `tqdm 0.8.0` → `crossterm 0.25` →
`mio 0.8.11`, which does *not* support visionOS — so onde needed its own `mio`
visionOS patch even though the tqdm fork "already fixed" crossterm upstream of
it.

The fix that removed the whole problem: the crossterm-0.29 tqdm fork is now
**published to crates.io as `onde-tqdm`** and consumed by the mistral.rs fork as
a `version =` registry dep (package rename), exactly like `onde-mistralrs-*` and
`onde-candle-*`. A registry dep survives publish, so onde's registry-resolved
tree gets `onde-tqdm` → `crossterm 0.29` → `mio 1.x`, and the `mio 0.8.11`
line disappears entirely. This is now **done**: `onde-mistralrs-*` 0.9.3
(carrying the `onde-tqdm` dep) is published, onde consumes 0.9.3, and onde's
`[patch.crates-io] mio` entry has been **deleted**. The `mio` fork is retired.
See the onde-tqdm and mio entries.

---

## Inventory

| Crate | Fork / source | Ref | Purpose | Consumed where |
|-------|---------------|-----|---------|----------------|
| mistral.rs (12 crates) | `setoelkahfi/mistral.rs` | branch `fix/all-platform-fixes` | All-platform fixes; **published** as `onde-mistralrs-*` | onde deps (registry) |
| candle-core, candle-nn | `setoelkahfi/candle` | branch `onde/candle-0.11.0-27f20fea` (rev `27f20fea`) | Post-0.11.0 candle APIs; **published** as `onde-candle-{core,nn}` | mistral.rs fork deps (registry) |
| tqdm | `setoelkahfi/tqdm` | branch `release/onde-tqdm` (version `0.8.2`) | Bumps crossterm to 0.29 (→ mio 1.x); **published** as `onde-tqdm` | mistral.rs fork dep (registry) |
| sysctl | `setoelkahfi/sysctl-rs` | branch `feature/watchos` | watchOS (`target_os = "watchos"`) support | `[patch.crates-io]` in onde **and** mistral.rs fork |
| mio | `setoelkahfi/mio` | branch `feature/visionos` | visionOS support for the `mio 0.8.11` line | **RETIRED** — patch removed from onde once it consumed onde-mistralrs 0.9.3 (onde-tqdm → mio 1.x). Fork kept for history/upstream PR. |

> **Retired:** `core2` (`bbqsrc/core2` rev `545e84bc`) — was a `[patch.crates-io]`
> workaround for the yanked `core2 0.4.0`; removed from the mistral.rs fork after
> `bitstream-io` 4.x dropped `core2`. No longer present.

---

## Per-fork detail

### mistral.rs → `onde-mistralrs-*`

- **Fork:** `setoelkahfi/mistral.rs`, branch `fix/all-platform-fixes`.
- **Why:** carries fixes upstream hasn't merged (iOS 32-bit memory limits,
  watchOS/visionOS Metal compile, Android 32-bit const overflows, HF cache
  seeding, metallib link flags, and more).
- **Published as:** `onde-mistralrs`, `-core`, `-quant`, `-paged-attn`,
  `-flash-attn`, `-metal-compile`, `-code-exec`, `-sandbox`, `-macros`,
  `-vision`, `-audio`, `-mcp`. The rename is a `package = "onde-..."` override on
  each workspace dep entry, so Rust source still `use`s `mistralrs_*` unchanged.
- **Publishing + bump flow:** see [[sdk-regressions]] (the 12-crate publish
  order, the candle Step 0, and the tqdm/core2 publish-verification snags).

### candle → `onde-candle-{core,nn}`

- **Fork:** `setoelkahfi/candle`, branch `onde/candle-<ver>-<rev>` (currently
  `onde/candle-0.11.0-27f20fea`).
- **Why:** mistral.rs pins a candle git rev past the latest release for APIs no
  published candle has (`QTensor::indexed_gemv`, `gemv_fused_shared_lhs`,
  `BarrierPool::execute_chunked`). Publishing the `onde-mistralrs-*` crates
  fails verification (E0080/E0599) if their candle dep resolves to the released
  crate, so the pinned rev is republished under `onde-candle-*`.
- **Only the diverged crates are renamed.** At the 0.11.0 rev only `candle-core`
  and `candle-nn` differ from the release; `candle-metal-kernels` and
  `candle-flash-attn-v3` are byte-identical, so the mistral.rs fork points those
  at the official `0.11.0` crates. `candle-nn` must be republished whenever
  `candle-core` is (even if unchanged) or the official `candle-nn` drags the
  official `candle-core` back into the graph → two `candle-core` crates → type
  mismatch.

### sysctl → watchOS

- **Fork:** `setoelkahfi/sysctl-rs`, branch `feature/watchos`. Upstream PR:
  johalun/sysctl-rs#74.
- **Why:** upstream sysctl has no `target_os = "watchos"` support.
- **Applied in:** `[patch.crates-io]` in **both** onde and the mistral.rs fork
  (each workspace root that builds for watchOS needs its own patch entry).

### mio → visionOS (RETIRED)

- **Status:** the onde `[patch.crates-io] mio` entry has been **removed**. onde
  now consumes `onde-mistralrs 0.9.3`, whose `onde-tqdm` dep pulls `crossterm
  0.29 → mio 1.x`, so `mio 0.8.11` is gone from onde's tree. The fork below is
  kept only for history and a possible upstream PR; it is no longer wired into
  any build.
- **Fork:** `setoelkahfi/mio`, branch `feature/visionos` (commit `683fca8`).
  Based on `mio 0.8.11`. This is the **official GitHub fork** of tokio-rs/mio
  (in tokio's fork network, so it can open upstream PRs). An earlier standalone
  clone that carried this branch was renamed to `setoelkahfi/mio-patch` and
  deleted; the branch now lives on the real fork.
- **Why:** `mio 0.8.11` hits `compile_error!("unsupported target for
  mio::unix::pipe")` on visionOS, and its Apple cfg gates (kqueue `Filter`/
  `Flags` type aliases, `net.rs` `sin_len`/`sin6_len`, `tcp.rs` accept path,
  `uds/listener.rs` `accept4`/`SOCK_NONBLOCK`) omit visionos, breaking the Swift
  XCFramework build and the visionOS CI checks. The fork adds
  `target_os = "visionos"` to every cfg gate that already lists `tvos`/`watchos`.
  tokio's `mio 1.x` already supports visionOS, so **only the 0.8.x line** needs it.
- **Where 0.8.11 came from:** `tqdm → crossterm 0.25 → mio 0.8.11`, and
  `signal-hook-mio → crossterm`. Not from tokio (that uses `mio 1.2.2`).
- **Applied in:** *(historical)* was `[patch.crates-io]` in onde. The fork's
  version (`0.8.11`) stayed semver-compatible with the `^0.8` requirement it
  replaced; it did not touch tokio's `^1`.
- **How it was retired (the real fix):** the `onde-tqdm` publish (below) removed
  the `mio 0.8.11` line from onde's registry-resolved tree entirely. Verified:
  with onde consuming the onde-tqdm-carrying `onde-mistralrs 0.9.3` crates, onde's
  lockfile resolves `mio 1.2.2` only — no 0.8.11 — and both visionOS targets
  (`aarch64-apple-visionos{,-sim}`) `cargo +nightly check -Z build-std` pass with
  the `mio` patch deleted.

### tqdm → crossterm 0.29 → `onde-tqdm`

- **Fork:** `setoelkahfi/tqdm`. Two branches:
  - `deps/bump-crossterm` — keeps the crate named `tqdm`, feeds the **upstream
    PR** mrlazy1708/tqdm#28 (do not rename the package here).
  - `release/onde-tqdm` — sets `package = "onde-tqdm"` (crate lib name stays
    `tqdm` so downstream `use tqdm::…` and doctests are unchanged) and is
    **published to crates.io as `onde-tqdm`** (0.8.1, 0.8.2).
- **Why:** bumps crossterm to 0.29 (→ mio 1.x), eliminating the whole `mio
  0.8.11` visionOS problem at its source.
- **Consumed in:** the mistral.rs fork's `[workspace.dependencies]` as
  `tqdm = { version = "0.8.2", package = "onde-tqdm" }` — a **registry** rename
  dep, so it survives `cargo publish` and reaches onde (unlike the old git dep,
  which was stripped). This is the same rename trick as `onde-mistralrs-*` and
  `onde-candle-*`.
- **Publish order:** publish `onde-tqdm` **before** republishing the
  `onde-mistralrs-*` crates that depend on it. Note the published
  `onde-mistralrs-core 0.9.2` still uses plain `tqdm ^0.8.0` (pre-rename); the
  `onde-tqdm` dep first ships in `0.9.3`.

### core2 → removed

- **Was:** `bbqsrc/core2` rev `545e84bc`, a `[patch.crates-io]` workaround for
  the yanked `core2 0.4.0` (pulled via `bitstream-io → rav1e → ravif → image`).
- **Status:** **deleted** from the mistral.rs fork. `bitstream-io` 4.x moved off
  `core2` (to `no_std_io2`), so nothing pulls it. Removed in the same commit that
  wired `onde-tqdm` and bumped to 0.9.3.

---

## Operational gotchas (hit these during the 1.2.1 release)

- **A patched fork must be pushed to GitHub before the patch works in CI.** The
  `mio` fork existed only locally; the patch URL 404'd until it was pushed to
  `setoelkahfi/mio`. Verify with `git ls-remote <url> <branch>` before relying
  on a patch.
- **Shallow clones cannot be pushed** (`remote unpack failed: index-pack
  failed`). If a fork was cloned shallow, `git fetch --unshallow <upstream>`
  first, then push.
- **Local fork fetches need `CARGO_NET_GIT_FETCH_WITH_CLI=true`.** This machine
  rewrites GitHub URLs to SSH aliases; cargo's built-in fetcher can't auth, so
  export that var (or run the `cargo update` that pulls the fork with it). CI
  fetches public https forks fine without it — same as the long-standing sysctl
  patch.
- **`[patch.crates-io]` version must be semver-compatible** with the requirement
  it replaces. A `0.8.11` fork replaces `^0.8`, not a `^1` requirement of the
  same crate — which is how the `mio` patch touches only the crossterm line and
  leaves tokio's `mio 1.x` alone.
- **Forks are public** (mio, sysctl-rs, tqdm are MIT/Apache): keep the upstream
  `LICENSE` intact. See [[legal-and-trademarks]] for attribution obligations
  (candle and mistral.rs especially).

---

## Adding a new patch

1. Fork upstream, add the fix on a descriptive branch, **push it public** to
   `setoelkahfi/<crate>`, keep the `LICENSE`.
2. Add to `[patch.crates-io]` in every workspace root that builds the affected
   target (onde, and the mistral.rs fork if it builds there too). Match the
   fork's crate version to the requirement being replaced.
3. `CARGO_NET_GIT_FETCH_WITH_CLI=true cargo update -p <crate>@<ver>` to move the
   lockfile onto the git source; verify `source = "git+..."` in `Cargo.lock`.
4. If it fixes a mistral.rs-fork build (not just onde), remember it is stripped
   on publish — onde will still need its own copy of the patch unless the fix
   also lands in a published `onde-*` crate.
5. `cargo check` on host, then let CI validate the target that was failing.

---

*Update this skill whenever a fork's branch/rev changes, a patch is added or
removed, or a fork gets published under an `onde-*` name.*
