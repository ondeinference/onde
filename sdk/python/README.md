<p align="center">
  <h1 align="center">onde-inference</h1>
  <p align="center"><strong>On-device AI inference for Python</strong></p>
</p>

<p align="center">
  <a href="https://pypi.org/project/onde-inference/"><img src="https://img.shields.io/pypi/v/onde-inference.svg" alt="PyPI version"></a>
  <a href="https://pypi.org/project/onde-inference/"><img src="https://img.shields.io/pypi/pyversions/onde-inference.svg" alt="Python 3.9+"></a>
  <a href="#license"><img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg" alt="License"></a>
</p>

---

Run LLMs and speech-to-text locally — no cloud, no latency, no data leaving the device. Powered by Rust via [mistral.rs](https://github.com/EricLBuehler/mistral.rs).

`onde-inference` is a Python binding for the [Onde](https://github.com/ondeinference/onde) Rust crate. It wraps the Rust inference engine via [UniFFI](https://mozilla.github.io/uniffi-rs/) bindings, giving you native-speed on-device inference with a simple, async Python API.

## Installation

```bash
pip install onde-inference
```

## Quick Start

### Basic Chat

```python
import asyncio
from onde_inference import OndeChatEngine, default_model_config


async def main():
    # Create the engine (starts with no model loaded).
    engine = OndeChatEngine()

    # Load the platform-appropriate default model.
    # macOS → Qwen 2.5 3B (~1.93 GB), Linux/Windows → Qwen 2.5 3B (~1.93 GB)
    elapsed = await engine.load_default_model(
        system_prompt="You are a helpful assistant.",
        sampling=None,
    )
    print(f"Model loaded in {elapsed:.1f}s")

    # Send a message — history is managed automatically.
    result = await engine.send_message("What is Rust?")
    print(result.text)
    print(f"Generated in {result.duration_display}")

    # Multi-turn conversation.
    follow_up = await engine.send_message("How does its ownership model work?")
    print(follow_up.text)

    # Cleanup.
    await engine.unload_model()


asyncio.run(main())
```

### Model Configuration Helpers

Choose a specific model instead of the platform default:

```python
from onde_inference import default_model_config, qwen25_1_5b_config, qwen25_3b_config

# Platform-appropriate default (3B on desktop, 1.5B on mobile).
config = default_model_config()

# Explicitly pick a model.
small = qwen25_1_5b_config()   # Qwen 2.5 1.5B — ~941 MB
medium = qwen25_3b_config()    # Qwen 2.5 3B  — ~1.93 GB

# Load a specific model config.
elapsed = await engine.load_gguf_model(
    config=small,
    system_prompt="You are a helpful assistant.",
    sampling=None,
)
```

### Sampling Configuration

Control how the model generates text:

```python
from onde_inference import (
    default_sampling_config,
    deterministic_sampling_config,
    mobile_sampling_config,
)

# Creative chat (temperature=0.7, top_p=0.95, max_tokens=512).
creative = default_sampling_config()

# Greedy decoding (temperature=0.0) — deterministic output.
greedy = deterministic_sampling_config()

# Conservative settings for constrained devices (max_tokens=128).
mobile = mobile_sampling_config()

# Pass to load or update at any time.
await engine.load_default_model(system_prompt="You are helpful.", sampling=greedy)
await engine.set_sampling(creative)
```

### Streaming

Stream tokens as they are generated using a callback class that implements `StreamChunkListener`:

```python
import asyncio
from onde_inference import (
    OndeChatEngine,
    StreamChunkListener,
    StreamChunk,
    stream_chat_message,
)


class MyStreamHandler(StreamChunkListener):
    def on_chunk(self, chunk: StreamChunk) -> bool:
        """Called for each token. Return True to continue, False to cancel."""
        if chunk.delta:
            print(chunk.delta, end="", flush=True)
        if chunk.done:
            print()  # newline at the end
        return True


async def main():
    engine = OndeChatEngine()
    await engine.load_default_model(
        system_prompt="You are a helpful assistant.",
        sampling=None,
    )

    # stream_chat_message is a free function (UniFFI requirement).
    await stream_chat_message(
        engine=engine,
        message="Tell me a short story about a robot learning to paint.",
        listener=MyStreamHandler(),
    )

    await engine.unload_model()


asyncio.run(main())
```

### Message Helpers

Build message lists for one-shot generation without modifying conversation history:

```python
from onde_inference import system_message, user_message, assistant_message

messages = [
    system_message("You expand short prompts into detailed descriptions."),
    user_message("a cat in space"),
]

result = await engine.generate(messages=messages, sampling=None)
print(result.text)
```

### Engine Introspection

```python
# Check engine state.
info = await engine.info()
print(info.status)        # "ready", "unloaded", "generating", etc.
print(info.model_name)    # "Qwen 2.5 3B" or None
print(info.history_length) # number of conversation turns

# Access or clear history.
history = await engine.history()
cleared = await engine.clear_history()

# Check if a model is loaded.
if await engine.is_loaded():
    print("Ready to chat!")
```

## Platform Support

| Platform | Accelerator | Notes |
|----------|-------------|-------|
| **macOS** | Metal GPU | Apple Silicon and Intel Macs |
| **Linux** | CPU / CUDA | NVIDIA GPU acceleration with CUDA builds |
| **Windows** | CPU / CUDA | NVIDIA GPU acceleration with CUDA builds |

## Supported Models

| Model | Size | Best For |
|-------|------|----------|
| **Qwen 2.5 1.5B** (GGUF Q4_K_M) | ~941 MB | Lightweight / memory-constrained environments |
| **Qwen 2.5 3B** (GGUF Q4_K_M) | ~1.93 GB | Balanced quality and performance on desktop |

Models are automatically downloaded from HuggingFace Hub on first use and cached locally.

## How It Works

`onde-inference` is a thin Python layer over the **Onde** Rust crate. [UniFFI](https://mozilla.github.io/uniffi-rs/) generates Python bindings from the Rust source, so every call goes directly into native compiled code — there is no Python inference loop. Async methods use Python's `asyncio` (UniFFI generates async Python wrappers for Rust async functions backed by Tokio).

## Links

- **Source**: [github.com/ondeinference/onde](https://github.com/ondeinference/onde)
- **Rust crate**: [crates.io/crates/onde](https://crates.io/crates/onde)
- **Documentation**: [docs.rs/onde](https://docs.rs/onde)
- **Inference engine**: [mistral.rs](https://github.com/EricLBuehler/mistral.rs)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](https://github.com/ondeinference/onde/blob/main/LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](https://github.com/ondeinference/onde/blob/main/LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Copyright

Copyright 2026 [Onde Inference](https://ondeinference.com)