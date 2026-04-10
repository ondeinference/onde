---
name: sdk-kotlin
description: Build, generate, and publish the Onde Inference Kotlin/Android SDK. Covers UniFFI Kotlin binding generation from the Rust onde crate, Android library project structure, cargo-ndk cross-compilation for all Android ABIs, Maven Central publishing via Vanniktech plugin, Android filesystem sandbox setup (HF_HOME / TMPDIR via Os.setenv), ProGuard consumer rules, GitHub Actions CI release workflow, and the Jetpack Compose example app.
allowed-tools: Read, Write, Edit, Glob, Grep, Bash
user-invocable: true
---

# Skill: Kotlin / Android SDK

## What This Skill Covers

Building the `onde-inference` Android AAR from Rust source using UniFFI, distributing it via Maven Central, and maintaining the Kotlin idiomatic wrapper on top of the generated bindings.

---

## Repository Layout

```
onde/
├── src/                                    # Rust source — UniFFI-annotated types and OndeChatEngine
├── uniffi-bindgen/                         # Standalone uniffi-bindgen CLI binary (pinned =0.31.0)
├── sdk/kotlin/
│   ├── settings.gradle.kts                 # Gradle root — includes :lib and :example
│   ├── build.gradle.kts                    # root — plugin declarations only (all versions live here)
│   ├── gradle.properties                   # GROUP, POM_ARTIFACT_ID, VERSION_NAME, signing/OSSRH
│   ├── gradle/wrapper/
│   │   └── gradle-wrapper.properties       # pinned Gradle 8.9
│   ├── lib/
│   │   ├── build.gradle.kts               # Android library + Vanniktech maven-publish
│   │   ├── consumer-rules.pro             # ProGuard: preserve UniFFI JNI bridge symbols
│   │   └── src/
│   │       ├── generated/kotlin/          # UniFFI-generated onde.kt  ← GITIGNORED
│   │       └── main/
│   │           ├── AndroidManifest.xml
│   │           ├── jniLibs/              # compiled .so per ABI  ← GITIGNORED
│   │           └── kotlin/com/ondeinference/onde/
│   │               └── OndeInference.kt  # Android idiomatic wrapper
│   ├── example/
│   │   ├── build.gradle.kts               # com.android.application — no plugin versions here
│   │   ├── proguard-rules.pro
│   │   ├── README.md
│   │   └── src/main/
│   │       ├── AndroidManifest.xml        # INTERNET permission
│   │       ├── res/values/
│   │       │   ├── strings.xml
│   │       │   └── themes.xml             # Material3 NoActionBar activity theme
│   │       └── kotlin/com/ondeinference/example/
│   │           ├── MainActivity.kt        # ComponentActivity — enableEdgeToEdge + setContent
│   │           ├── ChatViewModel.kt       # model lifecycle, streaming, history
│   │           └── ui/
│   │               ├── ChatScreen.kt      # full Compose UI
│   │               └── theme/Theme.kt     # dynamic colour + dark mode
│   └── scripts/
│       ├── build-android.sh              # Rust .so → jniLibs/ via cargo-ndk
│       └── generate-bindings.sh          # uniffi-bindgen → src/generated/kotlin/onde.kt
└── .github/workflows/
    └── release-sdk-kotlin.yml            # tag push → build → Maven Central publish
```

---

## Key Design Decisions

| Decision | Rationale |
|---|---|
| UniFFI Kotlin bindings — not hand-written JNI | Same Rust source as Swift/Dart, zero API drift |
| `src/generated/kotlin/` is gitignored | Regenerated from Rust on every release |
| `cargo-ndk` for cross-compilation | Handles NDK toolchain wiring for all 4 ABIs transparently |
| Vanniktech `maven-publish` plugin | Simplest path to Maven Central + signing with one plugin |
| `minSdk 26` | Required for `android.system.Os.setenv` to set `HF_HOME` |
| `OndeInference` wraps `OndeChatEngine` | Adds Android filesystem init, `Dispatchers.IO`, and `Flow` streaming |
| ProGuard consumer rules bundled in AAR | Prevents R8 from stripping UniFFI JNI bridge in consumer apps |
| JNA dependency required (`net.java.dev.jna:jna:5.14.0@aar`) | UniFFI 0.31.0 generates Kotlin bindings that use `com.sun.jna.*` for the FFI bridge. Despite the `com.sun` package name, JNA works on Android when pulled as `@aar`. Without it, you get `Unresolved reference 'sun'` on every import. |
| `:example` is a Gradle submodule, not a standalone project | Plugin versions are declared once in the root `build.gradle.kts` and inherited by all submodules — a standalone project would need versions on every plugin, and `project(":lib")` would not resolve |

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

The generated Kotlin package is `uniffi.onde.*`. The `OndeInference.kt` wrapper
re-exports these as `typealias` under `com.ondeinference.onde.*` so callers only
need one import.

---

## Android ABI Targets

| ABI | Rust triple | Typical device |
|---|---|---|
| `arm64-v8a` | `aarch64-linux-android` | Modern Android phones (primary) |
| `armeabi-v7a` | `armv7-linux-androideabi` | Older 32-bit ARM phones |
| `x86_64` | `x86_64-linux-android` | Android emulators (Intel/AMD) |
| `x86` | `i686-linux-android` | 32-bit emulators |

---

## Build Sequence (Manual / CI)

### 1. Prerequisites

```bash
# Rust Android targets
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android

# cargo-ndk
cargo install cargo-ndk

# Android NDK
export ANDROID_NDK_HOME=$HOME/Library/Android/sdk/ndk/26.1.10909125
```

### 2. Build .so files for all ABIs

```bash
./sdk/kotlin/scripts/build-android.sh          # release, all ABIs
./sdk/kotlin/scripts/build-android.sh --debug  # debug build
```

Internally uses:

```bash
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 -t x86 \
    --manifest-path Cargo.toml --release
```

Outputs land in `sdk/kotlin/lib/src/main/jniLibs/<ABI>/libonde.so`.

### 3. Build the uniffi-bindgen CLI

```bash
cargo build --manifest-path uniffi-bindgen/Cargo.toml --release
# Binary: uniffi-bindgen/target/release/uniffi-bindgen
```

### 4. Generate Kotlin UniFFI bindings

```bash
./sdk/kotlin/scripts/generate-bindings.sh
```

Internally runs:

```bash
uniffi-bindgen/target/release/uniffi-bindgen generate \
    target/aarch64-linux-android/release/libonde.so \
    --language kotlin \
    --out-dir sdk/kotlin/lib/src/generated/kotlin
```

Produces `sdk/kotlin/lib/src/generated/kotlin/onde.kt` — the complete
generated binding including all types, the `OndeChatEngine` class, free
functions, and `System.loadLibrary("onde")` in a companion object static init.

### 5. Build the AAR

```bash
cd sdk/kotlin
./gradlew :lib:assembleRelease
# Output: lib/build/outputs/aar/lib-release.aar
```

---

## Android Filesystem Sandbox Setup

On Android, `dirs::home_dir()` (used by `hf-hub` under the hood) panics because
there is no home directory in the Android sandbox. The Rust engine requires
`HF_HOME` to be set **before** any model load on Android.

The `OndeInference` Kotlin wrapper handles this automatically in `setup()`:

```kotlin
fun setup() {
    if (configured) return

    val hfHome     = File(dataDir, "models").also { it.mkdirs() }
    val hfHubCache = File(hfHome,  "hub").also   { it.mkdirs() }
    val tmpDir     = File(dataDir, "tmp").also   { it.mkdirs() }

    Os.setenv("HF_HOME",               hfHome.absolutePath,     true)
    Os.setenv("HF_HUB_CACHE",          hfHubCache.absolutePath, true)
    Os.setenv("HUGGINGFACE_HUB_CACHE", hfHubCache.absolutePath, true)
    Os.setenv("TMPDIR",                tmpDir.absolutePath,      true)

    configured = true
}
```

`setup()` is called automatically by `loadDefaultModel()` and `loadModel()`.
Callers can call it explicitly at app startup for tighter control.

### Filesystem layout inside filesDir

```
<filesDir>/
├── models/           ← HF_HOME
│   └── hub/          ← HF_HUB_CACHE  (GGUF blobs live here)
└── tmp/              ← TMPDIR
```

**This layout matches the shared convention used by the Tauri and Swift SDKs.**
On iOS/macOS, the equivalent paths are inside the App Group container at
`<group.com.ondeinference.apps>/models/hub/`. On Android, there is no app group
mechanism, so each app gets its own `filesDir`.

### Cross-platform model sharing comparison

| Platform | Mechanism | Shared across apps? | How it works |
|---|---|---|---|
| iOS / tvOS / macOS | App Groups (`group.com.ondeinference.apps`) | Yes — all apps signed by the same team that declare the same App Group entitlement share a single container directory | OS provides a shared `containerURL(forSecurityApplicationGroupIdentifier:)` path; all Onde apps read and write the same `models/hub/` within it |
| Android | Per-app `filesDir` sandbox | No — each app gets its own isolated copy | Linux UID isolation prevents one app from reading another's internal storage; no entitlement or manifest flag can opt in to sharing |

### Why Android can't do App Groups

Android's security model assigns each app a unique Linux UID at install time.
The kernel enforces file permissions — app A literally cannot open a file
descriptor inside app B's `/data/data/com.B/files/`. There is no platform
primitive to override this the way iOS App Groups do. The approaches that
_don't_ work:

- **`android:sharedUserId`** — deprecated in API 29, removed in API 33. Dead end.
- **`MANAGE_EXTERNAL_STORAGE`** — lets you write to `/sdcard/onde/models/` but Google Play rejects apps that use it unless they are file managers or antivirus tools.
- **SAF (Storage Access Framework)** — requires the user to manually pick a directory every time. Not viable for a background model load.

### What we do today

Each Onde-powered Android app downloads its own copy of the model (~941 MB for
Qwen 2.5 1.5B). This is what every major Android ML SDK does (TensorFlow Lite,
MediaPipe, ML Kit). Android devices typically ship with 128–256 GB storage, so
1 GB per app is tolerable. The SDK caches after first download, so the user only
pays the bandwidth cost once per app.

### Future: ContentProvider model manager

The Android-idiomatic way to share data across apps without special permissions
is a `ContentProvider`. The plan (not yet implemented):

1. The first Onde-powered app installed acts as the **model host**. It downloads
   models into its own `filesDir` as it does today.
2. It registers a `ContentProvider` at a well-known authority
   (e.g. `com.ondeinference.models`) that serves model files via
   `ParcelFileDescriptor`.
3. Other Onde-powered apps query the provider first. If a model is already
   cached by any sibling app, they get a read-only file descriptor — no
   download, no copy.
4. If the provider is not installed (no sibling app on the device), the app
   falls back to downloading its own copy.

```kotlin
// In the host app — exposes cached models to sibling apps
class OndeModelProvider : ContentProvider() {
    override fun openFile(uri: Uri, mode: String): ParcelFileDescriptor? {
        // uri = content://com.ondeinference.models/qwen25-1.5b
        val modelFile = File(context!!.filesDir, "models/hub/${uri.lastPathSegment}")
        if (!modelFile.exists()) return null
        return ParcelFileDescriptor.open(modelFile, ParcelFileDescriptor.MODE_READ_ONLY)
    }
}

// In any Onde-powered app — checks for a cached model before downloading
val uri = Uri.parse("content://com.ondeinference.models/qwen25-1.5b")
val pfd = contentResolver.openFileDescriptor(uri, "r")
if (pfd != null) {
    // model already downloaded by a sibling app — read from file descriptor
} else {
    // no sibling app has the model — download our own copy
}
```

No special permissions required, works on all API levels, no Google Play policy
issues. The only requirement is that at least one Onde app is installed on the
device. This is tracked but not yet implemented — ship with per-app caching for
now and add the provider when multiple production apps exist.

---

## Kotlin Public API

### `OndeInference` — primary entry point

```kotlin
import com.ondeinference.onde.OndeInference
import com.ondeinference.onde.OndeSampling
import com.ondeinference.onde.OndeModels
import com.ondeinference.onde.OndeMessage

val onde = OndeInference(context)          // pass applicationContext

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
OndeModels.default()     // platform default = Qwen 2.5 1.5B on Android
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
import com.vanniktech.maven.publish.AndroidSingleVariantLibrary
import com.vanniktech.maven.publish.SonatypeHost

android {
    namespace = "com.ondeinference.onde"
    compileSdk = 35
    defaultConfig { minSdk = 26 }

    sourceSets {
        getByName("main") {
            jniLibs.srcDirs("src/main/jniLibs")
            kotlin.srcDirs("src/main/kotlin", "src/generated/kotlin")
        }
    }

    // Do NOT add android { publishing { singleVariant("release") } } here.
    // AndroidSingleVariantLibrary in mavenPublishing already calls it — doing
    // both causes "singleVariant publishing DSL multiple times" at sync time.
}

mavenPublishing {
    // This is the correct way to configure variant + sources/javadoc for an
    // Android library with Vanniktech 0.25+. It registers singleVariant internally.
    configure(
        AndroidSingleVariantLibrary(
            variant = "release",
            sourcesJar = true,
            publishJavadocJar = true,
        )
    )
    publishToMavenCentral(SonatypeHost.CENTRAL_PORTAL)
    signAllPublications()
    coordinates("com.ondeinference", "onde-inference", "<version>")
    pom {
        name.set("Onde Inference")
        // ... licenses, developers, scm
    }
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

## Maven Central Publishing (CI)

Trigger: push a semver tag matching `[0-9]+.[0-9]+.[0-9]+`.

```bash
# 1. Bump VERSION_NAME in sdk/kotlin/gradle.properties
# 2. Bump version in Cargo.toml [package]
# 3. Commit, tag, push
git tag 0.1.4 && git push origin 0.1.4
```

CI (`release-sdk-kotlin.yml`) steps:
1. Install Rust stable with all 4 Android targets
2. Cache NDK 26.1.10909125
3. Install `cargo-ndk`
4. `build-android.sh --release` → `.so` files
5. Build `uniffi-bindgen`
6. `generate-bindings.sh` → `onde.kt`
7. Validate tag == `Cargo.toml` version
8. `./gradlew :lib:publishAndReleaseToMavenCentral`
9. Upload AAR as GitHub Release asset

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
| `.so` files missing from AAR | Run `scripts/build-android.sh`. The `jniLibs/` directory is gitignored. |
| UniFFI version mismatch | Keep `uniffi = "=0.31.0"` identical in `Cargo.toml`, `build-deps`, and `uniffi-bindgen/Cargo.toml`. |
| R8 strips UniFFI bridge at runtime | Ensure `consumer-rules.pro` is present in the AAR and includes `-keep class uniffi.onde.**`. |
| `System.loadLibrary("onde")` fails | The `.so` files must match the ABI of the device. Check that all 4 ABIs are built and included in `jniLibs/`. |
| `Os.setenv` not available | `minSdk` must be 26+. `android.system.Os` is available from API 21 but `setenv` specifically needs 26. |
| Gradle fails with `VERSION_NAME` missing | Ensure `gradle.properties` contains `VERSION_NAME=x.y.z`. |
| `cargo-ndk` not found in CI | Add `cargo install cargo-ndk --locked` step before the build step. |
| `ANDROID_NDK_HOME` not set | Set it explicitly: `echo "ANDROID_NDK_HOME=$ANDROID_SDK_ROOT/ndk/26.1.10909125" >> $GITHUB_ENV` |
| `build-android.sh: line XX: arm64: unbound variable` on macOS | Do not use Bash associative arrays in `sdk/kotlin/scripts/build-android.sh`. macOS often ships an older Bash where associative arrays are unreliable or unavailable. Use plain indexed arrays (or another portable structure) for ABI → Rust target mapping. |
| `Unresolved reference 'sun'` / `Unresolved reference 'Structure'` in generated `onde.kt` | UniFFI 0.31.0 generates Kotlin bindings that depend on JNA (`com.sun.jna.*`). Add `implementation("net.java.dev.jna:jna:5.14.0@aar")` to `lib/build.gradle.kts`. The `@aar` classifier is required on Android — a plain `.jar` will not work. Despite the `com.sun` package name, JNA is a third-party library, not a JDK internal, and works fine on Android. |
| HuggingFace download fails on first load | The app must declare `<uses-permission android:name="android.permission.INTERNET" />`. |
| `EngineStatus` values are lowercase (`"ready"`) | The Rust `Display` impl emits lowercase. Normalize in the UI layer: `info.status.toString().lowercase()` or compare with the generated sealed class variants directly. |
| `Plugin [id: 'com.android.application'] was not found` in `:example` | The example has no plugin versions of its own — they are inherited from the root `build.gradle.kts`. Open `sdk/kotlin/` (not `sdk/kotlin/example/`) in Android Studio. Never add a `settings.gradle.kts` inside `example/` — doing so makes Gradle treat it as a standalone root project, breaking plugin resolution and `project(":lib")`. |
| `Task 'wrapper' not found in project ':example'` | Same cause as above — Android Studio was opened at the wrong directory. The `wrapper` task only exists on the root project. Open `sdk/kotlin/`. |
| `Using singleVariant publishing DSL multiple times to publish variant "release"` | Vanniktech 0.25+ registers `singleVariant("release")` internally via `AndroidSingleVariantLibrary`. Do **not** also add `android { publishing { singleVariant("release") { ... } } }` — that registers it a second time and causes this error. Remove the `android { publishing }` block entirely and configure sources/javadoc through `mavenPublishing` instead: `configure(AndroidSingleVariantLibrary(variant = "release", sourcesJar = true, publishJavadocJar = true))`. |

---

## Testing

Unit tests can be run on the host JVM (no Android device needed) for pure Kotlin
logic, but any test that calls into Rust via the generated bindings requires a
connected Android device or emulator because `System.loadLibrary("onde")` only
works with an Android runtime.

```bash
# Kotlin-only unit tests (host JVM)
cd sdk/kotlin
./gradlew :lib:test

# Instrumented tests (requires connected device or emulator)
./gradlew :lib:connectedAndroidTest
```

For CI, use an emulator (`x86_64` ABI) spun up with `reactivecircus/android-emulator-runner`.

---

## Dependency Graph

```
OndeInference.kt
    └── uniffi.onde.OndeChatEngine         (generated onde.kt)
    └── uniffi.onde.StreamChunkListener    (generated onde.kt)
    └── uniffi.onde.*Config / *Message     (generated onde.kt)
            └── com.sun.jna.Native         (JNA — net.java.dev.jna:jna:5.14.0@aar)
                    └── libonde.so         (Rust, per ABI)
                            └── mistralrs (Candle CPU backend on Android)
                            └── hf-hub    (model download)
```

---

## Distribution Registry

| Registry | Artifact | Import |
|---|---|---|
| Maven Central | `com.ondeinference:onde-inference` | `implementation("com.ondeinference:onde-inference:0.1.3")` |
| GitHub Releases | `onde-inference-<version>.aar` | Direct AAR download |

---

## Example App

A working Jetpack Compose chat app lives at `sdk/kotlin/example/`. It is the canonical way to see the SDK in action and is the first thing to open when verifying a new build.

### Opening in Android Studio

Always open **`sdk/kotlin/`** — the Gradle root. Do not open `sdk/kotlin/example/` directly. Android Studio will see both `:lib` and `:example` as modules and sync correctly.

### What the example covers

- Loading the default model with `loadDefaultModel()` and showing load time
- Streaming replies token-by-token via `onde.stream()` and `Flow<StreamChunk>`
- Multi-turn conversation history maintained in the Rust engine
- Clearing history without reloading the model
- Material3 dynamic colour theme with dark mode support
- Proper IME / navigation-bar inset handling with `imePadding()` + `navigationBarsPadding()`

### Gradle submodule rules

`example/` is a Gradle submodule of the `sdk/kotlin/` root project. The rules that must always hold:

1. **No `settings.gradle.kts` inside `example/`** — if one exists, Gradle stops its upward search there and treats `example/` as a standalone root, breaking plugin resolution and `project(":lib")`.
2. **No plugin versions in `example/build.gradle.kts`** — versions are declared once in the root `build.gradle.kts` with `apply false` and inherited by submodules.
3. **`include(":example")` in root `settings.gradle.kts`** — required for Gradle to recognise `:example` as a subproject.
4. **`project(":lib")` as the SDK dependency** — the example always builds against local source, not the published Maven artifact. This means the `.so` files and `onde.kt` must be generated before the example will compile.

The root `build.gradle.kts` must declare all plugins used by any submodule:

```kotlin
plugins {
    id("com.android.library")     version "8.5.2"  apply false  // :lib
    id("com.android.application") version "8.5.2"  apply false  // :example
    id("org.jetbrains.kotlin.android")       version "2.0.21" apply false
    id("org.jetbrains.kotlin.plugin.compose") version "2.0.21" apply false  // :example
    id("com.vanniktech.maven.publish")       version "0.28.0" apply false  // :lib
}
```

### Example architecture

```
ChatScreen (Compose — pure function of ChatUiState)
    ↓ collectAsStateWithLifecycle
ChatViewModel (AndroidViewModel + viewModelScope)
    ↓ loadModel()   → onde.loadDefaultModel(...)
    ↓ sendMessage() → onde.stream(...).collect { ... }
    ↓ clearChat()   → onde.clearHistory()
OndeInference  ← project(":lib")
    ↓ UniFFI / libonde.so
mistral.rs — Qwen 2.5 1.5B, Candle CPU backend
```

### Streaming pattern

Tokens are accumulated into a `StringBuilder` and the last item in the message list is replaced in-place on every chunk, so Compose only recomposes the one bubble that is changing:

```kotlin
val placeholderId = System.nanoTime()
// insert empty bubble once
_uiState.update { it.copy(messages = it.messages + UiMessage(id = placeholderId, ...)) }

val buffer = StringBuilder()
onde.stream(text).collect { chunk ->
    buffer.append(chunk.delta)
    // swap only the tail — no full list rebuild
    _uiState.update { state ->
        state.copy(messages = state.messages.replaceLast(
            UiMessage(id = placeholderId, content = buffer.toString(), isStreaming = !chunk.done)
        ))
    }
}
```

### Running the example

`build-android.sh` must stay portable on macOS. Avoid Bash associative arrays in that script — use indexed arrays instead.

```bash
cd sdk/kotlin

# 1. Build the .so files (needed by :lib, which :example depends on)
./scripts/build-android.sh

# 2. Generate the Kotlin bindings
./scripts/generate-bindings.sh

# 3. Install on a connected device
./gradlew :example:installDebug
```

Or open `sdk/kotlin/` in Android Studio and hit Run on the `:example` configuration.