---
name: sdk-kotlin
description: Build, generate, and publish the Onde Inference Kotlin Multiplatform SDK (Android + JVM). Covers UniFFI Kotlin binding generation, KMP source set layout with shared srcDir pattern, cargo-ndk cross-compilation for Android ABIs, JVM native library bundling for macOS Apple Silicon, Maven Central publishing via Vanniktech plugin, klibs.io discovery requirements, ProGuard consumer rules, and the Jetpack Compose example app.
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
user-invocable: true
---

# Skill: Kotlin Multiplatform SDK

## What This Skill Covers

Building the `onde-inference` Kotlin Multiplatform library from Rust source
using UniFFI, distributing it via Maven Central, and getting it listed on
[klibs.io](https://klibs.io) — the JetBrains-maintained discovery platform for
KMP libraries.

**Targets:**

| Target | GPU Backend | Default Model | Primary Use Case |
|--------|-------------|---------------|------------------|
| `androidTarget()` | CPU (Candle) | Qwen 2.5 1.5B (~941 MB) | Mobile apps |
| `jvm()` | Metal (macOS) / CPU (Linux) | Qwen 2.5 3B (~1.93 GB) | Desktop / server on Apple Silicon |

**Context:** Onde's tagline is "AI for Apple silicon devices." The primary SDKs
are **Swift** (iOS, tvOS, macOS) and **Dart** (Flutter, cross-platform). The
Kotlin SDK extends Onde to Android and JVM developers — particularly those
building KMP apps that target both Android and macOS, or running inference on
Apple Silicon Macs from Kotlin/JVM.

---

## Repository Layout

```
onde/
├── src/                                       # Rust source — UniFFI-annotated types and OndeChatEngine
├── uniffi-bindgen/                            # Standalone uniffi-bindgen CLI binary (pinned =0.31.0)
├── sdk/kotlin/
│   ├── settings.gradle.kts                    # Gradle root — includes :lib and :example
│   ├── build.gradle.kts                       # root — plugin declarations (all versions live here)
│   ├── gradle.properties                      # GROUP, POM_ARTIFACT_ID, VERSION_NAME, signing/OSSRH
│   ├── gradle/wrapper/
│   │   └── gradle-wrapper.properties          # pinned Gradle 8.9
│   ├── lib/
│   │   ├── build.gradle.kts                   # KMP library + Vanniktech maven-publish
│   │   ├── consumer-rules.pro                 # ProGuard: preserve UniFFI JNI bridge symbols
│   │   └── src/
│   │       ├── commonMain/kotlin/.gitkeep     # empty — KMP requires this source set to exist
│   │       ├── shared/kotlin/                 # shared srcDir added to BOTH androidMain and jvmMain
│   │       │   └── com/ondeinference/onde/
│   │       │       ├── PlatformSupport.kt     # internal interface — setEnv + ensureNativeLoaded
│   │       │       ├── OndeInference.kt       # engine wrapper (internal constructor)
│   │       │       └── Convenience.kt         # OndeSampling, OndeModels, OndeMessage
│   │       ├── generated/kotlin/              # UniFFI-generated onde.kt  ← GITIGNORED
│   │       ├── androidMain/
│   │       │   ├── AndroidManifest.xml
│   │       │   ├── jniLibs/                   # compiled .so per ABI  ← GITIGNORED
│   │       │   └── kotlin/com/ondeinference/onde/
│   │       │       └── Platform.android.kt    # AndroidPlatform + factory fun OndeInference(context)
│   │       └── jvmMain/
│   │           ├── kotlin/com/ondeinference/onde/
│   │           │   ├── Platform.jvm.kt        # JvmPlatform + factory fun OndeInference(dataDir)
│   │           │   └── NativeLoader.kt        # extract libonde from JAR resources
│   │           └── resources/native/          # bundled .dylib/.so  ← GITIGNORED
│   │               ├── macos-aarch64/libonde.dylib
│   │               ├── macos-x86_64/libonde.dylib
│   │               ├── linux-x86_64/libonde.so
│   │               └── linux-aarch64/libonde.so
│   ├── example/
│   │   ├── build.gradle.kts                   # com.android.application — no plugin versions here
│   │   ├── proguard-rules.pro
│   │   ├── README.md
│   │   └── src/main/
│   │       ├── AndroidManifest.xml            # INTERNET permission
│   │       ├── res/values/
│   │       │   ├── strings.xml
│   │       │   └── themes.xml                 # Material3 NoActionBar activity theme
│   │       └── kotlin/com/ondeinference/example/
│   │           ├── MainActivity.kt            # ComponentActivity — enableEdgeToEdge + setContent
│   │           ├── ChatViewModel.kt           # model lifecycle, streaming, history
│   │           └── ui/
│   │               ├── ChatScreen.kt          # full Compose UI
│   │               └── theme/Theme.kt         # dynamic colour + dark mode
│   └── scripts/
│       ├── build-android.sh                   # Rust .so → androidMain/jniLibs/ via cargo-ndk
│       ├── build-jvm.sh                       # Rust .dylib/.so → jvmMain/resources/native/
│       └── generate-bindings.sh               # uniffi-bindgen → src/generated/kotlin/onde.kt
└── .github/workflows/
    └── release-sdk-kotlin.yml                 # tag push → build → Maven Central publish
```

---

## KMP Architecture

### Why KMP Instead of Android-Only

1. **klibs.io listing** — the JetBrains KMP discovery platform requires
   `kotlin-tooling-metadata.json`, which only the `kotlin.multiplatform` Gradle
   plugin generates. An Android-only AAR will not appear on klibs.io.
2. **JVM target** — Kotlin developers on macOS Apple Silicon can run inference
   locally from desktop/server apps, Gradle plugins, or CLI tools.
3. **Future-proof** — adding Kotlin/Native targets (iOS, macOS) later requires
   no structural changes. The Swift SDK already covers Apple platforms natively,
   but KMP apps that share Kotlin across Android + iOS could benefit.

### Source Set Strategy: Shared srcDir

Instead of KMP intermediate source sets (which add Gradle complexity and
require careful dependency resolution for JNA), the SDK uses a **shared srcDir**
pattern:

```
commonMain/   — empty (KMP requires it)
shared/       — hand-written wrapper code (compiled for BOTH targets)
generated/    — UniFFI-generated onde.kt (compiled for BOTH targets)
androidMain/  — Android-specific: Os.setenv, Context factory
jvmMain/      — JVM-specific: JNA setenv, NativeLoader, File factory
```

In `lib/build.gradle.kts`:

```kotlin
androidMain.get().kotlin.srcDir("src/shared/kotlin")
androidMain.get().kotlin.srcDir("src/generated/kotlin")
jvmMain.get().kotlin.srcDir("src/shared/kotlin")
jvmMain.get().kotlin.srcDir("src/generated/kotlin")
```

Both targets compile the shared code independently with their own dependencies.
Since both are JVM-based, `java.io.File`, JNA, and coroutines are available in
the shared code.

### Platform Abstraction: Interface Pattern

The shared `OndeInference` class takes a `PlatformSupport` interface via an
`internal` constructor. Each target provides a concrete implementation and a
top-level factory function:

```
PlatformSupport (interface — shared/)
├── AndroidPlatform (object — androidMain/)
│   └── fun OndeInference(context, dataDir?) → OndeInference
└── JvmPlatform (object — jvmMain/)
    └── fun OndeInference(dataDir?) → OndeInference
```

| Operation | Android | JVM |
|-----------|---------|-----|
| `setEnv(key, value)` | `Os.setenv(key, value, true)` | JNA → `libc.setenv(key, value, 1)` |
| `ensureNativeLoaded()` | No-op (APK jniLibs) | `NativeLoader.ensureLoaded()` |
| Default `dataDir` | `context.filesDir` | `~/.onde/` |

---

## Key Design Decisions

| Decision | Rationale |
|---|---|
| KMP with `androidTarget()` + `jvm()` | Required for klibs.io listing; enables macOS Apple Silicon JVM users |
| Shared srcDir instead of intermediate source sets | Avoids KMP dependency resolution complexity with JNA (`@aar` vs JAR) |
| Interface-based platform abstraction | Simpler than `expect`/`actual` when constructor signatures differ between targets |
| UniFFI Kotlin bindings — not hand-written JNI | Same Rust source as Swift/Dart, zero API drift |
| `src/generated/kotlin/` is gitignored | Regenerated from Rust on every release |
| `cargo-ndk` for Android cross-compilation | Handles NDK toolchain wiring for all 4 ABIs transparently |
| `NativeLoader` extracts dylib from JAR resources | JVM users don't need to manage `java.library.path` manually |
| Vanniktech `maven-publish` plugin with `KotlinMultiplatform` | Publishes KMP metadata + per-target artifacts to Maven Central |
| `minSdk 26` | Required for `android.system.Os.setenv` to set `HF_HOME` |
| JNA dependency (`net.java.dev.jna:jna:5.14.0`) | UniFFI 0.31.0 generates Kotlin bindings using `com.sun.jna.*`. Android uses `@aar` variant; JVM uses plain JAR. |
| ProGuard consumer rules bundled in AAR | Prevents R8 from stripping UniFFI JNI bridge in consumer apps |
| `:example` is a Gradle submodule | Plugin versions declared once in root `build.gradle.kts` and inherited |

---

## UniFFI Kotlin Type Map

| Rust type | UniFFI derive | Kotlin type |
|---|---|---|
| `ChatRole` | `uniffi::Enum` | `sealed class ChatRole` |
| `ChatMessage` | `uniffi::Record` | `data class ChatMessage` |
| `SamplingConfig` | `uniffi::Record` | `data class SamplingConfig` |
| `GgufModelConfig` | `uniffi::Record` | `data class GgufModelConfig` |
| `InferenceResult` | `uniffi::Record` | `data class InferenceResult` |
| `StreamChunk` | `uniffi::Record` | `data class StreamChunk` |
| `EngineStatus` | `uniffi::Enum` | `sealed class EngineStatus` |
| `EngineInfo` | `uniffi::Record` | `data class EngineInfo` |
| `InferenceError` | `uniffi::Error` | `sealed class InferenceException : Exception` |
| `OndeChatEngine` | `uniffi::Object` | `class OndeChatEngine` |
| `StreamChunkListener` | `callback_interface` | `interface StreamChunkListener` |

The generated Kotlin package is `uniffi.onde.*`. The shared `OndeInference.kt`
re-exports these as `typealias` under `com.ondeinference.onde.*` so callers only
need one import.

---

## Build Targets

### Android ABI Targets

| ABI | Rust triple | Typical device |
|---|---|---|
| `arm64-v8a` | `aarch64-linux-android` | Modern Android phones (primary) |
| `armeabi-v7a` | `armv7-linux-androideabi` | Older 32-bit ARM phones |
| `x86_64` | `x86_64-linux-android` | Android emulators (Intel/AMD) |
| `x86` | `i686-linux-android` | 32-bit emulators |

### JVM Native Library Targets

| Platform | Rust triple | Library | Primary Use Case |
|---|---|---|---|
| macOS Apple Silicon | `aarch64-apple-darwin` | `libonde.dylib` | **Primary** — Mac development + Metal GPU |
| macOS Intel | `x86_64-apple-darwin` | `libonde.dylib` | Legacy Mac support |
| Linux x86_64 | `x86_64-unknown-linux-gnu` | `libonde.so` | CI, servers |
| Linux ARM64 | `aarch64-unknown-linux-gnu` | `libonde.so` | ARM servers |

---

## Build Sequence (Manual / CI)

### 1. Prerequisites

```bash
# Rust toolchain
rustup toolchain install stable

# Android targets + cargo-ndk
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android
cargo install cargo-ndk

# Android NDK
export ANDROID_NDK_HOME=$HOME/Library/Android/sdk/ndk/26.1.10909125

# macOS/JVM target (usually pre-installed on Apple Silicon)
rustup target add aarch64-apple-darwin
```

### 2. Build Android .so files

```bash
./sdk/kotlin/scripts/build-android.sh          # release, all ABIs
./sdk/kotlin/scripts/build-android.sh --debug  # debug build
FILTER_ABI=arm64-v8a ./sdk/kotlin/scripts/build-android.sh  # single ABI
```

Outputs land in `sdk/kotlin/lib/src/androidMain/jniLibs/<ABI>/libonde.so`.

### 3. Build JVM native library

```bash
./sdk/kotlin/scripts/build-jvm.sh              # host platform, release
./sdk/kotlin/scripts/build-jvm.sh --debug      # debug build
ONDE_TARGET_TRIPLE=x86_64-apple-darwin ./sdk/kotlin/scripts/build-jvm.sh  # cross-compile
```

Outputs land in `sdk/kotlin/lib/src/jvmMain/resources/native/<os-arch>/libonde.<ext>`.

### 4. Build the uniffi-bindgen CLI

```bash
cargo build --manifest-path uniffi-bindgen/Cargo.toml --release
# Binary: uniffi-bindgen/target/release/uniffi-bindgen
```

### 5. Generate Kotlin UniFFI bindings

```bash
./sdk/kotlin/scripts/generate-bindings.sh
```

Produces `sdk/kotlin/lib/src/generated/kotlin/onde.kt` — the complete
generated binding including all types, the `OndeChatEngine` class, free
functions, and `System.loadLibrary("onde")` in a companion object static init.

### 6. Build the library

```bash
cd sdk/kotlin

# Android AAR
./gradlew :lib:assembleRelease

# JVM JAR (with bundled native lib)
./gradlew :lib:jvmJar

# Both
./gradlew :lib:build
```

---

## JVM Native Library Loading

The JVM target bundles native libraries inside the JAR under
`/native/<os>-<arch>/libonde.<ext>`. The `NativeLoader` object handles
extraction and loading at runtime:

```
OndeInference() factory (jvmMain)
  └── JvmPlatform.ensureNativeLoaded()
        └── NativeLoader.ensureLoaded()
              ├── Detect OS + arch from system properties
              ├── Extract /native/<os>-<arch>/libonde.<ext> from JAR resources
              ├── Write to temp file: <java.io.tmpdir>/onde-native/libonde.<ext>
              └── System.load(tempFile.absolutePath)
```

If the bundled library is not found (e.g. development mode without running
`build-jvm.sh`), `NativeLoader` falls back to `System.loadLibrary("onde")`
which searches `java.library.path`.

**Important:** `NativeLoader.ensureLoaded()` must run BEFORE any UniFFI type is
accessed. The `OndeInference` constructor handles this via the `init` block:

```kotlin
class OndeInference internal constructor(...) : AutoCloseable {
    init { platform.ensureNativeLoaded() }  // runs before engine = OndeChatEngine()
    private val engine = uniffi.onde.OndeChatEngine()
}
```

---

## Android Filesystem Sandbox Setup

On Android, `dirs::home_dir()` (used by `hf-hub` under the hood) panics because
there is no home directory in the Android sandbox. The Rust engine requires
`HF_HOME` to be set **before** any model load on Android.

The `OndeInference` wrapper handles this automatically in `setup()`:

```kotlin
fun setup() {
    if (configured) return

    val hfHome     = File(dataDir, "models").also { it.mkdirs() }
    val hfHubCache = File(hfHome,  "hub").also   { it.mkdirs() }
    val tmpDir     = File(dataDir, "tmp").also   { it.mkdirs() }

    platform.setEnv("HF_HOME",               hfHome.absolutePath)
    platform.setEnv("HF_HUB_CACHE",          hfHubCache.absolutePath)
    platform.setEnv("HUGGINGFACE_HUB_CACHE", hfHubCache.absolutePath)
    platform.setEnv("TMPDIR",                tmpDir.absolutePath)

    configured = true
}
```

`setup()` is called automatically by `loadDefaultModel()` and `loadModel()`.

### Filesystem layout

```
# Android (context.filesDir)
<filesDir>/
├── models/           ← HF_HOME
│   └── hub/          ← HF_HUB_CACHE
└── tmp/              ← TMPDIR

# JVM (defaults to ~/.onde/)
~/.onde/
├── models/           ← HF_HOME
│   └── hub/          ← HF_HUB_CACHE
└── tmp/              ← TMPDIR
```

### Cross-platform model cache comparison

| Platform | SDK | Cache Location | Shared across apps? |
|---|---|---|---|
| iOS / tvOS / macOS | Swift | App Group container | Yes (same team + entitlement) |
| macOS (JVM) | Kotlin | `~/.onde/models/hub/` | Yes (all JVM apps on same user) |
| Android | Kotlin | `<filesDir>/models/hub/` | No (per-app sandbox) |
| Flutter | Dart | Platform-dependent | Follows platform conventions |

### Why Android can't share models across apps

Android's security model assigns each app a unique Linux UID at install time.
The kernel enforces file permissions — app A cannot open a file descriptor
inside app B's `/data/data/com.B/files/`. There is no platform primitive to
override this the way iOS App Groups do. The approaches that don't work:

- **`android:sharedUserId`** — deprecated API 29, removed API 33
- **`MANAGE_EXTERNAL_STORAGE`** — Google Play rejects unless file manager
- **SAF (Storage Access Framework)** — requires user interaction every time

Each Onde-powered Android app downloads its own copy (~941 MB). This matches
how TensorFlow Lite, MediaPipe, and ML Kit handle model caching. The SDK caches
after first download — bandwidth cost is one-time per app.

---

## Kotlin Public API

### `OndeInference` — primary entry point

```kotlin
import com.ondeinference.onde.OndeInference
import com.ondeinference.onde.OndeSampling
import com.ondeinference.onde.OndeModels
import com.ondeinference.onde.OndeMessage

// ── Android ──────────────────────────────────────────────────
val onde = OndeInference(context)          // pass applicationContext

// ── JVM (macOS / Linux) ──────────────────────────────────────
val onde = OndeInference()                 // uses ~/.onde/
val onde = OndeInference(File("/custom"))  // custom cache dir

// ── Shared API (both platforms) ──────────────────────────────

// Load (downloads from HuggingFace Hub on first run)
val elapsed: Double = onde.loadDefaultModel(
    systemPrompt = "You are helpful.",
    sampling = OndeSampling.mobile()
)

// Chat (suspend)
val result = onde.chat("What is Rust?")
println(result.text)

// Streaming (Flow)
onde.stream("Tell me a story.").collect { chunk ->
    print(chunk.delta)
}

// One-shot (no history side-effects)
val summary = onde.generate(
    messages = listOf(
        OndeMessage.system("Summarise this."),
        OndeMessage.user(longText)
    )
)

// Status
val info = onde.info()    // EngineInfo

// History
onde.clearHistory()
onde.pushHistory(OndeMessage.user("Injected context"))

// Cleanup
onde.unload()
onde.close()              // decrements Rust Arc refcount
```

### `OndeSampling` — convenience constructors

```kotlin
OndeSampling.default()        // temp=0.7, top_p=0.95, max=512 tokens
OndeSampling.deterministic()  // temp=0.0, greedy
OndeSampling.mobile()         // temp=0.7, max=128 tokens
```

### `OndeModels` — model config constructors

```kotlin
OndeModels.default()     // platform default (1.5B on Android, 3B on macOS JVM)
OndeModels.qwen25_1_5b() // ~941 MB
OndeModels.qwen25_3b()   // ~1.93 GB
```

### `OndeMessage` — message constructors

```kotlin
OndeMessage.system("You are helpful.")
OndeMessage.user("Hello!")
OndeMessage.assistant("Hi there!")
```

---

## lib/build.gradle.kts Key Sections

```kotlin
import com.vanniktech.maven.publish.KotlinMultiplatform
import com.vanniktech.maven.publish.JavadocJar
import com.vanniktech.maven.publish.SonatypeHost

plugins {
    id("org.jetbrains.kotlin.multiplatform")
    id("com.android.library")
    id("com.vanniktech.maven.publish")
}

kotlin {
    androidTarget {
        publishLibraryVariants("release")
        compilations.all { kotlinOptions { jvmTarget = "17" } }
    }
    jvm {
        compilations.all { kotlinOptions { jvmTarget = "17" } }
    }

    sourceSets {
        androidMain.dependencies {
            implementation("net.java.dev.jna:jna:5.14.0@aar")
            implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.8.1")
        }
        jvmMain.dependencies {
            implementation("net.java.dev.jna:jna:5.14.0")
            implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.8.1")
        }

        // Shared srcDirs — compiled independently for each target
        androidMain.get().kotlin.srcDir("src/shared/kotlin")
        androidMain.get().kotlin.srcDir("src/generated/kotlin")
        jvmMain.get().kotlin.srcDir("src/shared/kotlin")
        jvmMain.get().kotlin.srcDir("src/generated/kotlin")
    }
}

mavenPublishing {
    configure(KotlinMultiplatform(
        javadocJar = JavadocJar.Empty(),
        sourcesJar = true,
    ))
    publishToMavenCentral(SonatypeHost.CENTRAL_PORTAL)
    signAllPublications()
    coordinates("com.ondeinference", "onde-inference", "<version>")
}
```

The Vanniktech plugin reads signing and OSSRH credentials from environment
variables prefixed with `ORG_GRADLE_PROJECT_`:

| Gradle property | CI env var |
|---|---|
| `mavenCentralUsername` | `ORG_GRADLE_PROJECT_mavenCentralUsername` |
| `mavenCentralPassword` | `ORG_GRADLE_PROJECT_mavenCentralPassword` |
| `signingKeyId` | `ORG_GRADLE_PROJECT_signingKeyId` |
| `signingKey` | `ORG_GRADLE_PROJECT_signingKey` |
| `signingPassword` | `ORG_GRADLE_PROJECT_signingPassword` |

---

## ProGuard Consumer Rules

The `consumer-rules.pro` bundled in the AAR instructs R8/ProGuard in consumer
apps to preserve:

- `uniffi.onde.**` — all generated binding classes
- `com.ondeinference.onde.**` — the public wrapper
- `uniffi.onde.StreamChunkListener` — callback interface called from Rust
- Native method declarations (`keepclasseswithmembernames`)
- Static initializers in `uniffi.onde.*` (contains `System.loadLibrary("onde")`)

Always update `consumer-rules.pro` when adding new top-level UniFFI-exported
types or callback interfaces.

---

## klibs.io Discovery

[klibs.io](https://klibs.io) is a JetBrains-maintained search platform for
Kotlin Multiplatform libraries. It auto-indexes from Maven Central.

### Requirements for listing

| Requirement | How we satisfy it |
|---|---|
| Open source on GitHub | `ondeinference/onde` is public |
| At least one artifact on Maven Central | Published via Vanniktech plugin |
| Artifact has `kotlin-tooling-metadata.json` | KMP plugin generates this automatically |
| POM has valid GitHub `url` or `scm.url` | `scm { url.set("https://github.com/ondeinference/onde") }` |

### Timeline

- New libraries appear **within one month** of first Maven Central publish
  (public index update frequency).
- New versions of already-indexed libraries appear **the next day**.
- No manual submission needed — discovery is fully automatic.

---

## Maven Central Publishing (CI)

Trigger: push a semver tag matching `[0-9]+.[0-9]+.[0-9]+`.

```bash
# 1. Bump VERSION_NAME in sdk/kotlin/gradle.properties
# 2. Bump version in Cargo.toml [package]
# 3. Commit, tag, push
git tag 1.0.1 && git push origin 1.0.1
```

CI (`release-sdk-kotlin.yml`) steps:
1. Install Rust stable with all 4 Android targets + host target
2. Cache NDK 26.1.10909125
3. Install `cargo-ndk`
4. `build-android.sh --release` → `.so` files
5. `build-jvm.sh --release` → `.dylib` / `.so`
6. Build `uniffi-bindgen`
7. `generate-bindings.sh` → `onde.kt`
8. Validate tag == `Cargo.toml` version
9. `./gradlew :lib:publishAndReleaseToMavenCentral`
10. Upload artifacts as GitHub Release assets

---

## Version Synchronisation Rule

`VERSION_NAME` in `sdk/kotlin/gradle.properties` MUST match `version` in the
root `Cargo.toml`. The CI workflow enforces this and fails fast on mismatch.
Never bump one without the other.

---

## Common Pitfalls

| Pitfall | Fix |
|---|---|
| `dirs::home_dir()` panics on Android | `setup()` must be called before any Rust model load. `OndeInference.loadDefaultModel()` calls it automatically. |
| Generated `onde.kt` not found at compile time | Run `scripts/generate-bindings.sh`. The file is gitignored and must be regenerated after every Rust API change. |
| `.so` files missing from AAR | Run `scripts/build-android.sh`. The `androidMain/jniLibs/` directory is gitignored. |
| JVM `UnsatisfiedLinkError` at runtime | Run `scripts/build-jvm.sh` to bundle the native lib in JAR resources, or add `libonde.dylib` to `java.library.path`. |
| UniFFI version mismatch | Keep `uniffi = "=0.31.0"` identical in `Cargo.toml`, `build-deps`, and `uniffi-bindgen/Cargo.toml`. |
| R8 strips UniFFI bridge at runtime | Ensure `consumer-rules.pro` is present in the AAR and includes `-keep class uniffi.onde.**`. |
| `System.loadLibrary("onde")` fails | The `.so` files must match the ABI of the device. Check that all 4 ABIs are built and included in `androidMain/jniLibs/`. |
| `Os.setenv` not available | `minSdk` must be 26+. `android.system.Os` is available from API 21 but `setenv` specifically needs 26. |
| Gradle fails with `VERSION_NAME` missing | Ensure `gradle.properties` contains `VERSION_NAME=x.y.z`. |
| `cargo-ndk` not found in CI | Add `cargo install cargo-ndk --locked` step before the build step. |
| `ANDROID_NDK_HOME` not set | Set it explicitly: `echo "ANDROID_NDK_HOME=$ANDROID_SDK_ROOT/ndk/26.1.10909125" >> $GITHUB_ENV` |
| `build-android.sh: line XX: arm64: unbound variable` on macOS | Do not use Bash associative arrays. The script uses portable indexed arrays for ABI → Rust target mapping. |
| `Unresolved reference 'sun'` / `Unresolved reference 'Structure'` in `onde.kt` | UniFFI 0.31.0 depends on JNA (`com.sun.jna.*`). Add `implementation("net.java.dev.jna:jna:5.14.0@aar")` for Android or `implementation("net.java.dev.jna:jna:5.14.0")` for JVM. |
| HuggingFace download fails on first load | The app must declare `<uses-permission android:name="android.permission.INTERNET" />`. |
| `Plugin [id: 'com.android.application'] was not found` in `:example` | Open `sdk/kotlin/` (not `sdk/kotlin/example/`) in Android Studio. Plugin versions are inherited from the root `build.gradle.kts`. |
| `Using singleVariant publishing DSL multiple times` | This error means the old `AndroidSingleVariantLibrary` config is still present. Use `KotlinMultiplatform(...)` instead — the KMP plugin handles variant publishing internally. |
| Convenience functions (`OndeSampling.default()` etc.) crash before `OndeInference()` created | These functions call into the native library via UniFFI. Always create an `OndeInference` instance first — its `init` block ensures the native library is loaded. |

---

## Testing

### Unit tests (host JVM — no Android device needed)

Pure Kotlin logic tests that don't call into Rust:

```bash
cd sdk/kotlin
./gradlew :lib:test              # all JVM unit tests
./gradlew :lib:jvmTest           # JVM target only
```

### Instrumented tests (requires Android device or emulator)

Tests that call into Rust via the generated bindings:

```bash
./gradlew :lib:connectedAndroidTest
```

For CI, use an emulator (`x86_64` ABI) with `reactivecircus/android-emulator-runner`.

### JVM integration tests

Tests that load the native library and run inference:

```bash
# Build the native lib first
./sdk/kotlin/scripts/build-jvm.sh

# Run JVM tests
cd sdk/kotlin && ./gradlew :lib:jvmTest
```

---

## Dependency Graph

```
OndeInference.kt (shared)
├── PlatformSupport (interface, shared)
│   ├── AndroidPlatform (androidMain)
│   │   └── android.system.Os.setenv
│   └── JvmPlatform (jvmMain)
│       ├── JNA → libc.setenv
│       └── NativeLoader → extracts libonde from JAR resources
├── uniffi.onde.OndeChatEngine     (generated onde.kt)
├── uniffi.onde.StreamChunkListener (generated onde.kt)
├── uniffi.onde.*Config / *Message  (generated onde.kt)
│       └── com.sun.jna.Native     (JNA)
│               ├── libonde.so      (Android, per ABI)
│               └── libonde.dylib   (macOS, from JAR resources)
│                       └── mistralrs (Metal on macOS, Candle CPU on Android)
│                       └── hf-hub   (model download)
└── Convenience.kt (shared)
    └── OndeSampling, OndeModels, OndeMessage
```

---

## Distribution Registry

| Registry | Artifact | Import | Discovery |
|---|---|---|---|
| Maven Central | `com.ondeinference:onde-inference` | `implementation("com.ondeinference:onde-inference:1.0.0")` | [search.maven.org](https://search.maven.org) |
| klibs.io | `onde-inference` | Same Gradle coordinate | [klibs.io](https://klibs.io) (auto-indexed) |
| GitHub Releases | `onde-inference-<version>.aar` + `.jar` | Direct download | [github.com/ondeinference/onde/releases](https://github.com/ondeinference/onde/releases) |

### End-game for Kotlin developers

One line in `build.gradle.kts`:

```kotlin
implementation("com.ondeinference:onde-inference:1.0.0")
```

---

## Example App

A working Jetpack Compose chat app lives at `sdk/kotlin/example/`. It is the
canonical way to see the SDK in action and the first thing to open when
verifying a new build.

### Opening in Android Studio

Always open **`sdk/kotlin/`** — the Gradle root. Do not open
`sdk/kotlin/example/` directly. Android Studio will see both `:lib` and
`:example` as modules and sync correctly.

### What the example covers

- Loading the default model with `loadDefaultModel()` and showing load time
- Streaming replies token-by-token via `onde.stream()` and `Flow<StreamChunk>`
- Multi-turn conversation history maintained in the Rust engine
- Clearing history without reloading the model
- Material3 dynamic colour theme with dark mode support
- Proper IME / navigation-bar inset handling

### Gradle submodule rules

1. **No `settings.gradle.kts` inside `example/`** — Gradle would treat it as
   standalone, breaking plugin resolution and `project(":lib")`.
2. **No plugin versions in `example/build.gradle.kts`** — inherited from root.
3. **`include(":example")` in root `settings.gradle.kts`** — required for Gradle.
4. **`project(":lib")` as the SDK dependency** — builds against local source.

The root `build.gradle.kts` declares all plugins used by any submodule:

```kotlin
plugins {
    id("com.android.library")               version "8.5.2"  apply false
    id("com.android.application")           version "8.5.2"  apply false
    id("org.jetbrains.kotlin.android")      version "2.0.21" apply false
    id("org.jetbrains.kotlin.multiplatform") version "2.0.21" apply false
    id("org.jetbrains.kotlin.plugin.compose") version "2.0.21" apply false
    id("com.vanniktech.maven.publish")      version "0.28.0" apply false
}
```

### Streaming pattern

```kotlin
val placeholderId = System.nanoTime()
_uiState.update { it.copy(messages = it.messages + UiMessage(id = placeholderId, ...)) }

val buffer = StringBuilder()
onde.stream(text).collect { chunk ->
    buffer.append(chunk.delta)
    _uiState.update { state ->
        state.copy(messages = state.messages.replaceLast(
            UiMessage(id = placeholderId, content = buffer.toString(), isStreaming = !chunk.done)
        ))
    }
}
```

### Running the example

```bash
cd sdk/kotlin

# 1. Build the .so files
./scripts/build-android.sh

# 2. Generate the Kotlin bindings
./scripts/generate-bindings.sh

# 3. Install on a connected device
./gradlew :example:installDebug
```

Or open `sdk/kotlin/` in Android Studio and hit Run on the `:example` configuration.

---

## Cross-Platform SDK Comparison

Onde ships SDKs for four platforms. The Kotlin SDK is one piece of the
distribution strategy:

| SDK | Primary Platform | GPU Backend | Package Manager | Discovery |
|-----|-----------------|-------------|-----------------|-----------|
| **Swift** | iOS, tvOS, macOS | Metal | Swift Package Manager | Swift Package Index |
| **Dart** | Android, iOS (Flutter) | Platform-dependent | pub.dev | pub.dev search |
| **Kotlin** | Android, JVM (macOS) | Metal (JVM) / CPU (Android) | Maven Central | klibs.io |
| **React Native** | Android, iOS | Platform-dependent | npm | npmjs.com |

The Swift SDK is the primary distribution — it runs natively on Apple silicon
with Metal acceleration. The Kotlin JVM target provides the same Metal-backed
inference for Kotlin developers on macOS, while the Android target extends
reach to the Android ecosystem.

---

*Update this skill when adding KMP targets (e.g. Kotlin/Native for iOS),
changing the source set structure, or modifying the Maven Central publishing
pipeline.*