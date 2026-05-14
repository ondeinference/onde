---
name: sdk-branding
description: Onde Inference SDK branding rules. Covers copyright headers for every language, brand identity (logo, colors, badges), README structure, and file classification (generated vs hand-written). Apply when creating or editing any SDK source file, README, or documentation.
allowed-tools: Read, Write, Edit, Glob, Grep
user-invocable: false
---

# SDK Branding

Branding rules for the Onde Inference SDK. These apply across all languages and all SDK surfaces: source files, READMEs, package metadata, and CI output.

---

## Copyright Header

Every hand-written source file starts with this three-line block. No blank line before it. One blank line after it (before the first import or code).

### Format

```
// Copyright 2026 Onde Inference (Splitfire AB). All rights reserved.
// Use of this source code is governed by the MIT license.
//
```

Use `//` comment syntax for all languages that support it: Dart, Swift, Kotlin, Rust, C, C++.

### Language reference

**Dart**
```dart
// Copyright 2026 Onde Inference (Splitfire AB). All rights reserved.
// Use of this source code is governed by the MIT license.
//
import 'dart:async';
```

**Swift**
```swift
// Copyright 2026 Onde Inference (Splitfire AB). All rights reserved.
// Use of this source code is governed by the MIT license.
//
import Foundation
```

**Kotlin**
```kotlin
// Copyright 2026 Onde Inference (Splitfire AB). All rights reserved.
// Use of this source code is governed by the MIT license.
//
package com.ondeinference
```

**Rust**
```rust
// Copyright 2026 Onde Inference (Splitfire AB). All rights reserved.
// Use of this source code is governed by the MIT license.
//
use std::sync::Arc;
```

**C / C++**
```cpp
// Copyright 2026 Onde Inference (Splitfire AB). All rights reserved.
// Use of this source code is governed by the MIT license.
//
#include <flutter/plugin_registrar.h>
```

### Existing file-level comments

If the file already has a descriptive comment block below the copyright position (e.g. `// OndeInferencePlugin.swift\n// Native iOS plugin...`), prepend the copyright header above it — do not replace it.

```swift
// Copyright 2026 Onde Inference (Splitfire AB). All rights reserved.
// Use of this source code is governed by the MIT license.
//
// OndeInferencePlugin.swift
//
// Native iOS plugin for onde_inference.
//
```

---

## File Classification

Apply the header only to hand-written files. Never add it to generated files.

### Always add the header

| File | Notes |
|---|---|
| `lib/src/engine.dart` | Core SDK logic |
| `lib/src/types.dart` | SDK types |
| `lib/onde_inference.dart` | Public barrel file |
| `test/dart_test.dart` | Hand-written unit tests |
| `example/lib/main.dart` | Example app entry point |
| `ios/Classes/OndeInferencePlugin.swift` | iOS plugin |
| `macos/Classes/OndeInferencePlugin.swift` | macOS plugin |
| `android/src/main/kotlin/**/*.kt` | Android plugin |
| `windows/**/*.cpp` | Windows plugin |
| `linux/**/*.cc` / `linux/**/*.h` | Linux plugin |
| `src/**/*.rs` | Rust crate source |

### Never add the header — generated files

These files are written by a tool on every codegen run. Adding a header would be overwritten or cause diffs noise.

| File / Pattern | Generator |
|---|---|
| `lib/src/frb_generated.dart/**` | `flutter_rust_bridge_codegen` |
| `**/*.freezed.dart` | `build_runner` + `freezed` |
| `**/*.g.dart` | `build_runner` |
| `example/*/Flutter/GeneratedPluginRegistrant.*` | `flutter pub get` |
| `generated/**` (repo root) | `uniffi-bindgen` |
| `onde.swift` in `onde-swift` | `uniffi-bindgen` |

### Never add the header — platform scaffold

These files are produced by `flutter create` or Xcode project templates. They are committed to git but were not hand-written by the team.

| Pattern | Tool |
|---|---|
| `example/ios/Runner/AppDelegate.swift` | `flutter create` |
| `example/ios/Runner/SceneDelegate.swift` | `flutter create` |
| `example/ios/RunnerTests/RunnerTests.swift` | `flutter create` |
| `example/macos/Runner/AppDelegate.swift` | `flutter create` |
| `example/macos/Runner/MainFlutterWindow.swift` | `flutter create` |
| `example/macos/RunnerTests/RunnerTests.swift` | `flutter create` |
| `example/test/widget_test.dart` | `flutter create` |

**Rule of thumb:** if the first meaningful line of a file is a Flutter or Xcode framework import with no project-specific logic, it is scaffold — skip it.

---

## Brand Identity

### Logo

Always reference the logo via the raw GitHub URL so it renders on pub.dev, crates.io, and any mirror.

```
https://raw.githubusercontent.com/ondeinference/onde/main/assets/onde-inference-logo.svg
```

Width: `96` for top-level READMEs. Use `72` for secondary pages (example apps, sub-package docs).

### Colors

| Role | Hex |
|---|---|
| Brand green (foreground, badge color) | `#235843` |
| Dark background (badge `labelColor`) | `#17211D` |

### Badge row

Badge policy depends on the README type.

**Root README (`onde/README.md`)**

Use the full ecosystem badge wall here. This is the hub page for the whole SDK family.

```html
<a href="https://crates.io/crates/onde"><img src="https://img.shields.io/crates/v/onde?style=flat-square&color=235843&labelColor=17211D&label=crates.io" alt="crates.io"></a>
<a href="https://central.sonatype.com/artifact/com.ondeinference/onde-inference"><img src="https://img.shields.io/maven-central/v/com.ondeinference/onde-inference?style=flat-square&color=235843&labelColor=17211D&label=maven" alt="Maven Central"></a>
<a href="https://swiftpackageindex.com/ondeinference/onde-swift"><img src="https://img.shields.io/badge/Swift%20Package%20Index-onde--swift-235843?style=flat-square&labelColor=17211D" alt="Swift Package Index"></a>
<a href="https://pub.dev/packages/onde_inference"><img src="https://img.shields.io/pub/v/onde_inference?style=flat-square&color=235843&labelColor=17211D&label=pub.dev" alt="pub.dev"></a>
<a href="https://www.npmjs.com/package/@ondeinference/react-native"><img src="https://img.shields.io/npm/v/@ondeinference/react-native?style=flat-square&color=235843&labelColor=17211D&label=npm" alt="npm"></a>
<a href="https://ondeinference.com"><img src="https://img.shields.io/badge/ondeinference.com-235843?style=flat-square&labelColor=17211D" alt="Website"></a>
<a href="https://apps.apple.com/se/developer/splitfire-ab/id1831430993"><img src="https://img.shields.io/badge/App%20Store-live-235843?style=flat-square&labelColor=17211D" alt="App Store"></a>
```

**SDK README**

Use one primary registry badge for that SDK, plus `Website` and `License`. Do not repeat the full ecosystem badge wall on every SDK page.

**Flutter / Dart SDK:**
```html
<a href="https://pub.dev/packages/onde_inference"><img src="https://img.shields.io/pub/v/onde_inference?style=flat-square&color=235843&labelColor=17211D&label=pub.dev" alt="pub.dev"></a>
<a href="https://ondeinference.com"><img src="https://img.shields.io/badge/ondeinference.com-235843?style=flat-square&labelColor=17211D" alt="Website"></a>
<a href="https://github.com/ondeinference/onde/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-235843?style=flat-square&labelColor=17211D" alt="License"></a>
```

**Rust SDK:**
```html
<a href="https://crates.io/crates/onde"><img src="https://img.shields.io/crates/v/onde?style=flat-square&color=235843&labelColor=17211D&label=crates.io" alt="crates.io"></a>
<a href="https://ondeinference.com"><img src="https://img.shields.io/badge/ondeinference.com-235843?style=flat-square&labelColor=17211D" alt="Website"></a>
<a href="https://github.com/ondeinference/onde/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-235843?style=flat-square&labelColor=17211D" alt="License"></a>
```

**Kotlin Multiplatform SDK:**
```html
<a href="https://central.sonatype.com/artifact/com.ondeinference/onde-inference"><img src="https://img.shields.io/maven-central/v/com.ondeinference/onde-inference?style=flat-square&color=235843&labelColor=17211D&label=maven" alt="Maven Central"></a>
<a href="https://ondeinference.com"><img src="https://img.shields.io/badge/ondeinference.com-235843?style=flat-square&labelColor=17211D" alt="Website"></a>
<a href="https://github.com/ondeinference/onde/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-235843?style=flat-square&labelColor=17211D" alt="License"></a>
```

**React Native SDK:**
```html
<a href="https://www.npmjs.com/package/@ondeinference/react-native"><img src="https://img.shields.io/npm/v/@ondeinference/react-native?style=flat-square&color=235843&labelColor=17211D&label=npm" alt="npm"></a>
<a href="https://ondeinference.com"><img src="https://img.shields.io/badge/ondeinference.com-235843?style=flat-square&labelColor=17211D" alt="Website"></a>
<a href="https://github.com/ondeinference/onde/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-235843?style=flat-square&labelColor=17211D" alt="License"></a>
```

**Swift SDK:**
```html
<a href="https://swiftpackageindex.com/ondeinference/onde-swift"><img src="https://img.shields.io/badge/Swift%20Package%20Index-onde--swift-235843?style=flat-square&labelColor=17211D" alt="Swift Package Index"></a>
<a href="https://ondeinference.com"><img src="https://img.shields.io/badge/ondeinference.com-235843?style=flat-square&labelColor=17211D" alt="Website"></a>
<a href="https://github.com/ondeinference/onde/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-235843?style=flat-square&labelColor=17211D" alt="License"></a>
```

### Cross-SDK nav row

Every SDK README should include a centered text-link row under the badges. These sibling links matter for discovery, branding, and cross-SDK navigation.

Use this pattern and adjust the current page's own SDK label if needed:

```html
<p align="center">
  <a href="https://github.com/ondeinference/onde">Rust SDK</a> · <a href="https://github.com/ondeinference/onde-swift">Swift SDK</a> · <a href="https://central.sonatype.com/artifact/com.ondeinference/onde-inference">Kotlin Multiplatform SDK</a> · <a href="https://pub.dev/packages/onde_inference">Flutter SDK</a> · <a href="https://www.npmjs.com/package/@ondeinference/react-native">React Native SDK</a> · <a href="https://ondeinference.com">Website</a>
</p>
```

### Footer

Every README ends with this line, inside a centered `<p>`:

```html
<p align="center">
  <sub>© 2026 <a href="https://ondeinference.com">Onde Inference</a> — MIT License</sub>
</p>
```

Use `— MIT License` on sub-pages (example apps, package-level docs). Omit it on the top-level repo README (it has its own license section).

---

## README Structure

### Top-level SDK README (e.g. `sdk/dart/README.md`)

1. Centered logo block
2. Centered `<h1>` with SDK name
3. Centered tagline (`<p><strong>...</strong></p>`)
4. Centered badge row with the SDK's primary registry badge, `Website`, and `License`
5. Centered cross-SDK nav links (`Rust SDK · Swift SDK · Kotlin Multiplatform SDK · Flutter SDK · React Native SDK · Website`)
6. `---` divider
7. A direct intro paragraph that explains what runs where and what the user gets
8. `## Platform support` — table: Platform | GPU backend | Default model | Notes
9. `## Quick start` — install snippet, initialization, then usage sections
10. `## Model selection` — table + code examples when relevant
11. `## Sampling` — presets table + code when relevant
12. `## Error handling` when relevant
13. `## Contributing`
14. `## License`
15. Footer

### Example app README

1. Centered logo block (width `96`)
2. Centered `<h1>` with `Example App` in the title
3. Centered one-line description (bold) + second line: `No server. No API key. No data leaving the device.`
4. Centered badge row for the parent SDK or example context
5. `---` divider
6. `## What this example shows` — feature table mapping feature → file/function
7. `## Running the example` — Prerequisites, Steps, Platform notes table
8. `## SDK quick reference` — minimal happy-path snippet (load → stream → unload)
9. `## Project structure` — directory tree
10. `## Learn more` — links to the package page, ondeinference.com, and GitHub
11. Footer

### Taglines

Use these as the default direction, but keep them natural if the page needs slightly different wording. Avoid stiff marketing language and avoid em dashes in prose.

| Context | Tagline |
|---|---|
| Rust crate | `Run LLMs on-device, with first-class support for Apple silicon.` |
| Flutter SDK | `Run LLMs on-device from Flutter and Dart. Metal on iOS and macOS, CPU everywhere else.` |
| Kotlin SDK | `Run LLMs on-device from Kotlin. No cloud, no API key, and no user data leaving the device.` |
| React Native SDK | `Run LLMs on-device from React Native. Metal on iOS, CPU on Android, and no user data leaving the device.` |
| Swift SDK | `Run LLMs on-device from Swift, with first-class support for Apple silicon.` |
| Example app | `A Flutter chat app that runs LLM inference fully on-device.` |

---

## Package Metadata Branding

### `pubspec.yaml`

```yaml
description: >-
  On-device LLM inference for Flutter & Dart. Run Qwen 2.5 models locally
  with Metal on iOS and macOS, CPU on Android and desktop. No cloud, no API key.
repository: https://github.com/ondeinference/onde/
homepage: https://ondeinference.com
issue_tracker: https://github.com/ondeinference/onde/issues
```

### `Cargo.toml`

```toml
description = "On-device inference engine for Apple silicon."
license = "MIT"
repository = "https://github.com/ondeinference/onde"
homepage = "https://ondeinference.com"
documentation = "https://docs.rs/onde"
keywords = ["inference", "on-device", "chat", "llm", "mistral"]
categories = ["science::ml", "api-bindings"]
```

---

## Version Synchronisation

The version in `Cargo.toml` and `sdk/dart/pubspec.yaml` must always be identical. When bumping a version, update both files in the same commit. The CI workflows for Swift and Dart both validate that the git tag equals the version in their respective metadata file — a mismatch fails fast.

```
Cargo.toml        version = "X.Y.Z"
sdk/dart/pubspec.yaml   version: X.Y.Z
git tag                 X.Y.Z
```

---

## Applying the Copyright Header to an Existing Codebase

When asked to add copyright headers to a set of files:

1. Run `find` or `Glob` to list all `.dart`, `.swift`, `.kt`, `.rs`, `.cpp`, `.cc`, `.h` files under the SDK path.
2. For each file, read its first line.
3. Skip if the first line already starts with `// Copyright`.
4. Skip if the first line contains `Generated` or `Do not edit`.
5. Skip if the file path matches any generated or scaffold pattern above.
6. Prepend the three-line header followed by a blank line.
7. If the file begins with an existing file-description comment block, insert the header above that block.

Do all qualifying files in one pass. Do not ask for confirmation per file.