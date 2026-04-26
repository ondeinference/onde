<p align="center">
  <img src="./assets/onde-inference-logo.svg" alt="Onde Inference" width="96">
</p>

<h1 align="center">Onde Inference</h1>

<p align="center">
  <strong>On-device LLM inference — optimized for <a href="https://en.wikipedia.org/wiki/Apple_silicon">Apple silicon</a>.</strong>
</p>

<p align="center">
  <a href="https://crates.io/crates/onde"><img src="https://img.shields.io/crates/v/onde?style=flat-square&color=235843&labelColor=17211D&label=crates.io" alt="crates.io"></a>
  <a href="https://swiftpackageindex.com/ondeinference/onde-swift"><img src="https://img.shields.io/badge/Swift%20Package%20Index-onde--swift-235843?style=flat-square&labelColor=17211D" alt="Swift Package Index"></a>
  <a href="https://pub.dev/packages/onde_inference"><img src="https://img.shields.io/pub/v/onde_inference?style=flat-square&color=235843&labelColor=17211D&label=pub.dev" alt="pub.dev"></a>
  <a href="https://www.npmjs.com/package/@ondeinference/react-native"><img src="https://img.shields.io/npm/v/@ondeinference/react-native?style=flat-square&color=235843&labelColor=17211D&label=npm" alt="npm"></a>
  <a href="https://ondeinference.com"><img src="https://img.shields.io/badge/ondeinference.com-235843?style=flat-square&labelColor=17211D" alt="Website"></a>
  <a href="https://apps.apple.com/se/developer/splitfire-ab/id1831430993"><img src="https://img.shields.io/badge/App%20Store-live-235843?style=flat-square&labelColor=17211D" alt="App Store"></a>
</p>

<p align="center">
  <a href="https://github.com/ondeinference/onde-swift">Swift SDK</a> · <a href="https://pub.dev/packages/onde_inference">Flutter SDK</a> · <a href="https://www.npmjs.com/package/@ondeinference/react-native">React Native SDK</a> · <a href="https://ondeinference.com">Website</a>
</p>

---

## In production

Onde powers live App Store apps with fully on-device chat — no server, no latency, no data leaving the device.

<p align="center">
  <a href="https://apps.apple.com/se/developer/splitfire-ab/id1831430993" target="_blank">
    <img src="https://developer.apple.com/assets/elements/badges/download-on-the-app-store.svg" alt="Download on the App Store" height="44">
  </a>
</p>

---

## License

Onde is dual-licensed under **MIT** and **Apache 2.0**. You may use it under either license at your option.

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)

### Dependency attribution

| Dependency | License | Author |
|---|---|---|
| [mistral.rs](https://github.com/EricLBuehler/mistral.rs) | MIT | Eric Buehler |
| [UniFFI](https://github.com/mozilla/uniffi-rs) | MPL-2.0 | Mozilla |
| [tokio](https://github.com/tokio-rs/tokio) | MIT | Tokio contributors |

### Model licenses

Models downloaded by Onde have their own licenses independent of this crate. By using Onde, you are also subject to the license of the model you load:

| Model | Size | License | Commercial use |
|---|---|---|---|
| Qwen 2.5 1.5B Instruct (GGUF Q4_K_M) | ~941 MB | [Qwen Community License](https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct/blob/main/LICENSE) | ✅ with conditions¹ |
| Qwen 2.5 3B Instruct (GGUF Q4_K_M) | ~1.93 GB | [Qwen Community License](https://huggingface.co/Qwen/Qwen2.5-3B-Instruct/blob/main/LICENSE) | ✅ with conditions¹ |
| Qwen 2.5 Coder 7B Instruct (GGUF Q4_K_M) | ~4.4 GB | [Qwen Community License](https://huggingface.co/Qwen/Qwen2.5-Coder-7B-Instruct/blob/main/LICENSE) | ✅ with conditions¹ |
| Qwen 3 1.7B (GGUF Q4_K_M) | ~1.3 GB | [Apache 2.0](https://huggingface.co/Qwen/Qwen3-1.7B/blob/main/LICENSE) | ✅ |
| Qwen 3 4B (GGUF Q4_K_M) | ~2.7 GB | [Apache 2.0](https://huggingface.co/Qwen/Qwen3-4B/blob/main/LICENSE) | ✅ |
| Qwen 3 8B (GGUF Q4_K_M) | ~5 GB | [Apache 2.0](https://huggingface.co/Qwen/Qwen3-8B/blob/main/LICENSE) | ✅ |
| Qwen 3 14B (GGUF Q4_K_M) | ~8.4 GB | [Apache 2.0](https://huggingface.co/Qwen/Qwen3-14B/blob/main/LICENSE) | ✅ |
| DeepSeek Coder 6.7B Instruct (GGUF Q4_K_M) | ~3.8 GB | [DeepSeek License v1.0](https://huggingface.co/deepseek-ai/deepseek-coder-6.7b-instruct/blob/main/LICENSE) | ✅ with conditions² |

¹ **Qwen Community License conditions:** no training of competing models, attribution required, no misrepresentation of origin. Organisations with more than 100 million monthly active users must obtain a separate commercial licence from Alibaba Cloud.

² **DeepSeek License v1.0 conditions:** use-based restrictions apply (see Attachment A of the license). Prohibits military use, generation of disinformation, and certain other uses. Governing law is PRC law.

Onde's own license (MIT OR Apache-2.0) is independent of these model licenses. If you build an application on top of Onde, you are responsible for complying with the license of whichever model your users load.

---

## Copyright

© 2026 [Onde Inference](https://ondeinference.com) (Splitfire AB).
