<p align="center">
  <img src="https://raw.githubusercontent.com/ondeinference/onde/main/assets/onde-inference-logo.svg" alt="Onde Inference" width="96">
</p>

<h1 align="center">Onde Inference — Kotlin / Android SDK</h1>

<p align="center">
  <strong>On-device LLM inference for Android. Run Qwen 2.5 models locally — no cloud, no API key, no data leaving the device.</strong>
</p>

<p align="center">
  <a href="https://central.sonatype.com/artifact/com.ondeinference/onde-inference"><img src="https://img.shields.io/maven-central/v/com.ondeinference/onde-inference" alt="Maven Central"></a>
  <a href="https://ondeinference.com"><img src="https://img.shields.io/badge/website-ondeinference.com-blue" alt="Website"></a>
  <a href="https://github.com/ondeinference/onde/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue" alt="License"></a>
</p>

---

## What is Onde?

Onde is an on-device LLM inference SDK for Android. It wraps [mistral.rs](https://github.com/EricLBuehler/mistral.rs) behind a clean Kotlin API — automatic model downloading from HuggingFace Hub, cache management, and CPU inference via the Candle backend.

- **On-device** — no cloud, no API key, no data leaving the device
- **Kotlin-first** — suspend functions and `Flow<StreamChunk>` streaming
- **UniFFI-powered** — the Rust engine is bound to Kotlin via [uniffi-rs](https://github.com/mozilla/uniffi-rs)
- **Maven Central** — single Gradle dependency, no manual `.so` management

---

## Installation

Add to your app's `build.gradle.kts`:

```kotlin
dependencies {
    implementation("com.ondeinference:onde-inference:0.1.3")
}
```

Add `INTERNET` permission to `AndroidManifest.xml` (required for the initial model download):

```xml
<uses-permission android:name="android.permission.INTERNET" />
```

**Minimum requirements**
- Android 8.0 (API 26) or higher
- ~1.1 GB free storage for the default model (Qwen 2.5 1.5B Q4_K_M)

---

## Quick start

```kotlin
import com.ondeinference.onde.OndeInference
import kotlinx.coroutines.launch

class MainActivity : AppCompatActivity() {

    private val onde by lazy { OndeInference(applicationContext) }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        lifecycleScope.launch {
            // 1. Load the default model (Qwen 2.5 1.5B on Android, ~941 MB)
            //    Downloads from HuggingFace Hub on first run (~1–5 min on Wi-Fi).
            val elapsed = onde.loadDefaultModel(
                systemPrompt = "You are a helpful, concise assistant."
            )
            println("Model loaded in ${elapsed}s")

            // 2. Chat
            val result = onde.chat("What is the capital of Sweden?")
            println(result.text) // → "The capital of Sweden is Stockholm."

            // 3. Clean up
            onde.unload()
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        onde.close()
    }
}
```

---

## Streaming

Receive tokens as they are generated using `Flow`:

```kotlin
lifecycleScope.launch {
    onde.stream("Write a haiku about the ocean.").collect { chunk ->
        // Append each token delta to your UI
        textView.append(chunk.delta)
        if (chunk.done) println("\n[done]")
    }
}
```

---

## Multi-turn conversation

The engine maintains conversation history automatically:

```kotlin
lifecycleScope.launch {
    onde.loadDefaultModel(systemPrompt = "You are a Rust tutor.")

    val r1 = onde.chat("What is ownership?")
    println(r1.text)

    val r2 = onde.chat("Can you give me a code example?")
    println(r2.text) // knows the context from r1

    // Clear history without unloading the model
    onde.clearHistory()
}
```

---

## Load a specific model

```kotlin
import com.ondeinference.onde.OndeModels

lifecycleScope.launch {
    // Qwen 2.5 1.5B — default on Android
    val elapsed = onde.loadModel(
        config = OndeModels.qwen25_1_5b(),
        systemPrompt = "You are a coding assistant."
    )
}
```

---

## Sampling configuration

```kotlin
import com.ondeinference.onde.OndeSampling

lifecycleScope.launch {
    // Default creative chat — temperature 0.7, top_p 0.95, max 512 tokens
    onde.loadDefaultModel(sampling = OndeSampling.default())

    // Greedy / deterministic — best for factual Q&A and code generation
    onde.loadDefaultModel(sampling = OndeSampling.deterministic())

    // Mobile-conservative — max 128 tokens, faster on low-end devices
    onde.loadDefaultModel(sampling = OndeSampling.mobile())
}
```

---

## One-shot generation (no history side-effects)

```kotlin
import com.ondeinference.onde.OndeMessage

lifecycleScope.launch {
    val result = onde.generate(
        messages = listOf(
            OndeMessage.system("You are a summariser."),
            OndeMessage.user("Summarise this text: $longArticle"),
        ),
        sampling = OndeSampling.deterministic()
    )
    println(result.text)
}
```

---

## Engine status

```kotlin
lifecycleScope.launch {
    val info = onde.info()
    println("Status: ${info.status}")          // Ready / Loading / Unloaded / Error
    println("Model: ${info.modelName}")
    println("Memory: ${info.approxMemory}")
    println("History: ${info.historyLength} turns")
}
```

---

## File system layout

Onde stores model files in the app's internal `filesDir`. No external storage or
special permissions are required beyond `INTERNET`.

```
<filesDir>/
├── models/
│   └── hub/
│       └── models--bartowski--Qwen2.5-1.5B-Instruct-GGUF/
│           └── ...
└── tmp/
```

You can inspect the cache location at runtime:

```kotlin
println(onde.modelCacheDir.absolutePath)
```

---

## Architecture

```
┌───────────────────────────────────────┐
│  Kotlin app / Android activity        │
│  import com.ondeinference.onde.*      │
└──────────────┬────────────────────────┘
               │ suspend / Flow
               ▼
┌───────────────────────────────────────┐
│  OndeInference.kt  (this SDK)         │
│  Android filesystem setup             │
│  Dispatchers.IO dispatch              │
└──────────────┬────────────────────────┘
               │ UniFFI-generated Kotlin (onde.kt)
               ▼
┌───────────────────────────────────────┐
│  libonde.so  (Rust, per ABI)          │
│  mistral.rs  +  Candle  (CPU)         │
│  HuggingFace Hub download             │
└───────────────────────────────────────┘
```

The Rust engine is compiled for four ABIs and bundled as JNI libs:

| ABI | Architecture |
|-----|-------------|
| `arm64-v8a` | 64-bit ARM (modern Android phones) |
| `armeabi-v7a` | 32-bit ARM (older phones) |
| `x86_64` | Android emulators (Intel/AMD) |
| `x86` | 32-bit emulators |

---

## Building from source

### Prerequisites

```bash
# Rust Android targets
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android

# cargo-ndk — manages NDK toolchain for cross-compilation
cargo install cargo-ndk

# Android NDK (via Android Studio or SDK manager)
export ANDROID_NDK_HOME=$HOME/Library/Android/sdk/ndk/26.1.10909125
```

### Build the native libraries

```bash
# From the repo root
./sdk/kotlin/scripts/build-android.sh
```

This produces `.so` files in `sdk/kotlin/lib/src/main/jniLibs/`.

### Generate Kotlin UniFFI bindings

```bash
./sdk/kotlin/scripts/generate-bindings.sh
```

This generates `onde.kt` in `sdk/kotlin/lib/src/generated/kotlin/`. The generated file
is gitignored — regenerate it whenever the Rust API changes.

### Build the AAR

```bash
cd sdk/kotlin
./gradlew :lib:assembleRelease
# Output: lib/build/outputs/aar/lib-release.aar
```

---

## Publishing to Maven Central

Publishing requires:

| Secret | Description |
|--------|-------------|
| `MAVEN_CENTRAL_USERNAME` | Sonatype Central Portal username |
| `MAVEN_CENTRAL_PASSWORD` | Sonatype Central Portal password |
| `SIGNING_KEY_ID` | PGP key ID (last 8 hex chars) |
| `SIGNING_KEY` | ASCII-armored PGP private key |
| `SIGNING_PASSWORD` | PGP key passphrase |

Tag a release to trigger the workflow:

```bash
# Bump version in Cargo.toml and gradle.properties first
git tag 0.1.4 && git push origin 0.1.4
```

The CI workflow builds all ABIs, generates bindings, packages the AAR, signs it, and
publishes to Maven Central automatically.

---

## License

MIT OR Apache-2.0 — see [LICENSE](../../LICENSE).

Built by [Onde Inference / Splitfire AB](https://ondeinference.com).
