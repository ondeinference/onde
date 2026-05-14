# Onde Chat

A simple Android chat app built with the [Onde Inference](https://ondeinference.com) SDK. It runs Qwen 2.5 1.5B entirely on your phone — no server, no API key, nothing leaves the device.

---

## What it does

- Downloads the model on first launch (~941 MB, cached after that)
- Streams replies token by token as the model generates them
- Keeps a full conversation history across turns
- Works offline once the model is cached

---

## Getting started

Open `onde/sdk/kotlin/example/` as the project root in Android Studio (not the repo root — the `settings.gradle.kts` lives here). Gradle will pull everything from Maven Central on first sync.

You'll need a device or emulator running **Android 8.0 (API 26) or higher**. An x86_64 emulator works fine, though a real ARM phone will feel noticeably faster.

Hit Run. The first launch will download the model over your internet connection — after that the app is fully offline.

---

## Project layout

```
example/
├── settings.gradle.kts          standalone project root
├── build.gradle.kts             deps and build config
└── src/main/
    ├── AndroidManifest.xml      INTERNET permission lives here
    └── kotlin/com/ondeinference/example/
        ├── MainActivity.kt      one activity, sets up the theme and hands off to Compose
        ├── ChatViewModel.kt     model loading, streaming, history — all the logic
        └── ui/
            ├── ChatScreen.kt    everything you see on screen
            └── theme/Theme.kt  Material3 colours, dynamic on Android 12+
```

---

## How the streaming works

When you send a message, the app drops an empty assistant bubble into the list immediately, then fills it in character by character as tokens arrive from the Rust engine. The ViewModel holds a `StringBuilder` and swaps the tail of the message list on every chunk — so Compose only recomposes the one bubble that's changing, not the whole conversation.

```kotlin
onde.stream(text).collect { chunk ->
    buffer.append(chunk.delta)
    // update only the last bubble in the list
}
```

---

## Swapping the model

The app loads the platform default (Qwen 2.5 1.5B) via `loadDefaultModel()`. To use a different model, call `loadModel()` with a config instead:

```kotlin
onde.loadModel(
    config = OndeModels.qwen25_3b(), // ~1.93 GB — not great on phones
    systemPrompt = "You are a creative writing assistant.",
    sampling = OndeSampling.deterministic(),
)
```

Available sampling presets:

| Preset | Temp | Max tokens | Good for |
|---|---|---|---|
| `OndeSampling.mobile()` | 0.7 | 128 | everyday chat |
| `OndeSampling.default()` | 0.7 | 512 | longer responses |
| `OndeSampling.deterministic()` | 0.0 | 512 | coding, facts |

---

## Developing against a local SDK build

By default the app pulls `onde-inference` from Maven Central. If you want to test changes to the SDK itself, open the parent directory (`onde/sdk/kotlin/`) in Android Studio instead and swap the dependency in `build.gradle.kts`:

```kotlin
// instead of:
implementation("com.ondeinference:onde-inference:0.1.3")

// use:
implementation(project(":lib"))
```

---

## Troubleshooting

**Download never finishes** — check your internet connection. HuggingFace Hub occasionally rate-limits anonymous downloads; setting an `HF_TOKEN` env variable before building can help.

**App crashes immediately** — make sure you're on API 26+. The SDK uses `Os.setenv` to configure the HuggingFace cache paths, which requires Android 8.0.

**Inference is very slow** — CPU inference on an emulator is slow by design. Try a physical ARM device for a much better experience.

**Out of memory on load** — the 1.5B model needs roughly 1–1.5 GB of free RAM. Close other apps and try again, or restart the device.