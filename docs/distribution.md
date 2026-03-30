---
title: "Distribution"
description: "How Onde is distributed across package registries — crates.io, Swift Package Index, PyPI, RubyGems, and Maven Central."
---

# Distribution

Onde is published to every major package registry under a consistent naming convention:

| Registry | Package name | Namespace |
| -------- | ------------ | --------- |
| crates.io | `onde` | — |
| Swift Package Index | `onde-swift` | `ondeinference` |
| PyPI | `onde-inference` | — |
| RubyGems | `onde-inference` | — |
| Maven Central | `onde-inference` | `com.ondeinference` |

**Naming rules:**
- Registries with a namespace → `ondeinference` / `onde`
- Registries without a namespace → `onde-inference` (hyphenated) to avoid conflicts
- Swift follows Apple conventions → PascalCase package name (`Onde`), kebab-case repo slug (`onde-swift`)
- The Rust crate uses the bare `onde` name on crates.io

---

## Rust — crates.io

The core Rust crate is published as [`onde`](https://crates.io/crates/onde).

```toml
[dependencies]
onde = "0.1"
```

Publish:

```bash
cargo publish
```

---

## Swift — Swift Package Index

The Swift package is distributed from a dedicated public repository [`ondeinference/onde-swift`](https://github.com/ondeinference/onde-swift) as a remote binary package. The `OndeFramework.xcframework` is attached as a GitHub Release asset; `Package.swift` references it by URL and SHA-256 checksum.

Swift Package Index indexes the `onde-swift` repository automatically on every new Git tag.

### Installation

In Xcode: **File → Add Package Dependencies** and enter:

```
https://github.com/ondeinference/onde-swift
```

Or in `Package.swift`:

```swift
dependencies: [
    .package(url: "https://github.com/ondeinference/onde-swift", from: "0.1.0")
]
```

### Release process

1. Build `OndeFramework.xcframework` from Rust source (see [Swift Package](swift-package#building-the-xcframework-locally)).
2. Zip it and compute the checksum:
   ```bash
   zip -r OndeFramework.xcframework.zip OndeFramework.xcframework
   shasum -a 256 OndeFramework.xcframework.zip
   ```
3. Create a GitHub Release on the Rust source repo and attach the zip as a release asset.
4. In `onde-swift`, update `Package.swift` with the new `url:` and `checksum:`.
5. Commit, tag the version (`0.1.0`, `0.2.0`, etc.), and push — SPI picks it up automatically.

---

## Python — PyPI

The Python package is published as [`onde-inference`](https://pypi.org/project/onde-inference) and imported as `onde_inference`. It uses [maturin](https://github.com/PyO3/maturin) with `bindings = "uniffi"` to generate Python bindings from the same UniFFI scaffolding used for Swift and Kotlin.

### Installation

```bash
pip install onde-inference
```

### Usage

```python
import asyncio
from onde_inference import OndeChatEngine, default_model_config

async def main():
    engine = OndeChatEngine()
    await engine.load_default_model(
        system_prompt="You are a helpful assistant.",
        sampling=None,
    )
    result = await engine.send_message("Hello!")
    print(result.text)

asyncio.run(main())
```

### Prerequisites

- Python 3.9+
- `maturin >= 1.7` — `uv tool install maturin` or `pip install maturin`
- `uniffi-bindgen == 0.31.0` — must match the `uniffi` version pinned in `Cargo.toml`

  ```bash
  pip install uniffi-bindgen==0.31.0
  ```

### Build

```bash
cd sdk/python

# Build a release wheel for the current platform
maturin build --release --out dist

# Development install into the current venv
maturin develop
```

### Publish

```bash
cd sdk/python
maturin publish
```

To publish wheels for all platforms, run `maturin publish` on each platform in CI and pass `--skip-existing` so subsequent uploads don't fail. Platforms to cover:

- macOS: `aarch64-apple-darwin`, `x86_64-apple-darwin`
- Linux: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`
- Windows: `x86_64-pc-windows-msvc`

---

## Ruby — RubyGems

The Ruby gem is published as [`onde-inference`](https://rubygems.org/gems/onde-inference) and required as `onde`. It is a [Magnus](https://github.com/matsadler/magnus)-based native extension exposing Onde's HuggingFace cache management and model metadata to Ruby.

See the [Ruby Gem guide](ruby-gem) for full API reference and Rails integration examples.

### Installation

```bash
gem install onde-inference
```

Or in your `Gemfile`:

```ruby
gem "onde-inference"
```

### Publish

```bash
cd sdk/gem
bundle exec rake compile   # build the native extension
gem build onde-inference.gemspec
gem push onde-inference-*.gem
```

---

## Kotlin / Android — Maven Central

The Kotlin Android library is published as `com.ondeinference:onde-inference` on Maven Central.

### Installation

In `build.gradle.kts`:

```kotlin
dependencies {
    implementation("com.ondeinference:onde-inference:0.1.0")
}
```

### Build

The Android library is built with `build-kotlin.sh` at the crate root. It cross-compiles `libonde.so` for all four Android ABIs and generates Kotlin bindings via `uniffi-bindgen`:

```bash
# Full release build for all Android ABIs
./build-kotlin.sh

# arm64 only (faster for development)
./build-kotlin.sh --target aarch64-linux-android
```

Then assemble and publish the AAR from the Android project:

```bash
cd sdk/kotlin
./gradlew :onde:assembleRelease
./gradlew publishReleasePublicationToMavenCentralRepository
```
