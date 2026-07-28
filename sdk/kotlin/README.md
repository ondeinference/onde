<p align="center">
  <img src="https://raw.githubusercontent.com/ondeinference/onde/main/assets/onde-inference-logo.svg" alt="Onde Inference" width="96">
</p>

<h1 align="center">Onde Inference Kotlin SDK</h1>

<p align="center">
  <strong>Run LLMs on-device from Kotlin with <a href="https://ondeinference.com/">Onde Inference</a>. No cloud, no API key, and no user data leaving the device.</strong>
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

Onde is an on-device LLM inference SDK for Kotlin apps. It wraps the shared Rust core built on top of [mistral.rs](https://github.com/EricLBuehler/mistral.rs) in a Kotlin-friendly API with model downloads from Hugging Face, local cache management, and native inference under the hood.

- **Runs locally**: no cloud, no API key, no user data leaving the device
- **Kotlin-friendly**: `suspend` functions and `Flow<StreamChunk>` for streaming
- **Shared Rust core**: the same engine powers the Rust, Swift, Flutter, and React Native SDKs
- **Kotlin Multiplatform**: today this package targets Android and JVM
- **Published on Maven Central**: add one Gradle dependency and start integrating

If you want to experiment with models before wiring them into an app, use [Onde CLI](https://github.com/ondeinference/onde-cli).

---

## Installation

Add the SDK to your app's `build.gradle.kts`:

```kotlin
dependencies {
    implementation("com.ondeinference:onde-inference:1.0.0")
}
```

Add `INTERNET` permission to `AndroidManifest.xml` for the initial model download:

```xml
<uses-permission android:name="android.permission.INTERNET" />
```

### Minimum requirements

- **Android**: API 26+ (Android 8.0+)
- **Storage**: about 1.1 GB free for the default Android model
- **JVM**: a supported desktop target with the bundled native Onde library

---

## Quick start on Android

```kotlin
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.lifecycle.lifecycleScope
import com.ondeinference.onde.OndeInference
import kotlinx.coroutines.launch

class MainActivity : ComponentActivity() {
    private val onde by lazy { OndeInference(applicationContext) }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        lifecycleScope.launch {
            val elapsed = onde.loadDefaultModel(
                systemPrompt = "You are a helpful, concise assistant."
            )
            println("Model loaded in ${elapsed}s")

            val result = onde.chat("What is the capital of Sweden?")
            println(result.text)
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        onde.close()
    }
}
```

On Android, the default model is **Qwen 2.5 1.5B Instruct (GGUF Q4_K_M, ~941 MB)**.

---

## Quick start on JVM

```kotlin
import com.ondeinference.onde.OndeInference
import kotlinx.coroutines.runBlocking

fun main() = runBlocking {
    val onde = OndeInference()
    try {
        onde.loadDefaultModel(systemPrompt = "You are a helpful assistant.")
        val reply = onde.chat("Give me three Kotlin tips.")
        println(reply.text)
    } finally {
        onde.close()
    }
}
```

On desktop JVM targets, the default model is **Qwen 2.5 3B Instruct (GGUF Q4_K_M, ~1.93 GB)**.

---

## Streaming

If you want tokens as they arrive, collect a `Flow<StreamChunk>`:

```kotlin
lifecycleScope.launch {
    onde.stream("Write a haiku about the ocean.").collect { chunk ->
        textView.append(chunk.delta)
        if (chunk.done) println("\n[done]")
    }
}
```

---

## Multi-turn conversation

Onde keeps conversation history for you:

```kotlin
lifecycleScope.launch {
    onde.loadDefaultModel(systemPrompt = "You are a Rust tutor.")

    val r1 = onde.chat("What is ownership?")
    println(r1.text)

    val r2 = onde.chat("Can you give me a code example?")
    println(r2.text)

    onde.clearHistory()
}
```

---

## Models and sampling

The SDK exposes convenience helpers for supported models and sampling presets:

```kotlin
import com.ondeinference.onde.OndeModels
import com.ondeinference.onde.OndeSampling

lifecycleScope.launch {
    onde.loadModel(
        config = OndeModels.qwen25_1_5b(),
        systemPrompt = "You are a coding assistant.",
        sampling = OndeSampling.deterministic(),
    )
}
```

Available helpers include:

- `OndeModels.default()`
- `OndeModels.qwen25_1_5b()`
- `OndeModels.qwen25_3b()`
- `OndeSampling.default()`
- `OndeSampling.deterministic()`
- `OndeSampling.mobile()`

---

## One-shot generation

Use `generate()` when you want a response without mutating chat history:

```kotlin
import com.ondeinference.onde.OndeMessage
import com.ondeinference.onde.OndeSampling

lifecycleScope.launch {
    val result = onde.generate(
        messages = listOf(
            OndeMessage.system("You are a summariser."),
            OndeMessage.user("Summarise this article in five bullet points."),
        ),
        sampling = OndeSampling.deterministic(),
    )
    println(result.text)
}
```

---

## Engine status and cache location

```kotlin
lifecycleScope.launch {
    val info = onde.info()
    println("Status: ${info.status}")
    println("Model: ${info.modelName}")
    println("History: ${info.historyLength} turns")
}

println(onde.modelCacheDir.absolutePath)
```

Onde stores model files inside the configured data directory:

```text
<dataDir>/
├── models/
│   └── hub/
│       └── models--bartowski--Qwen2.5-1.5B-Instruct-GGUF/
│           └── ...
└── tmp/
```

On Android, the default `dataDir` is `context.filesDir`.

---

## Building from source

The Kotlin SDK lives in `sdk/kotlin/` and is built from the shared Rust crate in the repository root.

### Prerequisites

```bash
# Rust Android targets
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android

# cargo-ndk for Android cross-compilation
cargo install cargo-ndk

# Android NDK
export ANDROID_NDK_HOME=$HOME/Library/Android/sdk/ndk/26.1.10909125
```

### Build Android native libraries

```bash
./sdk/kotlin/scripts/build-android.sh
```

This produces `.so` files in `sdk/kotlin/lib/src/androidMain/jniLibs/`.

### Build JVM native library

```bash
./sdk/kotlin/scripts/build-jvm.sh
```

This bundles the desktop native library into `sdk/kotlin/lib/src/jvmMain/resources/native/`.

### Generate Kotlin UniFFI bindings

```bash
./sdk/kotlin/scripts/generate-bindings.sh
```

This generates the Kotlin bindings in `sdk/kotlin/lib/src/generated/kotlin/`.

> The generated bindings and compiled native libraries are intentionally gitignored and should be regenerated from source.

### Build the Gradle modules

```bash
cd sdk/kotlin
./gradlew :lib:assembleRelease
./gradlew :example:assembleDebug
```

---

## Publishing to Maven Central

Publishing is handled by CI. The published coordinates are controlled by `sdk/kotlin/gradle.properties`:

- `GROUP=com.ondeinference`
- `POM_ARTIFACT_ID=onde-inference`
- `VERSION_NAME=1.0.0`

Tag a release after bumping the Rust and Kotlin versions together.

---

## Project layout

```text
sdk/kotlin/
├── build.gradle.kts
├── settings.gradle.kts
├── gradle.properties
├── README.md
├── example/                 # Android demo app
├── lib/                     # Kotlin Multiplatform SDK module
│   └── src/
│       ├── androidMain/
│       ├── jvmMain/
│       ├── shared/
│       └── generated/
└── scripts/
    ├── build-android.sh
    ├── build-jvm.sh
    └── generate-bindings.sh
```

---

## Troubleshooting

**Download never finishes** — check your internet connection. Hugging Face Hub can rate-limit anonymous downloads.

**App crashes immediately on Android** — make sure the device is running API 26 or newer. The Android platform setup uses `Os.setenv`.

**Inference is very slow** — Android emulators are slow for CPU inference. A physical ARM device is much faster.

**Out of memory while loading** — the default Android model needs roughly 1–1.5 GB of free RAM.

**Native library not found on desktop** — rebuild the JVM native library with `./sdk/kotlin/scripts/build-jvm.sh` so the expected resource bundle is present.

---

## License

MIT OR Apache-2.0. See [LICENSE](../../LICENSE).

## Copyright

© 2026 [Splitfire AB](https://5mb.app) ([Onde Inference](https://ondeinference.com)).
