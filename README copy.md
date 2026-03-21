<h1 align="center">
  <br>
  Onde
  <br>
</h1>

<p align="center">
  <strong>On-device inference for cross-platform apps.</strong>
  <br>
  Run LLMs, diffusion models, and speech-to-text locally — no cloud, no latency, no data leaving the device.
</p>

<p align="center">
  <a href="https://crates.io/crates/onde"><img src="https://img.shields.io/crates/v/onde.svg" alt="crates.io"></a>
  <a href="https://docs.rs/onde"><img src="https://docs.rs/onde/badge.svg" alt="docs.rs"></a>
  <a href="#license"><img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg" alt="License"></a>
</p>

---

**Onde** is a Rust crate that gives your app on-device AI in a single dependency. It wraps [mistral.rs](https://github.com/EricLBuehler/mistral.rs) for LLM and image generation, and [whisper.cpp](https://github.com/ggerganov/whisper.cpp) (via [transcribe-rs](https://github.com/cjpais/transcribe-rs)) for speech-to-text — with a unified API that handles model discovery, HuggingFace Hub downloads, cache management, and GPU acceleration across every platform.

## License

MIT License

---

## Copyright

<p align="center">
  2026 <a href="https://ondeinference.com">Onde Inference</a>
</p>
