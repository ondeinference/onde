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

**Consequence that bites:** the `tqdm` fork bumps crossterm to 0.29 (which pulls
mio 1.x, which supports visionOS). But that fork is a *git* dep of mistral.rs,
so it is stripped on publish. onde therefore resolves crates.io `tqdm 0.8.0` →
`crossterm 0.25` → `mio 0.8.11`, which does *not* support visionOS. That is why
onde needs its own `mio` patch even though the tqdm fork "already fixed"
crossterm upstream of it. See the mio and tqdm entries below.

---

## Inventory

| Crate | Fork / source | Ref | Purpose | Consumed where |
|-------|---------------|-----|---------|----------------|
| mistral.rs (12 crates) | `setoelkahfi/mistral.rs` | branch `fix/all-platform-fixes` | All-platform fixes; **published** as `onde-mistralrs-*` | onde deps (registry) |
| candle-core, candle-nn | `setoelkahfi/candle` | branch `onde/candle-0.11.0-27f20fea` (rev `27f20fea`) | Post-0.11.0 candle APIs; **published** as `onde-candle-{core,nn}` | mistral.rs fork deps (registry) |
| sysctl | `setoelkahfi/sysctl-rs` | branch `feature/watchos` | watchOS (`target_os = "watchos"`) support | `[patch.crates-io]` in onde **and** mistral.rs fork |
| mio | `setoelkahfi/mio` | branch `feature/visionos` | visionOS support for the `mio 0.8.11` line | `[patch.crates-io]` in onde |
| tqdm | `setoelkahfi/tqdm` | branch `deps/bump-crossterm` (version `0.8.0`) | Bumps crossterm to 0.29 | git dep in mistral.rs fork |
| core2 | `bbqsrc/core2` (not ours) | rev `545e84bc` | Yanked-`core2 0.4.0` workaround | `[patch.crates-io]` in mistral.rs fork — **now unused, removable** |

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

### mio → visionOS

- **Fork:** `setoelkahfi/mio`, branch `feature/visionos`. Based on `mio 0.8.11`.
- **Why:** `mio 0.8.11` hits `compile_error!("unsupported target for
  mio::unix::pipe")` on visionOS, breaking the Swift XCFramework build and the
  visionOS CI checks. The fork adds `target_os = "visionos"` to the cfg gates
  that already list `tvos`/`watchos` (kqueue/pipe/waker/uds). tokio's `mio 1.x`
  already supports visionOS, so **only the 0.8.x line** needs the patch.
- **Where 0.8.11 comes from:** `tqdm → crossterm 0.25 → mio 0.8.11`, and
  `signal-hook-mio → crossterm`. Not from tokio (that uses `mio 1.2.2`).
- **Applied in:** `[patch.crates-io]` in onde. The fork's version (`0.8.11`)
  must stay semver-compatible with the `^0.8` requirement it replaces; it does
  not touch tokio's `^1`.
- **Alternative fix (not used):** patching `tqdm`/`crossterm` up to crossterm
  0.29 in onde would drop `mio 0.8.11` entirely. The `mio` patch is the smaller,
  targeted change.

### tqdm → crossterm 0.29

- **Fork:** `setoelkahfi/tqdm`, branch `deps/bump-crossterm`, pinned `version = "0.8.0"`.
- **Why:** bumps crossterm to 0.29.
- **Applied in:** git dep in the **mistral.rs fork** only. **Stripped on
  publish**, so it does not reach onde — onde gets crates.io `tqdm 0.8.0`
  (crossterm 0.25 → mio 0.8.11). This is exactly why the onde `mio` patch is
  required; do not assume the tqdm fork covers onde.

### core2 → (dead)

- **Source:** `bbqsrc/core2` rev `545e84bc`, patched in the mistral.rs fork.
- **Status:** **unused.** It was a workaround for the yanked `core2 0.4.0`
  (pulled via `bitstream-io → rav1e → ravif → image`). Bumping `bitstream-io` to
  `4.10.0` dropped `core2` (it moved to `no_std_io2`), so the patch is now
  `[patch.unused]` in the fork lockfile. Safe to delete from the fork's
  `[patch.crates-io]` on the next fork touch.

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
