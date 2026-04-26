---
name: sdk-releases
description: End-to-end SDK release process for all Onde distribution channels. Covers version bump checklist, CI/CD pipelines for Rust (crates.io), Swift (XCFramework → onde-swift), Dart (pub.dev), and npm, tag validation, GitHub Release creation, manual publish fallback, and common pitfalls. Apply whenever bumping versions or troubleshooting release workflows.
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
user-invocable: true
---

# Skill: SDK Releases

## What This Skill Covers

The complete release pipeline for shipping Onde across all registries:

- **Rust crate** → crates.io (`onde`)
- **Swift package** → Swift Package Index (`onde-swift`)
- **Dart/Flutter package** → pub.dev (`onde_inference`)
- **React Native** → npm (`@ondeinference/react-native`)
- **Kotlin Multiplatform** → Maven Central (`com.ondeinference:onde-inference`)

All five SDK workflows share the same tag trigger. A single tag push fires all
five in parallel. When only a subset needs (re-)publishing, use the **manual
publish fallback** documented below to avoid collateral re-runs.

---

## Version Bump Checklist

Every release starts with a version bump. **All version sources must be updated
in a single commit before tagging.**

| # | File | Field | Example |
|---|------|-------|---------|
| 1 | `Cargo.toml` (root) | `version` | `version = "0.1.3"` |
| 2 | `sdk/dart/pubspec.yaml` | `version` | `version: 0.1.3` |
| 3 | `sdk/dart/CHANGELOG.md` | New `## 0.1.3` section | Prepend at top of file |
| 4 | `sdk/react-native/package.json` | `version` | `"version": "0.1.3"` |
| 5 | `sdk/react-native/rust/Cargo.toml` | `version` | `version = "0.1.3"` |
| 6 | `sdk/react-native/CHANGELOG.md` | New `## 0.1.3` section | Prepend at top of file |
| 7 | `sdk/kotlin/gradle.properties` | `VERSION_NAME` | `VERSION_NAME=0.1.3` |
| 8 | `sdk/dart/rust/Cargo.lock` | `onde` package version | Run `cd sdk/dart/rust && cargo update -p onde` |
| 9 | `sdk/react-native/rust/Cargo.lock` | `onde` package version | Run `cd sdk/react-native/rust && cargo update -p onde` |
| 10 | `Cargo.lock` (root) | `onde` package version | Run `cargo check` at repo root |

### Files you do NOT manually edit

| File | Why |
|------|-----|
| `onde-swift/Package.swift` | CI rewrites the `url:` + `checksum:` automatically |
| `onde-swift/Sources/Onde/onde.swift` | CI copies the freshly generated UniFFI glue |

### Quick bump commands

```bash
# 1. Edit Cargo.toml version (manual)
# 2. Edit sdk/dart/pubspec.yaml version (manual)
# 3. Prepend new section to sdk/dart/CHANGELOG.md (manual)
# 4. Edit sdk/react-native/package.json version (manual)
# 5. Edit sdk/react-native/rust/Cargo.toml version (manual)
# 6. Prepend new section to sdk/react-native/CHANGELOG.md (manual)
# 7. Edit sdk/kotlin/gradle.properties VERSION_NAME (manual)

# 8. Sync lockfiles
cargo check                              # updates root Cargo.lock
cd sdk/dart/rust && cargo update -p onde  # updates Dart SDK's Cargo.lock
cd sdk/react-native/rust && cargo update -p onde   # updates React Native SDK's Cargo.lock

# 9. Verify
grep '^version' Cargo.toml                          # "0.1.3"
grep '^version:' sdk/dart/pubspec.yaml               # 0.1.3
grep '"version"' sdk/react-native/package.json                # "0.1.3"
grep '^version' sdk/react-native/rust/Cargo.toml              # "0.1.3"
grep 'VERSION_NAME' sdk/kotlin/gradle.properties              # 1.0.0
grep 'name = "onde"' -A1 Cargo.lock                  # version = "0.1.3"
grep 'name = "onde"' -A1 sdk/dart/rust/Cargo.lock    # version = "0.1.3"
grep 'name = "onde"' -A1 sdk/react-native/rust/Cargo.lock     # version = "0.1.3"

# 10. Commit and tag
git add -A
git commit -m "0.1.3"
git tag 0.1.3
git push origin main 0.1.3
```

---

## Tag Format

All tags are **bare semver** — no `v` prefix.

```
0.1.3       ✅ correct
v0.1.3      ❌ will NOT trigger CI
0.1.3-beta  ❌ will NOT match the tag pattern
```

All five CI workflows use the same trigger pattern:

```yaml
on:
  push:
    tags:
      - "[0-9]+.[0-9]+.[0-9]+"
```

---

## CI Pipeline: Rust SDK (crates.io)

**Workflow:** `.github/workflows/release-sdk-rust.yml`
**Trigger:** Tag push matching `[0-9]+.[0-9]+.[0-9]+` or `workflow_dispatch`
**Runner:** `macos-26`

### Flow

```
Tag push (e.g. 1.0.0)
  │
  ├─ 1. Checkout + install stable Rust
  │
  ├─ 2. Validate tag == Cargo.toml version
  │     └─ Fails fast if mismatched
  │
  ├─ 3. cargo publish --dry-run
  │     └─ Compiles and packages; catches errors before real publish
  │
  └─ 4. cargo publish (tag push only)
        └─ Authenticates via CARGO_REGISTRY_TOKEN secret
```

### Key details

- **`workflow_dispatch`** runs the dry-run only (the publish step has
  `if: startsWith(github.ref, 'refs/tags/')`) — useful for validation.
- **Runner is `macos-26`** because the crate depends on `mistralrs` with Metal
  features, which requires macOS + Xcode to compile.
- **crates.io is immutable** — once a version is published, it cannot be
  re-published. You must bump to the next version. `cargo yank` hides a version
  from dependency resolution but does not delete it.

### Required secret

`CARGO_REGISTRY_TOKEN` — a crates.io API token scoped to `publish-update` for
the `onde` crate. Create at https://crates.io/settings/tokens. Best practice:
scope it to a single crate rather than using a global token.

---

## CI Pipeline: Swift SDK

**Workflow:** `.github/workflows/release-sdk-swift.yml`
**Trigger:** Tag push matching `[0-9]+.[0-9]+.[0-9]+`
**Runner:** `macos-15`

### Flow

```
Tag push (e.g. 0.1.3)
  │
  ├─ 1. Build XCFramework
  │     └─ .github/scripts/build-swift-xcframework.sh
  │        ├─ Extracts version from Cargo.toml via tomllib
  │        ├─ Compiles staticlibs for 5 Apple targets
  │        ├─ Generates UniFFI Swift bindings
  │        ├─ Assembles XCFramework, zips, computes SHA-256
  │        └─ Writes dist/swift/{version.txt, OndeFramework.checksum.txt}
  │
  ├─ 2. Read version + checksum into step outputs
  │
  ├─ 3. Validate tag == Cargo.toml version
  │     └─ Fails fast if mismatched (e.g. tagged 0.1.3 but Cargo.toml says 0.1.2)
  │
  ├─ 4. Upload CI artifacts (every run, including workflow_dispatch)
  │
  ├─ 5. Create GitHub Release on `onde` (tag push only)
  │     └─ Attaches OndeFramework.xcframework.zip + checksum
  │
  ├─ 6. Checkout ondeinference/onde-swift (using ONDE_SWIFT_PAT)
  │
  ├─ 7. Rewrite Package.swift
  │     └─ Python regex replaces .binaryTarget url: + checksum:
  │
  ├─ 8. Copy generated onde.swift → onde-swift/Sources/Onde/
  │
  └─ 9. Commit + tag onde-swift
        ├─ git commit -m "Release 0.1.3"  (skipped if no changes)
        ├─ git push origin HEAD:main
        └─ git tag -a "0.1.3" -m "Release 0.1.3"  (skipped if tag exists)
            └─ Annotated tag, never force-pushed
```

### What happens on `onde-swift` after the push

The `onde` CI pushes a commit (to `main`) and a tag (e.g. `0.1.3`) to `onde-swift`.
This triggers **two** workflows on `onde-swift`:

1. **`ci.yml`** (on push to `main`) — validates manifest + builds all 5 platforms
2. **`release.yml`** (on tag push) — validates package, then creates a GitHub
   Release with installation instructions and a link to the upstream `onde` changelog

Both run concurrently without conflict — they use different concurrency groups
(`ci-refs/heads/main` vs the tag ref).

### Tag immutability

Tags on `onde-swift` are **never force-pushed**. SPM caches the resolved commit
SHA per tag in `Package.resolved`. Moving a tag silently breaks every consumer's
lockfile. If a release needs to be redone:

1. Delete the remote tag: `git push origin :refs/tags/0.1.3`
2. Delete the GitHub Release on both `onde` and `onde-swift`
3. Fix the issue, re-tag, push again

---

## CI Pipeline: npm SDK (React Native)

**Workflow:** `.github/workflows/release-sdk-npm.yml`
**Trigger:** Tag push matching `[0-9]+.[0-9]+.[0-9]+` or `workflow_dispatch`
**Runner:** `ubuntu-latest`

### Flow

```
Tag push (e.g. 1.0.0)
  │
  ├─ validate job:
  │   ├─ Set up Node.js 22
  │   ├─ Read version from sdk/react-native/package.json
  │   ├─ Validate tag == package.json version
  │   ├─ npm install + npm run build
  │   └─ npm publish --dry-run --access public
  │
  └─ publish job (tag push only, needs: validate):
      ├─ npm install + npm run build
      ├─ Check if version already exists on npm
      │   └─ If yes → skip publish (idempotent, exits 0)
      └─ npm publish --access public
```

### npm idempotency guard

The publish step checks `npm view <package>@<version>` before publishing. If the
version already exists, it **skips** with a success exit code. This makes the npm
workflow safe to re-run — unlike Dart/Rust, it won't fail on a duplicate version.

### Required secret

`NPM_TOKEN` — a granular npm access token scoped to the `@ondeinference` org
with read+write packages. Create at npmjs.com → Access Tokens.

---

## CI Pipeline: Dart SDK

**Workflow:** `.github/workflows/release-sdk-dart.yml`
**Trigger:** Tag push matching `[0-9]+.[0-9]+.[0-9]+`
**Runner:** `ubuntu-latest`

### Flow

```
Tag push (e.g. 0.1.3)
  │
  ├─ validate job:
  │   ├─ Set up Flutter (version from sdk/dart/.flutter-version)
  │   ├─ Read version from pubspec.yaml
  │   ├─ Validate tag == pubspec.yaml version
  │   │   └─ Fails fast if mismatched
  │   ├─ flutter pub get
  │   ├─ flutter analyze --no-fatal-infos
  │   └─ flutter pub publish --dry-run
  │
  └─ publish job (tag push only, needs: validate):
      ├─ Set up Flutter
      ├─ flutter pub get
      ├─ Write PUB_CREDENTIALS to ~/.config/dart/pub-credentials.json
      └─ flutter pub publish --force
```

### Tag validation (Dart)

The Dart workflow independently validates the tag against `pubspec.yaml`:

```yaml
if [ "$TAG" != "$PUBSPEC_VERSION" ]; then
  echo "::error::Tag '$TAG' does not match pubspec.yaml version '$PUBSPEC_VERSION'."
  exit 1
fi
```

This means **both** `Cargo.toml` and `pubspec.yaml` must have the same version
as the git tag, or both workflows fail.

### Required secret

`PUB_CREDENTIALS` — the full JSON content of `~/.config/dart/pub-credentials.json`
from a machine that has run `dart pub login`. Add as a repository secret on `onde`.

---

## GitHub Releases (two repos)

| Repo | Created by | Contains |
|------|-----------|----------|
| `onde` | `release-sdk-swift.yml` (step 5) | XCFramework zip + checksum, auto-generated release notes |
| `onde-swift` | `onde-swift` `release.yml` | Installation snippet, platform table, link to upstream changelog |

Both repos need GitHub Releases because they serve different audiences:

- **`onde`** — Rust consumers, asset hosting for the XCFramework binary
- **`onde-swift`** — Swift developers, Swift Package Index surfaces these as version notes

---

## Version Sources and Validation Matrix

| Source | Read by | Validated against |
|--------|---------|-------------------|
| `Cargo.toml` `version` | `build-swift-xcframework.sh` (tomllib) | Git tag (Swift workflow) |
| `sdk/dart/pubspec.yaml` `version` | `release-sdk-dart.yml` (grep + awk) | Git tag (Dart workflow) |
| Git tag (`github.ref_name`) | Both workflows | Both version files above |
| `onde-swift` tag | Created by `release-sdk-swift.yml` | Same value as the `onde` tag |

All four must match. If any pair diverges, the relevant CI job fails.

---

## Secrets Required

| Secret | Used by | Purpose |
|--------|---------|---------|
| `CARGO_REGISTRY_TOKEN` | `release-sdk-rust.yml` | Authenticate with crates.io for `cargo publish`. Scoped to `publish-update` for the `onde` crate. Create at https://crates.io/settings/tokens. |
| `ONDE_SWIFT_PAT` | `release-sdk-swift.yml` | Push commits + tags to `ondeinference/onde-swift`. Must be a PAT (not `GITHUB_TOKEN`) so it triggers workflows on `onde-swift`. Needs `contents: write` scope. |
| `PUB_CREDENTIALS` | `release-sdk-dart.yml` | Authenticate with pub.dev for `flutter pub publish`. Full JSON from `~/.config/dart/pub-credentials.json`. |
| `NPM_TOKEN` | `release-sdk-npm.yml` | Authenticate with npm for `npm publish`. Granular access token scoped to the `@ondeinference` org with read+write packages. Create at npmjs.com → Access Tokens. |
| `ORG_GRADLE_PROJECT_MAVENCENTRALUSERNAME` | `release-sdk-kotlin.yml` | Sonatype Central Portal token username. Generate at central.sonatype.com → Profile → Generate User Token. |
| `ORG_GRADLE_PROJECT_MAVENCENTRALPASSWORD` | `release-sdk-kotlin.yml` | Sonatype Central Portal token password. Same dialog as above. |
| `ORG_GRADLE_PROJECT_SIGNINGKEYID` | `release-sdk-kotlin.yml` | Last 8 chars of GPG key fingerprint used to sign Maven Central artifacts. |
| `ORG_GRADLE_PROJECT_SIGNINGKEY` | `release-sdk-kotlin.yml` | ASCII-armored GPG private key. Export with `gpg --armor --export-secret-keys <KEY_ID>`. |
| `ORG_GRADLE_PROJECT_SIGNINGPASSWORD` | `release-sdk-kotlin.yml` | Passphrase for the GPG signing key. |
| `GRESIQ_API_KEY_DEV` | All release workflows | Build-time env for dev environment API key. |
| `GRESIQ_API_SECRET_DEV` | All release workflows | Build-time env for dev environment API secret. |
| `GRESIQ_API_KEY_PRODUCTION` | All release workflows | Build-time env for production API key. |
| `GRESIQ_API_SECRET_PRODUCTION` | All release workflows | Build-time env for production API secret. |
| `HF_TOKEN` | All release workflows | HuggingFace token baked in at build time (`option_env!`). |

---

## Common Pitfalls

| Pitfall | Symptom | Fix |
|---------|---------|-----|
| Forgot to bump `pubspec.yaml` | Dart CI fails: "Tag '0.1.3' does not match pubspec.yaml version '0.1.2'" | Bump `sdk/dart/pubspec.yaml`, amend commit, re-tag |
| Forgot to bump `sdk/react-native/package.json` | npm CI fails: "Tag '0.1.3' does not match package.json version '0.1.2'" | Bump `sdk/react-native/package.json`, amend commit, re-tag |
| Forgot to bump `sdk/react-native/rust/Cargo.toml` | npm SDK's Rust bridge crate version drifts from the main crate | Bump `sdk/react-native/rust/Cargo.toml`, run `cd sdk/react-native/rust && cargo update -p onde` |
| Forgot to update `sdk/dart/rust/Cargo.lock` | Dart SDK's Rust bridge builds against stale `onde` version | Run `cd sdk/dart/rust && cargo update -p onde` |
| Forgot to update `sdk/react-native/rust/Cargo.lock` | npm SDK's Rust bridge builds against stale `onde` version | Run `cd sdk/react-native/rust && cargo update -p onde` |
| Forgot to update `sdk/dart/CHANGELOG.md` | pub.dev shows stale changelog | Prepend new `## 0.1.3` section before tagging |
| Forgot to bump `sdk/react-native/CHANGELOG.md` | npm shows stale changelog | Prepend new `## 0.1.3` section before tagging |
| Forgot to bump `sdk/kotlin/gradle.properties` `VERSION_NAME` | Kotlin CI fails: tag vs gradle.properties mismatch | Bump `VERSION_NAME`, amend commit, re-tag |
| Tag has `v` prefix (`v0.1.3`) | CI does not trigger — tag pattern requires bare semver | Delete the tag, re-tag without `v` |
| `CARGO_REGISTRY_TOKEN` missing | `cargo publish` fails with auth error | Create a scoped token at crates.io/settings/tokens, add as repo secret |
| `ONDE_SWIFT_PAT` expired | `onde-swift` push fails with 403 | Regenerate PAT at github.com/settings/tokens, update repo secret |
| `PUB_CREDENTIALS` expired | `flutter pub publish` fails with 401 | Run `dart pub login` locally, copy new credentials JSON to repo secret |
| `NPM_TOKEN` expired | `npm publish` fails with 401/403 | Regenerate token at npmjs.com → Access Tokens, update repo secret |
| crates.io version already published | `cargo publish` fails with "already uploaded" | crates.io is immutable — bump to next version. `cargo yank` hides but doesn't delete. |
| npm version already published | npm publish step skips (idempotency guard) | Safe — the workflow exits 0. No action needed. |
| pub.dev version already published | `flutter pub publish` fails | pub.dev is immutable — bump to next version |
| Maven Central version already published | `publishAndReleaseToMavenCentral` fails | Maven Central is immutable — bump to next version |
| Maven Central namespace not verified | Deployment fails: "Namespace 'com.ondeinference' is not allowed" | Verify namespace on central.sonatype.com (add DNS TXT record, wait for Sonatype to poll) |
| Tag pushed before workflow exists on `main` | Workflow never fires for that tag | Use `workflow_dispatch` for dry-run, then publish manually. See **Manual Publish Fallback**. |
| All 5 workflows fire but only 1–2 need re-publishing | Collateral re-runs of Swift/npm/Dart/Rust/Kotlin | Don't delete+re-push the tag. Use **Manual Publish Fallback** for the specific SDKs. |
| `onde-swift` tag already exists | Tag push skipped (warning emitted) | Delete remote tag first: `git push origin :refs/tags/0.1.3` |
| Force-pushed a tag on `onde-swift` | SPM users get stale `Package.resolved` | Never force-push. Delete + re-create instead. Advise consumers to run `swift package resolve` |
| XCFramework URL 404 | `swift package resolve` fails on consumer side | Ensure the `onde` GitHub Release was created BEFORE the `onde-swift` tag was pushed (the workflow handles this order automatically) |
| `Package.swift` regex didn't match | Python `RuntimeError` in CI | Check that `onde-swift/Package.swift` still has a `.binaryTarget(name: "OndeFramework", ...)` block |
| Dart `flutter analyze` fails | `validate` job fails, `publish` is skipped | Fix analysis issues, re-tag |

---

## Manual Publish Fallback

When a tag was pushed before a workflow existed on `main`, or when you need to
publish only a subset of SDKs without re-triggering all four workflows:

### Rust (crates.io)

```bash
# Dry-run first
cargo publish --dry-run

# Publish
cargo publish
```

Requires `~/.cargo/credentials.toml` or `CARGO_REGISTRY_TOKEN` env var.

### Dart (pub.dev)

```bash
cd sdk/dart

# Dry-run
flutter pub publish --dry-run

# Publish
flutter pub publish --force
```

Requires `~/.config/dart/pub-credentials.json` from `dart pub login`.

### Kotlin (Maven Central)

```bash
cd sdk/kotlin

# Dry-run
./gradlew :lib:publishToMavenCentral --dry-run --no-configuration-cache

# Publish and release
./gradlew :lib:publishAndReleaseToMavenCentral --no-configuration-cache
```

Requires `~/.gradle/gradle.properties` with `mavenCentralUsername`,
`mavenCentralPassword`, `signingKeyId`, `signingKey`, `signingPassword`.
Also requires the `com.ondeinference` namespace to be verified on
central.sonatype.com.

### npm (React Native)

```bash
cd sdk/react-native
npm run build
npm publish --access public
```

Requires `npm login` or `NPM_TOKEN` env var.

### When to use manual publish

| Scenario | Approach |
|----------|----------|
| Tag exists but workflow wasn't on `main` yet | Merge workflow, then publish manually |
| One SDK's CI failed, others succeeded | Fix the issue, publish that SDK manually |
| Need to avoid re-triggering Swift XCFramework build | Publish Rust/Dart/npm/Kotlin manually |
| Testing a new workflow before first real tag | Use `workflow_dispatch` for dry-run only |

---

## Re-releasing a Version (Emergency Fix)

If a release was botched and you need to re-publish the same version:

### Swift (onde-swift)

```bash
# 1. Delete the remote tag on onde-swift
git -C /path/to/onde-swift push origin :refs/tags/0.1.3

# 2. Delete the GitHub Release on onde-swift (via GitHub UI or gh CLI)
gh release delete 0.1.3 --repo ondeinference/onde-swift --yes

# 3. Delete the GitHub Release on onde
gh release delete 0.1.3 --repo ondeinference/onde --yes

# 4. Delete the local and remote tag on onde
git tag -d 0.1.3
git push origin :refs/tags/0.1.3

# 5. Fix the issue, commit, re-tag, push
git tag 0.1.3
git push origin main 0.1.3
```

### Dart (pub.dev) / Rust (crates.io) / Kotlin (Maven Central)

**pub.dev, crates.io, and Maven Central are all immutable.** Once a version is
published, it cannot be re-published. You must bump to `0.1.4` instead.

- **crates.io:** `cargo yank --version 0.1.3` hides the version from dependency
  resolution but does not delete it.
- **pub.dev:** No yank equivalent. The version remains visible and installable.

If the `publish` job failed (i.e. the package was NOT published), you can
safely re-tag after fixing the issue.

### npm

The npm workflow's idempotency guard means re-running it is safe — it skips
if the version already exists. However, npm also does not allow re-publishing
the same version with different content. Bump to the next version if the
published content was wrong.

---

## End-to-End Release Timeline

```
Developer merges feature → development → main (--no-ff) then tags
  │
  ├─── release-sdk-rust.yml fires ──────────────────────────────────────┐
  │     ~2 min (dry-run + publish to crates.io)                          │
  │                                                                      │
  ├─── release-sdk-swift.yml fires ──────────────────────────────────────┤
  │     ~15 min (XCFramework build)                                      │
  │     Creates GitHub Release on onde                                   │
  │     Pushes commit + tag to onde-swift                                │
  │          │                                                           │
  │          ├─ onde-swift ci.yml fires (push to main)                   │
  │          │   ~5 min (validate + build 5 platforms)                   │
  │          │                                                           │
  │          └─ onde-swift release.yml fires (tag 1.0.0)                 │
  │              ~5 min (validate + create GitHub Release)               │
  │                                                                      │
  ├─── release-sdk-dart.yml fires ───────────────────────────────────────┤
  │     ~3 min (validate + publish to pub.dev)                           │
  │                                                                      │
  ├─── release-sdk-npm.yml fires ───────────────────────────────────────┤
  │     ~3 min (validate + publish to npm)                               │
  │                                                                      │
  ├─── release-sdk-kotlin.yml fires ────────────────────────────────────┤
  │     ~10 min (Android ABIs + JVM dylib + publish to Maven Central)    │
  │                                                                      │
  └─── Swift Package Index picks up onde-swift tag ──────────────────────┘
        ~30 min (SPI polling interval)
        klibs.io picks up Maven Central artifact within ~30 days
```

All five CI workflows fire **in parallel** from the single tag push. The only
ordering dependency is that `release-sdk-swift` must create the `onde` GitHub
Release (with XCFramework assets) **before** pushing the tag to `onde-swift` —
this is guaranteed by the step order within the workflow.

### Merge train

Always merge through the branch train before tagging. Use `--no-ff` to
create explicit merge commits — they serve as bookmarks in the history
showing where each feature landed.

```bash
git checkout development && git merge feature/my-branch --no-ff --no-edit
git checkout main && git merge development --no-ff --no-edit
git push origin development main
git tag X.Y.Z && git push origin X.Y.Z
```

Never tag from a feature branch. The tag must point to a commit on `main` that
contains all workflow files and fixes. If a workflow is added on a feature branch
and the tag is pushed before merging, the workflow won't exist at the tagged
commit and CI won't fire.

**Why `--no-ff`:** Multi-commit feature branches (like the Kotlin SDK with 35
files across 6 commits) look like they were committed directly to `main` with
fast-forward merges. `--no-ff` preserves the branch context. It also allows
reverting an entire feature with a single `git revert -m 1 <merge-commit>`.

---

## Distribution Registry Reference

| Registry | Package Name | Import | Workflow | Immutable? |
|----------|-------------|--------|----------|------------|
| crates.io | `onde` | `onde = "1.0"` | `release-sdk-rust.yml` | Yes (yank hides, doesn't delete) |
| Swift Package Index | `onde-swift` (org: `ondeinference`) | `import Onde` | `release-sdk-swift.yml` → `onde-swift/release.yml` | Tags are immutable by convention |
| pub.dev | `onde_inference` | `import 'package:onde_inference/onde_inference.dart'` | `release-sdk-dart.yml` | Yes (no yank) |
| npm | `@ondeinference/react-native` | `import { OndeChatEngine } from "@ondeinference/react-native"` | `release-sdk-npm.yml` | Yes (unpublish within 72h only) |
| Maven Central | `com.ondeinference:onde-inference` | `implementation("com.ondeinference:onde-inference:1.0.0")` | `release-sdk-kotlin.yml` | Yes (cannot re-publish same version) |
| klibs.io | `onde-inference` | Same Maven Central coordinate | Auto-indexed from Maven Central | N/A (discovery only) |

---

---

## React Native npm SDK

**Package:** `@ondeinference/react-native` (scoped under `@ondeinference` org on npm)
**Location:** `sdk/react-native/`
**Architecture:** Expo native module wrapping Rust via C FFI

### How it works

```
TypeScript (React Native)
  │  import { OndeChatEngine } from "@ondeinference/react-native"
  ▼
Expo Module (Swift / Kotlin)
  │  OndeInferenceModule — calls C FFI functions via @_silgen_name (iOS) / JNI (Android)
  ▼
Rust C FFI bridge (sdk/react-native/rust/)
  │  extern "C" functions with JSON serialization, global tokio::Runtime
  ▼
onde crate (src/)
  │  ChatEngine — tokio::sync::Mutex, mistral.rs inference
  ▼
mistral.rs → Metal (iOS) / CPU (Android)
```

### Key files

| File | Purpose |
|------|---------|
| `sdk/react-native/package.json` | npm package — `@ondeinference/react-native` |
| `sdk/react-native/expo-module.config.json` | Expo autolinking config |
| `sdk/react-native/src/index.ts` | Public TypeScript API — `OndeChatEngine`, free functions, JSON ↔ camelCase conversion |
| `sdk/react-native/src/types.ts` | TypeScript type definitions mirroring Rust types |
| `sdk/react-native/src/OndeInferenceModule.ts` | `requireNativeModule("OndeInference")` bridge |
| `sdk/react-native/rust/src/lib.rs` | C FFI exports (`extern "C"`) + Android JNI wrappers |
| `sdk/react-native/ios/OndeInferenceModule.swift` | Swift Expo module — calls Rust via `@_silgen_name` |
| `sdk/react-native/android/.../OndeInferenceModule.kt` | Kotlin Expo module — calls Rust via JNI `external fun` |
| `sdk/react-native/scripts/build-rust.sh` | Cross-compile Rust for iOS (staticlib) + Android (cdylib) |

### Building native libraries

```bash
# iOS (requires rustup targets: aarch64-apple-ios, aarch64-apple-ios-sim)
cd sdk/react-native && ./scripts/build-rust.sh ios

# Android (requires ANDROID_NDK_HOME)
cd sdk/react-native && ./scripts/build-rust.sh android

# Both
cd sdk/react-native && ./scripts/build-rust.sh all
```

### Publishing to npm

```bash
cd sdk/react-native
npm run build          # TypeScript → build/
npm publish --access public
```

The `--access public` flag is required for scoped packages on first publish.

---

## Registry Immutability Summary

All four registries are effectively immutable once a version is published:

- **crates.io** — `cargo yank` removes from resolver but the tarball stays forever
- **pub.dev** — No retraction mechanism at all
- **npm** — `npm unpublish` works within 72 hours, but only if no dependents
- **Maven Central** — No retraction mechanism. Once published, it's permanent.
- **Swift (onde-swift tags)** — Convention: never force-push tags. Delete + re-create if needed.

**Bottom line:** get the release right before publishing. Use `--dry-run` locally
and via `workflow_dispatch` before pushing a real tag.

---

*Update this skill when adding new SDK targets (Python, Ruby, Kotlin), changing
CI runners, or modifying the tag/version validation logic.*