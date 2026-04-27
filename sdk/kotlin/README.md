<p align="center">
  <img src="https://raw.githubusercontent.com/ondeinference/onde/main/assets/onde-inference-logo.svg" alt="Onde Inference" width="96">
</p>

<h1 align="center">Onde Inference Kotlin SDK</h1>

<p align="center">
  <strong>Run LLMs on-device from Kotlin. No cloud, no API key, no user data leaves the device.</strong>
</p>

<p align="center">
  <a href="https://central.sonatype.com/artifact/com.ondeinference/onde-inference"><img src="https://img.shields.io/maven-central/v/com.ondeinference/onde-inference?style=flat-square&color=235843&labelColor=17211D&label=maven" alt="Maven Central"></a>
  <a href="https://ondeinference.com"><img src="https://img.shields.io/badge/ondeinference.com-235843?style=flat-square&labelColor=17211D" alt="Website"></a>
  <a href="https://github.com/ondeinference/onde/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-235843?style=flat-square&labelColor=17211D" alt="License"></a>
</p>

<p align="center">
  <a href="https://github.com/ondeinference/onde">Rust SDK</a> · <a href="https://github.com/ondeinference/onde-swift">Swift SDK</a> · <a href="https://pub.dev/packages/onde_inference">Flutter SDK</a> · <a href="https://www.npmjs.com/package/@ondeinference/react-native">React Native SDK</a> · <a href="https://ondeinference.com">Website</a>
</p>

---

## What is Onde?

Onde is an on-device LLM inference SDK for Kotlin apps. It wraps [mistral.rs](https://github.com/EricLBuehler/mistral.rs) in a Kotlin-friendly API with model downloads from Hugging Face, local cache management, and native inference.

- **Runs locally**: no cloud, no API key, no user data leaves the device
- **Kotlin-friendly**: suspend functions and `Flow<StreamChunk>` for streaming
- **Rust core**: built on [uniffi-rs](https://github.com/mozilla/uniffi-rs)
- **One dependency**: add it from Maven Central and you're done

It's a Kotlin Multiplatform library. Right now that means Android and JVM, with the same Rust core underneath both.

---

## Installation

Add to your app's `build.gradle.kts`:

```kotlin
dependencies {
    implementation("com.ondeinference:onde-inference:0.1.3")
}
```

Add `INTERNET` permission to `AndroidManifest.xml` (the initial model download needs it):

```xml
<uses-permission android:name="android.permission.INTERNET" />
```

**Minimum requirements**
- Android 8.0 (API 26) or higher
- About 1.1 GB of free storage for the default model (Qwen 2.5 1.5B Q4_K_M)

---

## Quick start on Android

```kotlin
import com.ondeinference.onde.OndeInference
import kotlinx.coroutines.launch

class MainActivity : AppCompatActivity() {

    private val onde by lazy { OndeInference(applicationContext) }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        lifecycleScope.launch {
            // 1. Load the default model (Qwen 2.5 1.5B on Android, ~941 MB)
            //    Downloads from Hugging Face on first run.
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

To get tokens as they arrive, use `Flow`:

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

Onde tracks conversation history for you:

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
    // Qwen 2.5 1.5B, the default on Android
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
    // Default chat settings: temperature 0.7, top_p 0.95, max 512 tokens
    onde.loadDefaultModel(sampling = OndeSampling.default())

    // Deterministic output, good for factual answers and code
    onde.loadDefaultModel(sampling = OndeSampling.deterministic())

    // Shorter responses, better for lower-end devices
    onde.loadDefaultModel(sampling = OndeSampling.mobile())
}
```

---

## One-shot generation (without changing chat history)

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

Onde stores model files in the app's internal `filesDir`. No external storage permissions needed. `INTERNET` is only for the initial download.

```
<filesDir>/
├── models/
│   └── hub/
│       └── models--bartowski--Qwen2.5-1.5B-Instruct-GGUF/
│           └── ...
└── tmp/
```

You can check the cache location at runtime:

```kotlin
println(onde.modelCacheDir.absolutePath)
```

---

## How it fits together

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

On Android, the Rust engine is compiled for four ABIs and bundled as JNI libs:

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

# cargo-ndk manages NDK toolchain for cross-compilation
cargo install cargo-ndk

# Android NDK (via Android Studio or SDK manager)
export ANDROID_NDK_HOME=$HOME/Library/Android/sdk/ndk/26.1.10909125
```

### Build the native libraries

```bash
# From the repo root
./sdk/kotlin/scripts/build-android.sh
```

This produces `.so` files in `sdk/kotlin/lib/src/androidMain/jniLibs/`.

### Generate Kotlin UniFFI bindings

```bash
./sdk/kotlin/scripts/generate-bindings.sh
```

This generates `onde.kt` in `sdk/kotlin/lib/src/generated/kotlin/`. The generated file is gitignored, so regenerate it whenever the Rust API changes.

### Build the Android artifact

```bash
cd sdk/kotlin
./gradlew :lib:assembleRelease
# Output: lib/build/outputs/aar/lib-release.aar
```

---

## Publishing to Maven Central

CI handles publishing. It needs these secrets:

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

The CI workflow builds the native libraries, generates bindings, packages the Android artifact, signs it, and publishes to Maven Central.

---

## License

MIT OR Apache-2.0. See [LICENSE](../../LICENSE).

Built by [Onde Inference / Splitfire AB](https://ondeinference.com).