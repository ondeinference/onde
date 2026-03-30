---
title: "Onde"
description: "On-device chat inference for every platform. Run LLMs locally with no cloud, no latency, and no data leaving the device."
---

# Onde

**On-device chat inference for every platform.**

Run LLM chat locally — no cloud, no latency, no data leaving the device. Onde wraps [mistral.rs](https://github.com/EricLBuehler/mistral.rs) with a unified API that handles model discovery, HuggingFace Hub downloads, cache management, and GPU acceleration across every platform.

## Platforms

| Platform | GPU Backend | Default Model |
| -------- | ----------- | ------------- |
| macOS | Metal | Qwen 2.5 3B |
| iOS | Metal | Qwen 2.5 1.5B |
| tvOS | Metal | Qwen 2.5 1.5B |
| Android | CPU (candle) | Qwen 2.5 1.5B |
| Windows | CPU / CUDA | Qwen 2.5 3B |
| Linux | CPU / CUDA | Qwen 2.5 3B |

## SDKs

| Language | Package | Install |
| -------- | ------- | ------- |
| Rust | [`onde`](https://crates.io/crates/onde) | `onde = "0.1"` in Cargo.toml |
| Swift | [`onde-swift`](https://github.com/ondeinference/onde-swift) | Swift Package Manager |
| Kotlin | [`onde-inference`](https://central.sonatype.com/) | Gradle |
| Python | [`onde-inference`](https://pypi.org/project/onde-inference) | `pip install onde-inference` |
| Ruby | [`onde-inference`](https://rubygems.org/gems/onde-inference) | `gem install onde-inference` |

## Quick Start

### Swift (iOS / tvOS / macOS)

```swift
import Onde

let engine = OndeChatEngine()
try await engine.loadDefaultModel(systemPrompt: "You are a helpful assistant.", sampling: nil)

let result = try await engine.sendMessage(message: "Hello!")
print(result.text)
```

### Rust

```rust
use onde::inference::{ChatEngine, GgufModelConfig};

let engine = ChatEngine::new();
engine.load_gguf_model(
    GgufModelConfig::platform_default(),
    Some("You are a helpful assistant.".into()),
    None,
).await?;

let reply = engine.send_message("Hello!").await?;
println!("{}", reply.text);
```

### Python

```python
import asyncio
from onde_inference import OndeChatEngine

async def main():
    engine = OndeChatEngine()
    await engine.load_default_model(system_prompt="You are a helpful assistant.", sampling=None)
    result = await engine.send_message("Hello!")
    print(result.text)

asyncio.run(main())
```

## Guides

- [Developer Guide](dev) — build from source, architecture overview, platform support
- [Swift Package](swift-package) — XCFramework distribution, Swift API reference
- [Ruby Gem](ruby-gem) — Magnus-based native extension, Rails integration
- [Distribution](distribution) — publishing to crates.io, Swift Package Index, PyPI, RubyGems, Maven Central

## Design Principles

**Zero framework coupling.** Onde knows nothing about any app framework. Progress updates use plain callbacks — wire them to whatever event system you use.

**Platform-aware by default.** GPU backend, model size, cache directory — Onde picks the right defaults so you don't configure per-platform.

**One dependency.** Add `onde` and get on-device chat inference, HuggingFace cache management, and HF token resolution in a single crate.

## License

Licensed under either of [MIT](https://opensource.org/licenses/MIT) or [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0) at your option.