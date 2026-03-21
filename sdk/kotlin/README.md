# Onde — Kotlin (Android) Library

Kotlin bindings for the [Onde](https://ondeinference.com) on-device inference engine, generated via [UniFFI](https://github.com/mozilla/uniffi-rs).

This library provides Android apps with:

- **LLM chat inference** — load GGUF-quantized models (Qwen 2.5) and run multi-turn conversations entirely on-device.
- **Streaming generation** — token-by-token callbacks for real-time UI updates.
- **Whisper speech-to-text** — transcribe audio files and raw PCM samples using whisper.cpp (feature-gated behind `whisper`).
- **HuggingFace Hub integration** — download, cache, inspect, and manage models from HuggingFace.

All inference runs locally on the device CPU (ARM NEON). No network connection is required after the model is downloaded.

## Prerequisites

| Tool | Version | Notes |
|------|---------|-------|
| Rust | stable (1.80+) | `rustup update stable` |
| Android NDK | r27+ | Via Android Studio SDK Manager |
| Android SDK | compileSdk 36 | API 24+ (minSdk) |
| Cargo | latest | Ships with Rust |

Install the Rust Android cross-compilation targets:

```sh
rustup target add \
    aarch64-linux-android \
    armv7-linux-androideabi \
    x86_64-linux-android \
    i686-linux-android
```

Set the NDK path (or let the build script auto-detect it):

```sh
export ANDROID_NDK_HOME="$HOME/Library/Android/sdk/ndk/27.2.12479018"
# — or —
export ANDROID_HOME="$HOME/Library/Android/sdk"  # NDK auto-detected under ndk/
```

## Building

From the `onde/` crate root:

```sh
# Full release build for all four Android ABIs + Kotlin codegen
./build-kotlin.sh

# Debug build, arm64 only (faster iteration)
./build-kotlin.sh --debug --target aarch64-linux-android

# Regenerate Kotlin source without recompiling Rust
./build-kotlin.sh --generate-only

# Build with the Whisper feature enabled
ONDE_FEATURES=whisper ./build-kotlin.sh
```

The script performs three steps:

1. **Builds `uniffi-bindgen`** — a host binary that reads UniFFI metadata from the compiled library.
2. **Cross-compiles `libonde.so`** — for each Android target (`arm64-v8a`, `armeabi-v7a`, `x86_64`, `x86`).
3. **Generates Kotlin source** — runs `uniffi-bindgen generate --language kotlin` to produce `onde.kt` in the library's source set.

After a successful build, the directory structure looks like:

```
kotlin/
├── onde/
│   └── src/main/
│       ├── jniLibs/
│       │   ├── arm64-v8a/libonde.so
│       │   ├── armeabi-v7a/libonde.so
│       │   ├── x86_64/libonde.so
│       │   └── x86/libonde.so
│       └── kotlin/com/ondeinference/onde/
│           └── onde.kt            ← generated UniFFI bindings
├── build.gradle.kts
├── settings.gradle.kts
└── ...
```

## Integration

### Option A: Composite build (recommended for monorepo)

In your app's `settings.gradle.kts`:

```kotlin
includeBuild("path/to/onde/kotlin") {
    dependencySubstitution {
        substitute(module("com.ondeinference:onde")).using(project(":onde"))
    }
}
```

Then in your app's `build.gradle.kts`:

```kotlin
dependencies {
    implementation("com.ondeinference:onde")
}
```

### Option B: Direct project dependency

In your app's `settings.gradle.kts`:

```kotlin
include(":onde")
project(":onde").projectDir = file("path/to/onde/kotlin/onde")
```

Then in your app's `build.gradle.kts`:

```kotlin
dependencies {
    implementation(project(":onde"))
}
```

### Option C: Publish as AAR

```sh
cd kotlin
./gradlew :onde:assembleRelease
```

The AAR is produced at `onde/build/outputs/aar/onde-release.aar`. Publish it to your local Maven repository or a private artifact server.

## API Usage

### Chat Inference

```kotlin
import com.ondeinference.onde.*
import kotlinx.coroutines.*

// Create an engine instance.
val engine = OndeChatEngine()

// Load the platform-appropriate default model (Qwen 2.5 1.5B on Android).
val loadTimeSecs = engine.loadDefaultModel(
    systemPrompt = "You are a helpful assistant.",
    sampling = null  // use defaults
)
println("Model loaded in ${loadTimeSecs}s")

// Multi-turn chat — history is managed automatically.
val reply = engine.sendMessage("What is Kotlin?")
println(reply.text)            // "Kotlin is a modern programming language..."
println(reply.durationDisplay) // "4.2s"

// One-shot generation (does NOT modify conversation history).
val enhanced = engine.generate(
    messages = listOf(userMessage("Expand: a cat in space")),
    sampling = deterministicSamplingConfig()
)
println(enhanced.text)

// Check engine status.
val info = engine.info()
println("Status: ${info.status}, History: ${info.historyLength} turns")

// Conversation management.
val history: List<ChatMessage> = engine.history()
engine.clearHistory()
engine.setSystemPrompt("You are a pirate.")

// Cleanup — frees model memory.
engine.unloadModel()
```

### Streaming Inference

```kotlin
import com.ondeinference.onde.*

// Implement the callback interface.
class MyStreamHandler : StreamChunkListener {
    private val buffer = StringBuilder()

    override fun onChunk(chunk: StreamChunk): Boolean {
        buffer.append(chunk.delta)
        print(chunk.delta)  // Real-time token output

        // Return true to continue, false to cancel early.
        return !chunk.done
    }
}

// Stream through the free function (UniFFI 0.31 callback_interface pattern).
streamChatMessage(
    engine = engine,
    message = "Tell me a story about a brave robot.",
    listener = MyStreamHandler()
)
```

### Model Configuration

```kotlin
import com.ondeinference.onde.*

// Platform default (Qwen 2.5 1.5B on Android, 3B on desktop).
val defaultConfig = defaultModelConfig()

// Explicit model configs.
val small = qwen251_5bConfig()   // ~941 MB, ideal for mobile
val medium = qwen253bConfig()    // ~1.93 GB, for devices with more RAM

// Custom config.
val custom = GgufModelConfig(
    modelId = "bartowski/Qwen2.5-1.5B-Instruct-GGUF",
    files = listOf("Qwen2.5-1.5B-Instruct-Q4_K_M.gguf"),
    tokModelId = "Qwen/Qwen2.5-1.5B-Instruct",  // required on Android
    displayName = "Qwen 2.5 1.5B",
    approxMemory = "~941 MB (GGUF Q4_K_M)"
)

engine.loadGgufModel(
    config = custom,
    systemPrompt = "You are helpful.",
    sampling = mobileSamplingConfig()  // conservative defaults for mobile
)
```

### Sampling Configuration

```kotlin
import com.ondeinference.onde.*

// Creative chat defaults (temperature=0.7, top_p=0.95, max_tokens=512).
val creative = defaultSamplingConfig()

// Deterministic / greedy (temperature=0.0).
val greedy = deterministicSamplingConfig()

// Mobile-optimised (max_tokens=128 for faster response on CPU).
val mobile = mobileSamplingConfig()

// Fully custom.
val custom = SamplingConfig(
    temperature = 0.9,
    topP = 0.95,
    topK = 50u,
    minP = null,
    maxTokens = 256u,
    frequencyPenalty = 0.1f,
    presencePenalty = 0.1f
)

engine.setSampling(custom)
```

### Whisper Speech-to-Text (requires `whisper` feature)

```kotlin
import com.ondeinference.onde.*

// Create and load a Whisper engine.
val whisper = OndeWhisperEngine()

// Download or find a cached model.
val modelPath = findOrDownloadWhisperModelFfi(
    modelDir = "/data/user/0/com.myapp/files/whisper",
    modelName = null,  // platform default (ggml-base.bin on Android)
    listener = object : WhisperProgressListener {
        override fun onProgress(progress: WhisperModelDownloadProgress) {
            println("${progress.downloadedDisplay} / ${progress.totalDisplay}")
        }
    },
    appDataDir = context.filesDir.absolutePath
)

whisper.loadModel(modelPath)

// Transcribe from a file.
val result = whisper.transcribeFile(
    path = "/path/to/audio.wav",
    language = "en"  // or null for auto-detect
)
println(result.text)
result.segments.forEach { seg ->
    println("[${seg.startSecs}s - ${seg.endSecs}s] ${seg.text}")
}

// Transcribe from raw PCM samples.
val samples: List<Float> = readWavSamplesFfi("/path/to/audio.wav")
val result2 = whisper.transcribeSamples(samples, language = null)
println(result2.text)
```

### Helper Functions

```kotlin
import com.ondeinference.onde.*

// Message constructors.
val sys = systemMessage("You are helpful.")
val usr = userMessage("Hello!")
val ast = assistantMessage("Hi there!")

// Platform default Whisper model name.
val whisperModel = defaultModelForPlatform() // "ggml-base.bin" on Android

// All available Whisper model candidates.
val candidates = whisperModelCandidates()
// ["ggml-large-v3-turbo.bin", "ggml-large-v3.bin", ..., "ggml-tiny.en.bin"]
```

## Error Handling

All fallible operations throw typed exceptions generated from the Rust error enums:

```kotlin
try {
    engine.sendMessage("Hello")
} catch (e: InferenceError.NoModelLoaded) {
    // No model loaded — call loadGgufModel() or loadDefaultModel() first.
} catch (e: InferenceError.Inference) {
    // Inference failed: ${e.reason}
} catch (e: InferenceError.ModelBuild) {
    // Failed to build model: ${e.reason}
} catch (e: InferenceError.AlreadyLoaded) {
    // Model is already loaded: ${e.modelName}
} catch (e: InferenceError.Cancelled) {
    // Model loading was cancelled.
} catch (e: InferenceError.Other) {
    // Generic error: ${e.reason}
}
```

Whisper operations throw `WhisperError` variants:

```kotlin
try {
    whisper.transcribeFile(path, language)
} catch (e: WhisperError.ModelNotLoaded) { ... }
  catch (e: WhisperError.ModelLoadFailed) { ... }
  catch (e: WhisperError.TranscriptionFailed) { ... }
  catch (e: WhisperError.DownloadFailed) { ... }
  catch (e: WhisperError.Io) { ... }
```

## Threading Model

- **`OndeChatEngine`** methods are `suspend` functions — call them from a coroutine scope (e.g. `viewModelScope`, `lifecycleScope`).
- **`OndeWhisperEngine`** methods are synchronous (blocking). Call them from `Dispatchers.IO` or a background thread:

  ```kotlin
  withContext(Dispatchers.IO) {
      whisper.loadModel(path)
      val result = whisper.transcribeFile(audioPath, "en")
  }
  ```

- The underlying Rust engine is thread-safe (`Send + Sync`). Multiple coroutines can safely share a single `OndeChatEngine` instance.

## Project Structure

```
onde/
├── build-kotlin.sh            # Build script (cross-compile + bindgen)
├── uniffi.toml                # UniFFI config (Kotlin package name)
├── uniffi-bindgen/            # Host binary for generating bindings
│   ├── Cargo.toml
│   └── uniffi-bindgen.rs
├── kotlin/                    # Android library project (Gradle)
│   ├── build.gradle.kts       # Root Gradle build
│   ├── settings.gradle.kts
│   ├── gradle.properties
│   ├── README.md              # ← You are here
│   └── onde/                  # Library module
│       ├── build.gradle.kts
│       ├── proguard-rules.pro
│       ├── consumer-rules.pro
│       └── src/main/
│           ├── AndroidManifest.xml
│           ├── jniLibs/       # Native .so files (build artifact)
│           └── kotlin/com/ondeinference/onde/
│               └── onde.kt    # Generated UniFFI Kotlin bindings
├── src/                       # Rust source (the actual engine)
│   ├── lib.rs
│   ├── hf_cache.rs
│   ├── whisper.rs
│   └── inference/
│       ├── engine.rs
│       ├── ffi.rs             # UniFFI-exported wrappers
│       ├── models.rs
│       ├── token.rs
│       └── types.rs           # Records, Enums, Errors
└── Cargo.toml
```

## Troubleshooting

### `ANDROID_NDK_HOME not found`

Set the environment variable to your NDK installation:

```sh
export ANDROID_NDK_HOME="$HOME/Library/Android/sdk/ndk/27.2.12479018"
```

Or install the NDK via Android Studio → SDK Manager → SDK Tools → NDK (Side by side).

### `linker not found` during cross-compilation

Ensure the NDK toolchain binaries are present:

```sh
ls $ANDROID_NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/aarch64-linux-android24-clang
```

If missing, your NDK installation may be incomplete. Reinstall it.

### `UnsatisfiedLinkError: libonde.so` at runtime

The `.so` file is missing from the APK's `jniLibs` for the device's ABI. Verify:

1. `build-kotlin.sh` completed successfully for the target ABI.
2. The `.so` exists at `onde/src/main/jniLibs/<abi>/libonde.so`.
3. Your app's `build.gradle.kts` doesn't filter out the ABI via `ndk { abiFilters ... }`.

### `uniffi-bindgen` version mismatch

The `uniffi` version in `uniffi-bindgen/Cargo.toml` **must** exactly match the version in the main `onde` crate's `Cargo.toml` (`=0.31.0`). A mismatch causes a runtime panic with a checksum error.

## License

MIT OR Apache-2.0 — same as the `onde` crate.
